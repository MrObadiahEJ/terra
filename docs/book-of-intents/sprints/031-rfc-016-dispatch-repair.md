# Sprint 031 — RFC-016 Registry Dispatch Repair

**Date:** 2026-11-12 · **Status:** done, committed (`41fe53d`, `3a6c0a1`)
**Gate:** `cargo check -p terra-registry` passes; `cargo fmt -p terra-registry -- --check` passes

## Trigger

Commit `9bb31f4` wired the RFC-016 security/privacy instructions into the
`#[program]` dispatch block but left `cargo check -p terra-registry` failing with
E0432. The frontend and Pages builds were unaffected, but the on-chain crate was
red and the Book could not honestly claim the security instructions were
registered.

## Root cause

`#[derive(Accounts)]` emits a hidden `__client_accounts_*` module for every
accounts struct. Anchor's `#[program]` macro resolves `Context<…>` types against
the **crate root**, not against the module where the struct is declared.

The security contexts were re-exported from `security::` at the crate root, but
their generated client-account modules were not:

```
error[E0432]: unresolved import `crate`
  note: suggest importing `security::__client_accounts_record_audit_event`
```

## Fix

`terra-core/programs/terra_registry/src/lib.rs` now re-exports the eleven hidden
modules next to the public security contexts:

```rust
#[doc(hidden)]
pub(crate) use security::{
    __client_accounts_appeal_decision, __client_accounts_assign_reviewer,
    // … all 11 security instructions
};
```

`pub(crate)` is required: the generated modules are only crate-public, so a
plain `pub use` fails with E0365. `#[doc(hidden)]` keeps them out of generated
documentation.

The same run applied `rustfmt` to `src/security.rs` and `tests/security.rs`,
which had never been formatted after the RFC-016 implementation (`3a6c0a1`,
formatting only).

## Verification

- `cargo check -p terra-registry` — finished successfully in **33.6 s**
  (warm incremental target; this is the fastest credible Rust gate on the
  current machine).
- `cargo fmt -p terra-registry -- --check` — clean.
- No behavior change: only visibility of generated modules and whitespace.
- Not run: BPF/SBF build, `cargo test-sbf`, full workspace suite. Those remain
  deferred on this machine and are not needed to prove dispatch resolves.

## Remaining gaps

- The RFC-016 security module still has **no BPF integration tests** and is not
  deployed to devnet. A37 in `IMPLEMENTATION_STATUS.md` therefore stays at
  `2 – IMPLEMENTED, NOT FULLY VERIFIED`.
- The `security.rs` unit-test file exists but is a test scaffold; its coverage
  is not counted as on-chain verification.

## Book updates

`IMPLEMENTATION_STATUS.md` A37 evidence updated with the dispatch fix;
`work/CURRENT.md`, `CHANGELOG.md`, `SUMMARY.md` and `BOOK_OF_INTENTS.md`
extended with this sprint.
