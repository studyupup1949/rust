# accessible

`accessible` offers best-effort, non-trapping memory readability probes for Rust FFI boundaries. It
is intended as a diagnostic aid so C-callable shims can flag obviously bad pointers before diving
into more expensive work.

## Highlights

- Uses native, non-trapping read APIs on each supported platform.
- Checks only the first and last byte of a region to keep the overhead tiny.
- Treats zero-length regions and zero-sized types as readable to fit idiomatic Rust slices.
- Surface-level guardrails only—results are hints, not safety guarantees.

## Usage

Add the crate and call the helpers from an `extern "C"` entry point (status codes omitted for
brevity):

```rust
use accessible::{probe_readable_range, CheckError};

#[no_mangle]
pub unsafe extern "C" fn consume_buffer(ptr: *const u8, len: usize) {
    if let Err(CheckError::Unreadable(err)) = probe_readable_range(ptr, len) {
        eprintln!("input buffer was unreadable: {err}");
        return;
    }

    // Safe-ish to continue: the first and last byte were readable at the time of the probe.
}
```

Additional helpers exist for probing single values (`probe_readable_value`) and slices
(`probe_readable_slice`).

## Platform support

| Platform | Implementation |
| -------- | -------------- |
| Linux    | `process_vm_readv` with `/proc/self/mem` fallback |
| macOS    | `mach_vm_read_overwrite` |
| Windows  | `ReadProcessMemory` |
| Others   | Returns `CheckError::Unsupported` |

## Limitations

- Probes are race-prone: another thread can unmap memory between the check and later use.
- Only minimal bytes are read. If you need deeper verification, call `probe_readable` repeatedly.
- Fallbacks rely on OS facilities; very old kernels without `/proc/self/mem` will be reported as
  unreadable.

## Development

```bash
cargo fmt
cargo test
```

CI is configured to run the test suite on Linux, macOS, and Windows so regressions surface quickly
on the major platforms.
