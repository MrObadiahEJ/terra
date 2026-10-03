//! `proof_data` framing (v1) and Groth16 proof verification over BN254.
//!
//! Verification runs on the Solana `sol_alt_bn128_group_op` syscalls
//! (EVM precompile-compatible group operations, big-endian / EIP-196+197
//! encodings) through the `solana-bn254` wrappers — no curve arithmetic is
//! performed in the program itself. See RFC-011 §6.3.
//!
//! Frame layout (exactly [`ZK_FRAME_LEN`] bytes):
//!
//! ```text
//! "TG16" | ver=1 | Ed25519 statement sig (64) | VK (576) | proof (256)
//! ```
//!
//! The verification key is hash-pinned per zone root via the existing
//! `update_verification_key_hash` instruction; the Ed25519 signature binds
//! the frame to the canonical ownership statement; the Groth16 public input
//! is the first 31 bytes of `SHA-256(statement)` (248 bits), binding the proof
//! to the same statement while remaining a canonical BN254 scalar.

use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Frame layout
// ---------------------------------------------------------------------------

/// `proof_data` frame magic.
pub const ZK_FRAME_MAGIC: [u8; 4] = *b"TG16";
/// Current `proof_data` frame version.
pub const ZK_FRAME_VERSION: u8 = 1;
/// Ed25519 signature length.
pub const ZK_SIG_LEN: usize = 64;
/// Verification key: α(64) | β(128) | γ(128) | δ(128) | γ·abc[0](64) | γ·abc[1](64).
/// Exactly two `gamma_abc` points ⇒ one circuit public input (k = 1).
pub const ZK_VK_LEN: usize = 576;
/// Groth16 proof: A(64) | B(128) | C(64).
pub const ZK_PROOF_LEN: usize = 256;
/// Total frame length (4 + 1 + 64 + 576 + 256 = 901).
pub const ZK_FRAME_LEN: usize = 4 + 1 + ZK_SIG_LEN + ZK_VK_LEN + ZK_PROOF_LEN;

/// Offset of the Ed25519 signature inside the frame.
pub const ZK_FRAME_OFFSET_SIG: usize = 5;
/// Offset of the verification key inside the frame.
pub const ZK_FRAME_OFFSET_VK: usize = ZK_FRAME_OFFSET_SIG + ZK_SIG_LEN;
/// Offset of the Groth16 proof inside the frame.
pub const ZK_FRAME_OFFSET_PROOF: usize = ZK_FRAME_OFFSET_VK + ZK_VK_LEN;

/// Borrowed view of a well-formed `proof_data` v1 frame.
pub struct OwnershipProofFrame<'a> {
    /// Prover's Ed25519 signature over the canonical statement.
    pub signature: &'a [u8; ZK_SIG_LEN],
    /// Hash-pinned Groth16 verification key (raw bytes).
    pub verification_key: &'a [u8; ZK_VK_LEN],
    /// Groth16 proof (A | B | C, big-endian).
    pub proof: &'a [u8; ZK_PROOF_LEN],
}

/// Parse and structurally validate a `proof_data` v1 frame. Every failure
/// maps to `InvalidProofData` (6087) — framing, statement, key-pinning and
/// pairing failures are deliberately indistinguishable on chain.
pub fn parse_ownership_frame(data: &[u8]) -> Result<OwnershipProofFrame<'_>> {
    require!(data.len() == ZK_FRAME_LEN, TerraError::InvalidProofData);
    require!(data[..4] == ZK_FRAME_MAGIC, TerraError::InvalidProofData);
    require!(data[4] == ZK_FRAME_VERSION, TerraError::InvalidProofData);
    let signature = <&[u8; ZK_SIG_LEN]>::try_from(&data[5..5 + ZK_SIG_LEN])
        .map_err(|_| error!(TerraError::InvalidProofData))?;
    let verification_key =
        <&[u8; ZK_VK_LEN]>::try_from(&data[ZK_FRAME_OFFSET_VK..ZK_FRAME_OFFSET_VK + ZK_VK_LEN])
            .map_err(|_| error!(TerraError::InvalidProofData))?;
    let proof = <&[u8; ZK_PROOF_LEN]>::try_from(
        &data[ZK_FRAME_OFFSET_PROOF..ZK_FRAME_OFFSET_PROOF + ZK_PROOF_LEN],
    )
    .map_err(|_| error!(TerraError::InvalidProofData))?;
    Ok(OwnershipProofFrame {
        signature,
        verification_key,
        proof,
    })
}

// ---------------------------------------------------------------------------
// Public input: 248-bit SHA-256(statement) prefix
// ---------------------------------------------------------------------------

/// BN254 scalar field modulus `r` (big-endian).
const FR_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

