#!/usr/bin/env python3
"""Generate the API's Anchor account decoders from the checked-in IDL.

Every account type in `idl/terra_registry.json` gets an IDL-derived decoder:
no hand-maintained byte offsets, string lengths or array bounds. `--check` in
CI fails when the generated file drifts from the IDL (R-042, ADR-027).

Q-033 (b) staged rollout: stage 1 mirrors *all* account types into the generic
`chain_accounts` read model with transaction-level provenance and generates a
per-type acceptance test for each. `RICH_ACCOUNTS` are the types that also get
a typed projection in `indexer.rs`; promoting a type from the generic mirror
to a typed table is a later stage, not a reason to leave it undecoded.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IDL = ROOT / "idl" / "terra_registry.json"
OUTPUT = ROOT / "api" / "src" / "indexer_decoders.rs"

# Typed projections in indexer.rs (the original 8). The generic mirror covers
# every account in the IDL, including these.
RICH_ACCOUNTS = [
    "Parcel",
    "VerificationAttestation",
    "Rights",
    "Claim",
    "SpatialAsset",
    "GeometryVersion",
    "ThreeDModel",
    "SpatialSnapshot",
]

HEADER = """// GENERATED FILE — DO NOT EDIT.
// Regenerate: python3 terra-core/scripts/generate-indexer-decoders.py
// Source: terra-core/idl/terra_registry.json (all Anchor accounts + types).
// CI pin: `python3 scripts/generate-indexer-decoders.py --check`.
//
// Q-033 stage 1: every IDL account type is decodable and mirrored into the
// generic `chain_accounts` table. `RICH_ACCOUNT_NAMES` marks the types with an
// additional typed projection in `indexer.rs`.

//! IDL-derived Borsh decoders for every account in `terra_registry`.
//! String/vec/bytes length prefixes are u32 LE (Borsh), never u8/u16.
//! Trailing Anchor allocation padding is allowed; truncated buffers, invalid
//! UTF-8 and oversized untrusted lengths are rejected.

// Every IDL account gets a decoder; only the rich projections are called from
// non-test code. `decode_any` is the generic mirror path (Q-033 stage 1).
#![allow(dead_code, unused_mut)]

/// Upper bounds for untrusted length prefixes read from RPC account data.
pub const MAX_STRING_LEN: usize = 4096;
pub const MAX_BYTES_LEN: usize = 65536;
pub const MAX_VEC_LEN: usize = 256;

#[derive(Debug, Clone)]
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)] // Generated for every IDL primitive; not every account uses each one.
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(len)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn boolean(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn bytes32(&mut self) -> Option<[u8; 32]> {
        self.take(32)?.try_into().ok()
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        if len > MAX_STRING_LEN {
            return None;
        }
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).ok().map(str::to_string)
    }

    fn blob(&mut self) -> Option<Vec<u8>> {
        let len = self.u32()? as usize;
        if len > MAX_BYTES_LEN {
            return None;
        }
        Some(self.take(len)?.to_vec())
    }
}

fn discriminator(data: &[u8], expected: &[u8; 8]) -> bool {
    data.len() >= 8 && &data[..8] == expected
}

"""

FOOTER = """
/// Every account type in the checked-in IDL. Used by the staged-rollout
/// acceptance tests and by `/mirror/audit` coverage reporting.
pub const ALL_ACCOUNT_NAMES: &[&str] = &[
%(all_names)s
];

/// Account types with a typed projection in `indexer.rs` (stage 0).
/// The rest are mirrored generically into `chain_accounts` (stage 1).
pub const RICH_ACCOUNT_NAMES: &[&str] = &[
%(rich_names)s
];

/// All discriminators the indexer scans for. Used by the two-phase RPC read
/// (header-only program scan followed by a batched full-account fetch).
pub const INDEXER_DISCRIMINATORS: &[[u8; 8]] = &[
%(discriminators)s
];

/// Human-readable name for an account discriminator.
pub fn mirrored_account_name(data: &[u8]) -> Option<&'static str> {
%(name_matches)s
    None
}

