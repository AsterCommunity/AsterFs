# AsterFs

AsterFs provides small, auditable cross-platform filesystem utilities for synchronous and asynchronous Rust. It is AsterCommunity's maintained fork of [`fs4`](https://github.com/al8n/fs4), which in turn continues [`fs2`](https://github.com/danburkert/fs2-rs).

The fork preserves the upstream public API and feature matrix while tightening platform contracts that AsterDrive relies on. In particular, `allocate(len)` must reserve physical storage so later writes within that range do not fail for lack of space; changing only logical EOF is not sufficient.

## Installation

Until the first AsterFs release is published, pin an audited Git revision:

```toml
[dependencies]
aster-fs = { git = "https://github.com/AsterCommunity/AsterFs", rev = "<commit>", features = ["sync"] }
```

Available runtime features:

| Runtime / file type | Feature |
| --- | --- |
| `std::fs::File` | `sync` (default) |
| Tokio | `tokio` |
| async-std | `async-std` |
| Smol | `smol` |
| `fs-err` v2 / v3 | `fs-err2`, `fs-err3` |
| Tokio `fs-err` v2 / v3 | `fs-err2-tokio`, `fs-err3-tokio` |

The extension traits are exported from the crate root:

```rust,no_run
use aster_fs::FileExt;

let file = std::fs::File::create("reserved.bin")?;
file.allocate(128 * 1024 * 1024)?;
# Ok::<(), std::io::Error>(())
```

## Platform contract

- Linux and other supported Unix targets use `rustix` filesystem primitives and preserve native error codes.
- Apple targets use all-or-nothing `F_PREALLOCATE` allocation and validate `fstore_t.fst_bytesalloc` before extending logical EOF.
- Windows uses `FILE_ALLOCATION_INFO` and reports native capacity errors such as `ERROR_DISK_FULL`.
- Platforms without a real physical-preallocation primitive retain the upstream compatibility fallback; callers that require a hard reservation should restrict deployment to a verified backend.

Native runtime tests cover Linux, macOS, and Windows. The macOS CI path mounts a disposable size-limited APFS image to prove that an oversized request returns `ENOSPC` without leaving a partial allocation.

## Compatibility

- Package: `aster-fs`
- Crate: `aster_fs`
- MSRV: Rust 1.75 for the default `sync` feature
- License: MIT OR Apache-2.0

The `async-std` and `smol` features inherit the MSRV of their transitive dependencies, currently Rust 1.85.

## Credits

AsterFs preserves the work and copyright of the `fs4` and `fs2` authors. See [CHANGELOG.md](CHANGELOG.md), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE).