/// BN254 base field modulus `Fq` (big-endian), used for G1 negation.
const FQ_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c, 0xfd, 0x47,
];

/// `a -= b` for big-endian 256-bit values. Requires `a >= b`.
fn be_sub_assign(a: &mut [u8; 32], b: &[u8; 32]) {
    let mut borrow = 0u16;
    for i in (0..32).rev() {
        let d = a[i] as i32 - b[i] as i32 - borrow as i32;
        if d < 0 {
            a[i] = (d + 256) as u8;
            borrow = 1;
        } else {
            a[i] = d as u8;
            borrow = 0;
        }
    }
}

/// `a - b` for big-endian 256-bit values. Requires `a >= b`.
fn be_sub(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut out = *a;
    be_sub_assign(&mut out, b);
    out
}

/// Reduce a big-endian 256-bit integer modulo the BN254 scalar field `r`.
/// Inputs are SHA-256 digests (< 2^256 < 6r), so the loop runs at most 5
/// times.
pub fn fr_reduce(mut x: [u8; 32]) -> [u8; 32] {
    while x >= FR_MODULUS {
        be_sub_assign(&mut x, &FR_MODULUS);
    }
    x
}

/// Groth16 public input: the first 31 bytes of SHA-256(statement), interpreted
/// as a big-endian 248-bit integer. The leading zero guarantees a canonical
/// scalar and avoids an in-circuit modular reduction.
pub fn public_input(statement: &[u8]) -> [u8; 32] {
    let digest = solana_program::hash::hash(statement).to_bytes();
    let mut out = [0u8; 32];
    out[1..].copy_from_slice(&digest[..31]);
    out
}

// ---------------------------------------------------------------------------
// Groth16 verification (BN254 pairing equation via alt_bn128 syscalls)
// ---------------------------------------------------------------------------

/// Negate a big-endian G1 point (EIP-196: `x | y`): `(x, Fq - y)`. Points
/// with `y = 0` (including the identity marker `0 | 0`) are unchanged
/// because `-0 = 0`.
fn g1_negate(point: &[u8; 64]) -> [u8; 64] {
    let mut y = [0u8; 32];
    y.copy_from_slice(&point[32..64]);
    if y == [0u8; 32] {
        return *point;
    }
    let mut out = *point;
    out[32..64].copy_from_slice(&be_sub(&FQ_MODULUS, &y));
    out
}