/// Decode any known account into its name and canonical JSON view.
/// The JSON is stored in `chain_accounts.fields` with transaction provenance.
pub fn decode_any(data: &[u8]) -> Option<(&'static str, serde_json::Value)> {
%(decode_matches)s
    None
}
"""

ACCEPTANCE_HEADER = """
#[cfg(test)]
/// Per-type acceptance tests (Q-035/Q-033): each IDL account has a fixture
/// built from its own field spec. A type is only accepted if its decoder,
/// `decode_any` classification and JSON projection all agree.
mod idl_acceptance_tests {
    use super::*;

    fn byte_list(values: &[u8]) -> String {
        values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
    }
"""

SCALARS = {
    "u8": ("u8", "r.u8()?", "0u8", [7]),
    "u16": ("u16", "r.u16()?", "0u16", [7, 0]),
    "u32": ("u32", "r.u32()?", "0u32", [7, 0, 0, 0]),
    "u64": ("u64", "r.u64()?", "0u64", [7, 0, 0, 0, 0, 0, 0, 0]),
    "i32": ("i32", "r.i32()?", "0i32", [7, 0, 0, 0]),
    "i64": ("i64", "r.i64()?", "0i64", [7, 0, 0, 0, 0, 0, 0, 0]),
    "bool": ("bool", "r.boolean()?", "false", [1]),
    "pubkey": ("[u8; 32]", "r.bytes32()?", "[0u8; 32]", [7] * 32),
    "string": ("String", "r.string()?", "String::new()", [1, 0, 0, 0, 120]),
    "bytes": ("Vec<u8>", "r.blob()?", "Vec::new()", [1, 0, 0, 0, 170]),
}


def snake(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def screaming(name: str) -> str:
    return snake(name).upper()


def rust_type(t, types) -> str:
    if isinstance(t, str):
        if t in SCALARS:
            return SCALARS[t][0]
        raise ValueError(f"unsupported scalar type: {t}")
    if "array" in t:
        inner, count = t["array"]
        # serde only derives for arrays up to 32 elements. Larger Borsh arrays
        # are decoded into a Vec with an exact element count instead.
        if count > 32:
            return f"Vec<{rust_type(inner, types)}>"
        return f"[{rust_type(inner, types)}; {count}]"
    if "vec" in t:
        return f"Vec<{rust_type(t['vec'], types)}>"
    if "option" in t:
        return f"Option<{rust_type(t['option'], types)}>"
    if "defined" in t:
        return f"Parsed{t['defined']['name']}"
    raise ValueError(f"unsupported IDL type: {t}")


def default_expr(t) -> str:
    if isinstance(t, str):
        if t in SCALARS:
            return SCALARS[t][2]
        raise ValueError(f"unsupported scalar type: {t}")
    if "array" in t:
        inner, count = t["array"]
        if count > 32:
            return "Vec::new()"
        return f"[{default_expr(inner)}; {count}]"
    if "vec" in t or "bytes" in t:
        return "Vec::new()"
    if "option" in t:
        return "None"
    if "defined" in t:
        return f"Parsed{t['defined']['name']}::default()"
    raise ValueError(f"unsupported IDL type: {t}")


def read_expr(t) -> str:
    if isinstance(t, str):
        if t in SCALARS:
            return SCALARS[t][1]
        raise ValueError(f"unsupported scalar type: {t}")
    if "array" in t:
        inner, count = t["array"]
        if count > 32:
            return (
                f"{{ let mut out = Vec::with_capacity({count}); "
                + f"for _ in 0..{count} {{ out.push({read_expr(inner)}); }} out }}"
            )
        return (
            "{ let mut out: "
            + rust_type(t, None)
            + " = ["
            + default_expr(inner)
            + f"; {count}]; for item in &mut out {{ *item = {read_expr(inner)}; }} out }}"
        )
    if "vec" in t:
        inner = t["vec"]
        return (
            "{ let len = r.u32()? as usize; if len > MAX_VEC_LEN { return None; } "
            + "let mut out = Vec::with_capacity(len); "
            + f"for _ in 0..len {{ out.push({read_expr(inner)}); }} out }}"
        )
    if "option" in t:
        inner = t["option"]
        return f"match r.u8()? {{ 0 => None, 1 => Some({read_expr(inner)}), _ => return None }}"
    if "defined" in t:
        return f"read_{snake(t['defined']['name'])}(&mut *r)?"
    raise ValueError(f"unsupported IDL type: {t}")


def fixture(t) -> list[int]:
    """Deterministic minimal byte fixture for an IDL type."""
    if isinstance(t, str):
        if t in SCALARS:
            return list(SCALARS[t][3])
        raise ValueError(f"unsupported scalar type: {t}")
    if "array" in t:
        inner, count = t["array"]
        return fixture(inner) * count
    if "vec" in t:
        return [1, 0, 0, 0] + fixture(t["vec"])
    if "option" in t:
        return [1] + fixture(t["option"])
    if "defined" in t:
        return []
    raise ValueError(f"unsupported IDL type: {t}")


def defined_fixture(t, types) -> list[int]:
    """Byte fixture that also expands nested `defined` structs."""
    if isinstance(t, str):
        return fixture(t)
    if "defined" in t:
        out: list[int] = []
        for field in struct_fields(t["defined"]["name"], types):
            out.extend(defined_fixture(field["type"], types))
        return out
    if "array" in t:
        inner, count = t["array"]
        return defined_fixture(inner, types) * count
    if "vec" in t:
        return [1, 0, 0, 0] + defined_fixture(t["vec"], types)
    if "option" in t:
        return [1] + defined_fixture(t["option"], types)
    raise ValueError(f"unsupported IDL type: {t}")


def struct_fields(name: str, types) -> list:
    return types[name]["type"]["fields"]


def collect_defined(t, found, types) -> None:
    if isinstance(t, dict):
        if "defined" in t:
            name = t["defined"]["name"]
            if name not in found:
                found.add(name)
                for field in struct_fields(name, types):
                    collect_defined(field["type"], found, types)
        for value in t.values():
            if isinstance(value, (dict, list)):
                collect_defined(value, found, types)
    elif isinstance(t, list):
        for value in t:
            collect_defined(value, found, types)


def struct_block(name: str, fields, types, *, nested: bool) -> str:
    out = []
    struct = f"Parsed{name}"
    label = "nested IDL struct" if nested else "IDL account"
    out.append(f"/// {label} `{name}` decoded from Borsh bytes.\n")
    out.append("#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\n")
    out.append(f"pub struct {struct} {{\n")
    for field in fields:
        out.append(f"    pub {field['name']}: {rust_type(field['type'], types)},\n")
    out.append("}\n\n")
    reader = f"read_{snake(name)}"
    out.append(f"/// Read `{name}` fields (no discriminator) from a Borsh reader.\n")
    out.append(f"fn {reader}(mut r: &mut Reader) -> Option<{struct}> {{\n")
    out.append(f"    Some({struct} {{\n")
    for field in fields:
        out.append(f"        {field['name']}: {read_expr(field['type'])},\n")
    out.append("    })\n}\n\n")
    if not nested:
        disc = f"{screaming(name)}_DISCRIMINATOR"
        out.append(f"/// Decode an Anchor `{name}` account buffer.\n")
        out.append(f"pub fn parse_{snake(name)}(data: &[u8]) -> Option<{struct}> {{\n")
        out.append(f"    if !discriminator(data, &{disc}) {{\n        return None;\n    }}\n")
        out.append("    let mut r = Reader::new(data);\n")
        out.append("    r.take(8)?;\n")
        out.append(f"    {reader}(&mut r)\n}}\n\n")
    return "".join(out)


def generate() -> str:
    idl = json.loads(IDL.read_text())
    accounts = {a["name"]: a for a in idl["accounts"]}
    types = {t["name"]: t for t in idl["types"]}
    names = sorted(accounts)

    # Recursive nested types reached through `defined` (e.g. CountryAllocation).
    found: set[str] = set()
    for name in names:
        for field in struct_fields(name, types):
            collect_defined(field["type"], found, types)

    out = [HEADER]
    for name in sorted(found):
        out.append(struct_block(name, struct_fields(name, types), types, nested=True))
    for name in names:
        disc = accounts[name]["discriminator"]
        assert len(disc) == 8, name
        const = f"{screaming(name)}_DISCRIMINATOR"
        out.append(f"/// Anchor discriminator from the IDL for `{name}`.\n")
        out.append(
            f"pub const {const}: [u8; 8] = ["
            + ", ".join(str(b) for b in disc)
            + "];\n\n"
        )
        out.append(struct_block(name, struct_fields(name, types), types, nested=False))

    out.append(
        FOOTER
        % {
            "all_names": "\n".join(f'    "{name}",' for name in names),
            "rich_names": "\n".join(
                f'    "{name}",' for name in RICH_ACCOUNTS if name in accounts
            ),
            "discriminators": "\n".join(
                f"    {screaming(name)}_DISCRIMINATOR," for name in names
            ),
            "name_matches": "\n".join(
                f"    if discriminator(data, &{screaming(name)}_DISCRIMINATOR) {{\n"
                f'        return Some("{name}");\n'
                "    }"
                for name in names
            ),
            "decode_matches": "\n".join(
                f"    if discriminator(data, &{screaming(name)}_DISCRIMINATOR) {{\n"
                f"        return parse_{snake(name)}(data)\n"
                "            .and_then(|value| serde_json::to_value(value).ok())\n"
                f'            .map(|json| ("{name}", json));\n'
                "    }"
                for name in names
            ),
        }
    )

    out.append(ACCEPTANCE_HEADER)
    out.append(
        "    #[test]\n"
        "    fn every_idl_account_has_a_unique_discriminator() {\n"
        f"        assert_eq!(ALL_ACCOUNT_NAMES.len(), {len(names)});\n"
        "        let mut seen = std::collections::HashSet::new();\n"
        "        for name in ALL_ACCOUNT_NAMES {\n"
        "            assert!(seen.insert(*name), \"duplicate account name {name}\");\n"
        "        }\n"
        "        let mut discs = std::collections::HashSet::new();\n"
        "        for disc in INDEXER_DISCRIMINATORS {\n"
        "            assert!(discs.insert(*disc), \"duplicate discriminator {disc:?}\");\n"
        "        }\n"
        "    }\n\n"
    )
    for name in names:
        fields = struct_fields(name, types)
        disc = ", ".join(str(b) for b in accounts[name]["discriminator"])
        body: list[int] = []
        for field in fields:
            body.extend(defined_fixture(field["type"], types))
        field_names = ", ".join(f'"{f["name"]}"' for f in fields)
        # Assert the decoder succeeds and every field appears in the JSON
        # projection; the fixture bytes are built from the IDL field spec.
        out.append(
            f"    #[test]\n"
            f"    fn accepts_{snake(name)}() {{\n"
            f"        let mut data = vec![{disc}];\n"
            f"        data.extend_from_slice(&[{', '.join(str(b) for b in body)}]);\n"
            f"        let parsed = parse_{snake(name)}(&data)\n"
            f"            .expect(\"{name} IDL fixture must decode\");\n"
            f"        let _ = parsed;\n"
            f"        let (decoded_name, json) = decode_any(&data)\n"
            f'            .expect("decode_any must classify {name}");\n'
            f'        assert_eq!(decoded_name, "{name}");\n'
            f"        let object = json.as_object().expect(\"JSON object\");\n"
            f"        assert_eq!(object.len(), {len(fields)});\n"
            f"        for field in [{field_names}] {{\n"
            f'            assert!(object.contains_key(field), "missing field {{field}}");\n'
            "        }\n"
            "    }\n\n"
        )
    out.append("}\n")
    return "".join(out)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if the generated file differs from the IDL-derived output",
    )
    args = parser.parse_args()
    # Keep the generated file identical to `cargo fmt --check` output so the
    # generator pin and the Rust formatting gate cannot contradict each other.
    rendered = subprocess.run(
        ["rustfmt", "--edition", "2024"],
        input=generate(),
        text=True,
        capture_output=True,
        check=True,
    ).stdout
    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else ""
        if current != rendered:
            print(
                f"ERROR: {OUTPUT} is stale. Regenerate with "
                "python3 terra-core/scripts/generate-indexer-decoders.py",
                file=sys.stderr,
            )
            return 1
        print("indexer decoders match IDL")
        return 0
    OUTPUT.write_text(rendered)
    print(f"wrote {OUTPUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
