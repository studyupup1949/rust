# cspell:ignore DEVNULL, MSRV, pathlib, tomllib

@_default:
    just --list

# Minimum supported Rust version (read directly from Cargo.toml)
MSRV := `python3 - <<'PY'
import pathlib, tomllib
data = tomllib.loads(pathlib.Path('Cargo.toml').read_text())
print(data['package']['rust-version'])
PY`

# All toolchains to be validated (every minor channel from MSRV to current stable)
SUPPORTED_TOOLCHAINS := `python3 - <<'PY'
import pathlib, tomllib, subprocess, re
data = tomllib.loads(pathlib.Path('Cargo.toml').read_text())
msrv = data['package']['rust-version']
major, minor, *_ = (msrv.split('.') + ['0', '0'])[:3]
major = int(major)
minor = int(minor)
subprocess.run(
    ['rustup', 'toolchain', 'install', 'stable', '--profile', 'minimal'],
    check=True,
    stdout=subprocess.DEVNULL,
)
stable_version = subprocess.check_output(['rustup', 'run', 'stable', 'rustc', '--version'], text=True)
match = re.search(r"rustc (\d+)\.(\d+)\.(\d+)", stable_version)
stable_minor = int(match.group(2))
toolchains = [f"{major}.{m}" for m in range(minor, stable_minor + 1)]
print(' '.join(toolchains))
PY`

# Check for errors in all supported targets and configurations
check: check-aarch64-linux-i2c check-armv8-i2c check-armv8-i2c-async check-examples

# Run the complete check matrix using every supported toolchain
check-all-toolchains: install-supported-toolchains
    set -euo pipefail
    for toolchain in {{SUPPORTED_TOOLCHAINS}}; do \
        echo "==> Checking with $toolchain"; \
        rustup target add aarch64-unknown-linux-gnu thumbv8m.main-none-eabihf --toolchain "$toolchain" >/dev/null; \
        cargo +"$toolchain" check --target aarch64-unknown-linux-gnu --features "i2c std"; \
        cargo +"$toolchain" check --target thumbv8m.main-none-eabihf -F "i2c"; \
        cargo +"$toolchain" check --target thumbv8m.main-none-eabihf -F "async i2c"; \
        cargo +"$toolchain" check --target aarch64-unknown-linux-gnu --features "i2c std" --example raspberry-pi-i2c; \
        cargo +"$toolchain" check --target thumbv8m.main-none-eabihf --features "async defmt i2c" --example rp2350-i2c-async; \
    done

# Check for errors targeting AARCH64 running GNU/Linux with I²C feature enabled
check-aarch64-linux-i2c: install-toolchain
    cargo check --target aarch64-unknown-linux-gnu --features "i2c std"

# Check for errors targeting ARMv8-M with I²C feature enabled
check-armv8-i2c: install-toolchain
    cargo check --target thumbv8m.main-none-eabihf -F "i2c"

# Check for errors targeting ARMv8-M with async and I²C features enabled
check-armv8-i2c-async: install-toolchain
    cargo check --target thumbv8m.main-none-eabihf -F "async i2c"

# Check for errors in all examples
check-examples: check-example-raspberry-pi-i2c check-example-rp2350-i2c-async

# Check for errors in the Raspberry Pi I²C example
check-example-raspberry-pi-i2c: install-toolchain
    cargo check --target aarch64-unknown-linux-gnu --features "i2c std" --example raspberry-pi-i2c

# Check for errors in the Raspberry Pi RP2350 I²C async example
check-example-rp2350-i2c-async: install-toolchain
    cargo check --target thumbv8m.main-none-eabihf --features "async defmt i2c" --example rp2350-i2c-async

# Install rust and cargo tools for cross compiling to the target platforms
install-toolchain:
    rustup target add aarch64-unknown-linux-gnu thumbv8m.main-none-eabihf

# Ensure every supported toolchain (including the MSRV) is present locally
install-supported-toolchains:
    set -euo pipefail
    for toolchain in {{SUPPORTED_TOOLCHAINS}}; do \
        rustup toolchain install "$toolchain" --profile minimal >/dev/null; \
    done

# Ensure the MSRV toolchain is present locally
install-msrv-toolchain:
    rustup toolchain install {{MSRV}} --profile minimal >/dev/null

# Run the Raspberry Pi I²C example remotely via SSH
@_run-example-rpi-i2c-ssh host:
    echo NOT YET IMPLEMENTED 1>&2
    exit 1

# Run the RP2350 I²C async example on hardware
@_run-example-rp2350-i2c-async:
    echo NOT YET IMPLEMENTED 1>&2
    exit 1

# Run all unit tests in every supported configuration
test: test-i2c test-i2c-async

# Run the full test suite across every supported toolchain
test-all-toolchains: install-supported-toolchains
    set -euo pipefail
    for toolchain in {{SUPPORTED_TOOLCHAINS}}; do \
        echo "==> Testing with $toolchain"; \
        cargo +"$toolchain" test --features "i2c"; \
        cargo +"$toolchain" test --features "i2c async"; \
    done

# Run unit tests with the synchronous I²C feature set
test-i2c:
    cargo test --features "i2c"

# Run unit tests with both I²C and async features enabled
test-i2c-async:
    cargo test --features "i2c async"

# Run the unit tests on the MSRV toolchain
test-msrv: install-msrv-toolchain
    cargo +{{MSRV}} test --features "i2c"
    cargo +{{MSRV}} test --features "i2c async"

# Run the full check matrix on the MSRV toolchain
check-msrv: install-toolchain install-msrv-toolchain
    cargo +{{MSRV}} check --target aarch64-unknown-linux-gnu --features "i2c std"
    cargo +{{MSRV}} check --target thumbv8m.main-none-eabihf -F "i2c"
    cargo +{{MSRV}} check --target thumbv8m.main-none-eabihf -F "async i2c"
    cargo +{{MSRV}} check --target aarch64-unknown-linux-gnu --features "i2c std" --example raspberry-pi-i2c
    cargo +{{MSRV}} check --target thumbv8m.main-none-eabihf --features "async defmt i2c" --example rp2350-i2c-async