/// Verify a Groth16/BN254 proof against a single `Fr` public input.
///
/// Checks the standard pairing equation
/// `e(A, B) · e(-α, β) · e(-L, γ) · e(-C, δ) == 1`
/// with `L = γ·abc[0] + x · γ·abc[1]`, evaluated as one 4-pair
/// `alt_bn128` pairing product. Returns `false` for any malformed input,
/// syscall error, or pairing mismatch. Point validity (on-curve checks) is
/// performed by the syscall itself.
pub fn verify_groth16(vk: &[u8; ZK_VK_LEN], proof: &[u8; ZK_PROOF_LEN], x: &[u8; 32]) -> bool {
    // L = gamma_abc[0] + x * gamma_abc[1]  (1 scalar mul + 1 add)
    let mut mul_in = [0u8; 96];
    mul_in[..64].copy_from_slice(&vk[512..576]);
    mul_in[64..].copy_from_slice(x);
    let mul_out: [u8; 64] = match solana_bn254::prelude::alt_bn128_g1_multiplication_be(&mul_in) {
        Ok(out) => match <&[u8; 64]>::try_from(out.as_slice()) {
            Ok(p) => *p,
            Err(_) => return false,
        },
        Err(_) => return false,
    };
    let mut add_in = [0u8; 128];
    add_in[..64].copy_from_slice(&vk[448..512]);
    add_in[64..].copy_from_slice(&mul_out);
    let l: [u8; 64] = match solana_bn254::prelude::alt_bn128_g1_addition_be(&add_in) {
        Ok(out) => match <&[u8; 64]>::try_from(out.as_slice()) {
            Ok(p) => *p,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // e(A,B) · e(-α,β) · e(-L,γ) · e(-C,δ) == 1
    let mut pairing_in = [0u8; 768];
    let (mut neg_alpha, mut neg_l, mut neg_c) = ([0u8; 64], [0u8; 64], [0u8; 64]);
    neg_alpha.copy_from_slice(&vk[0..64]);
    neg_l.copy_from_slice(&l);
    neg_c.copy_from_slice(&proof[192..256]);
    neg_alpha = g1_negate(&neg_alpha);
    neg_l = g1_negate(&neg_l);
    neg_c = g1_negate(&neg_c);
    pairing_in[0..64].copy_from_slice(&proof[0..64]); // A
    pairing_in[64..192].copy_from_slice(&proof[64..192]); // B
    pairing_in[192..256].copy_from_slice(&neg_alpha); // -α
    pairing_in[256..384].copy_from_slice(&vk[64..192]); // β
    pairing_in[384..448].copy_from_slice(&neg_l); // -L
    pairing_in[448..576].copy_from_slice(&vk[192..320]); // γ
    pairing_in[576..640].copy_from_slice(&neg_c); // -C
    pairing_in[640..768].copy_from_slice(&vk[320..448]); // δ

    match solana_bn254::prelude::alt_bn128_pairing_be(&pairing_in) {
        Ok(out) => out.len() == 32 && out[..31].iter().all(|&b| b == 0) && out[31] == 1,
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_frame() -> Vec<u8> {
        let mut f = Vec::with_capacity(ZK_FRAME_LEN);
        f.extend_from_slice(&ZK_FRAME_MAGIC);
        f.push(ZK_FRAME_VERSION);
        for i in 0..(ZK_FRAME_LEN - 5) {
            f.push((i % 251) as u8);
        }
        f
    }

    #[test]
    fn frame_roundtrip() {
        let f = dummy_frame();
        assert_eq!(f.len(), ZK_FRAME_LEN);
        let frame = parse_ownership_frame(&f).expect("valid frame");
        assert_eq!(frame.signature[..], f[5..69]);
        assert_eq!(frame.verification_key[..], f[69..645]);
        assert_eq!(frame.proof[..], f[645..901]);
    }

    #[test]
    fn frame_rejects_bad_magic_version_and_length() {
        let f = dummy_frame();
        assert!(parse_ownership_frame(&[]).is_err());
        assert!(parse_ownership_frame(&[0u8; 64]).is_err());
        assert!(parse_ownership_frame(&f[..ZK_FRAME_LEN - 1]).is_err());
        let mut bad_magic = f.clone();
        bad_magic[0] = b'X';
        assert!(parse_ownership_frame(&bad_magic).is_err());
        let mut bad_version = f.clone();
        bad_version[4] = 2;
        assert!(parse_ownership_frame(&bad_version).is_err());
    }

    #[test]
    fn fr_reduce_known_vectors() {
        assert_eq!(fr_reduce([0u8; 32]), [0u8; 32]);
        assert_eq!(fr_reduce(FR_MODULUS), [0u8; 32]);
        let mut r_minus_one = FR_MODULUS;
        r_minus_one[31] -= 1;
        assert_eq!(fr_reduce(r_minus_one), r_minus_one);
        // r + 1 -> 1
        let mut r_plus_one = FR_MODULUS;
        r_plus_one[31] += 1;
        assert_eq!(fr_reduce(r_plus_one), {
            let mut one = [0u8; 32];
            one[31] = 1;
            one
        });
        assert_eq!(
            fr_reduce([0xff; 32]),
            [
                0x0e, 0x0a, 0x77, 0xc1, 0x9a, 0x07, 0xdf, 0x2f, 0x66, 0x6e, 0xa3, 0x6f, 0x78, 0x79,
                0x46, 0x2e, 0x36, 0xfc, 0x76, 0x95, 0x9f, 0x60, 0xcd, 0x29, 0xac, 0x96, 0x34, 0x1c,
                0x4f, 0xff, 0xff, 0xfa,
            ]
        );
    }

    #[test]
    fn public_input_matches_reference_vectors() {
        assert_eq!(
            public_input(b"abc"),
            [
                0x00, 0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d,
                0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff,
                0x61, 0xf2, 0x00, 0x15,
            ]
        );
        assert_eq!(
            public_input(b""),
            [
                0x00, 0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99,
                0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99,
                0x1b, 0x78, 0x52, 0xb8,
            ]
        );
        let x = public_input(b"statement");
        assert!(x < FR_MODULUS, "public input must be reduced");
    }

    #[test]
    fn g1_negate_semantics() {
        // Identity (y = 0) unchanged.
        let mut id = [0u8; 64];
        id[31] = 7;
        assert_eq!(g1_negate(&id), id);

        // y = Fq - 1 negates to y = 1; negation is an involution.
        let mut point = [0u8; 64];
        point[31] = 5; // x arbitrary
        let mut y = FQ_MODULUS;
        y[31] -= 1;
        point[32..].copy_from_slice(&y);
        let neg = g1_negate(&point);
        assert_eq!(&neg[..32], &point[..32]);
        let mut one = [0u8; 32];
        one[31] = 1;
        assert_eq!(&neg[32..], &one[..]);
        assert_eq!(g1_negate(&neg), point);
    }
}
