# AsterFs

AsterFs is AsterCommunity's maintained fork of `fs4`. It provides small, auditable filesystem primitives for sync and async Rust without importing product-domain behavior.

## Working contract

- Inspect the current branch, HEAD, worktree, applicable OS backend, and adjacent tests before editing.
- Preserve upstream API and feature compatibility unless a breaking change is explicitly approved.
- Keep platform syscalls in `src/unix/` or `src/windows/`; sync and async traits must share the same platform semantics.
- `allocate(len)` means physical space is reserved so later writes within `len` do not fail for lack of disk space. Logical `set_len` alone does not satisfy this contract.
- Native error codes must be preserved. Capacity exhaustion must not be converted into success or an unrelated error.
- Changes to platform behavior require native runtime evidence on every affected OS. Cross-compilation proves cfg and linking only.
- Keep tests bounded and clean up temporary volumes, images, files, and mounts even on failure.
- Run formatting, focused tests, the applicable feature matrix, cross-target checks, and `git diff --check` before delivery.

## Ownership

- Upstream history and copyright remain intact.
- AsterCommunity-specific fixes are recorded in the current changelog and linked to an issue.
- Do not publish, tag, force-push, delete branches, or rewrite upstream history without explicit authorization.
