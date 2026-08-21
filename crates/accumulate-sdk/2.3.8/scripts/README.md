# Development Scripts

Automation scripts for development, testing, and quality assurance of the Accumulate Rust SDK.

## Available Scripts

### PowerShell Scripts (`.ps1`)

#### `run_parity_gate.ps1`
**Purpose**: Comprehensive parity validation pipeline ensuring byte-for-byte compatibility with TypeScript SDK.

```powershell
# Run complete parity validation
.\scripts\run_parity_gate.ps1

# Custom configuration
.\scripts\run_parity_gate.ps1 -FuzzCount 2000 -CoverageThreshold 80

# Skip specific components
.\scripts\run_parity_gate.ps1 -SkipFormatting -SkipLinting
```

**Pipeline Steps**:
1. **TypeScript Fixture Generation**: Creates deterministic test vectors
2. **Code Quality**: Formatting (`cargo fmt`) and linting (`cargo clippy`)
3. **Quality Gates**: Scans for prohibited patterns (TODOs, stubs)
4. **Core Tests**: Canonical JSON, cryptography, codec validation
5. **Parity Tests**: TypeScript fuzzing and type matrix roundtrips
6. **Coverage Analysis**: Ensures adequate test coverage

**Parameters**:
- `-FuzzCount <int>` - Number of fuzz vectors to generate (default: 1000, max: 2000)
- `-CoverageThreshold <int>` - Minimum coverage percentage (default: 70)
- `-SkipFormatting` - Skip code formatting step
- `-SkipLinting` - Skip clippy linting step
- `-SkipTests` - Skip test execution (formatting and linting only)
- `-Verbose` - Enable detailed output

#### `coverage_gate.ps1`
**Purpose**: Code coverage analysis with configurable thresholds.

```powershell
# Run coverage analysis
.\scripts\coverage_gate.ps1

# Custom thresholds
.\scripts\coverage_gate.ps1 -OverallThreshold 75 -CriticalThreshold 90

# Generate HTML report
.\scripts\coverage_gate.ps1 -GenerateHtml
```

**Features**:
- Overall coverage threshold validation
- Critical module coverage analysis
- HTML report generation
- Coverage trend tracking

#### `package_check.ps1`
**Purpose**: Package readiness validation for publishing.

```powershell
# Validate package readiness
.\scripts\package_check.ps1

# Include documentation check
.\scripts\package_check.ps1 -CheckDocs

# Dry-run publish test
.\scripts\package_check.ps1 -DryRun
```

**Validations**:
- Cargo.toml metadata completeness
- Documentation build verification
- License file presence
- README.md quality check
- Dry-run publish test

### Shell Scripts (`.sh`)

#### `quality_gate.sh`
**Purpose**: Cross-platform quality validation (Linux/macOS).

```bash
# Run quality checks
./scripts/quality_gate.sh

# With custom parameters
./scripts/quality_gate.sh --coverage-threshold 80 --format --lint
```

#### `benchmark.sh`
**Purpose**: Performance benchmarking and regression testing.

```bash
# Run benchmarks
./scripts/benchmark.sh

# Compare with baseline
./scripts/benchmark.sh --compare baseline.json

# Generate benchmark report
./scripts/benchmark.sh --report
```

## Script Categories

### Quality Assurance
- **Parity Gate**: Cross-language compatibility validation
- **Coverage Gate**: Code coverage analysis and enforcement
- **Quality Gate**: Code quality metrics and standards
- **Package Check**: Release readiness validation

### Development Workflow
- **Format & Lint**: Code formatting and linting automation
- **Test Execution**: Automated test suite execution
- **Documentation**: API documentation generation
- **Benchmark**: Performance testing and analysis

### CI/CD Integration
- **Automated Validation**: Scripts designed for CI/CD pipelines
- **Parallel Execution**: Support for parallel test execution
- **Report Generation**: Machine-readable output formats
- **Error Handling**: Robust error handling and reporting

## Usage Patterns

### Local Development
```bash
# Quick quality check before commit
./scripts/run_parity_gate.ps1 -SkipTests

# Full validation before push
./scripts/run_parity_gate.ps1

# Coverage analysis
./scripts/coverage_gate.ps1 -GenerateHtml
```

### CI/CD Pipeline
```bash
# CI validation (faster execution)
./scripts/run_parity_gate.ps1 -FuzzCount 200 -CoverageThreshold 70

# Release validation
./scripts/package_check.ps1 -CheckDocs -DryRun

# Performance regression testing
./scripts/benchmark.sh --compare baseline.json
```

### Pre-Release Checklist
```bash
# 1. Code quality validation
./scripts/run_parity_gate.ps1 -FuzzCount 2000

# 2. Coverage analysis
./scripts/coverage_gate.ps1 -OverallThreshold 75

# 3. Package readiness
./scripts/package_check.ps1 -CheckDocs -DryRun

# 4. Performance validation
./scripts/benchmark.sh --report
```

## Environment Requirements

### PowerShell Scripts
- **PowerShell 5.0+** (Windows) or **PowerShell Core 6.0+** (cross-platform)
- **Rust toolchain** with `cargo`, `rustfmt`, `clippy`
- **Node.js** (for TypeScript fixture generation)
- **Git** (for change detection and repository operations)

### Shell Scripts
- **Bash 4.0+** or compatible shell
- **Standard Unix utilities** (`grep`, `sed`, `awk`, `find`)
- **Rust toolchain** and development dependencies

### Optional Dependencies
- **cargo-llvm-cov** - For coverage analysis
- **cargo-expand** - For macro expansion debugging
- **cargo-audit** - For security auditing
- **cargo-outdated** - For dependency management

## Configuration

### Environment Variables
```bash
# Coverage configuration
export COVERAGE_THRESHOLD=70
export CRITICAL_COVERAGE_THRESHOLD=85

# Test configuration
export RUST_LOG=info
export FUZZ_COUNT=1000

# Path configuration
export ACCUMULATE_REPO="/path/to/accumulate"
export TS_SDK_ROOT="/path/to/typescript-sdk"
```

### Script Parameters
Most scripts support common parameters:
- `--verbose` / `-v` - Enable detailed output
- `--help` / `-h` - Display usage information
- `--dry-run` - Show what would be done without executing
- `--config <file>` - Use custom configuration file

## Output and Reporting

### Exit Codes
- **0** - Success
- **1** - General failure
- **2** - Configuration error
- **3** - Test failure
- **4** - Coverage threshold not met
- **5** - Quality gate failure

### Report Formats
- **Console Output**: Human-readable progress and results
- **JSON Reports**: Machine-readable output for CI/CD integration
- **HTML Reports**: Detailed visual reports (coverage, benchmarks)
- **JUnit XML**: Test result format for CI/CD systems

### Example Output
```
ACCUMULATE RUST SDK PARITY GATE
===================================

Entering unified directory...
Generating TypeScript fixtures...
  Generated 1000 random test vectors
Formatting Rust code...
  Code formatting complete
Running Clippy linter...
  Linting passed with no warnings
 Running quality gates...
  No prohibited patterns found
Running core functionality tests...
  Core functionality tests passed
Running parity and roundtrip tests...
  All parity and roundtrip tests passed
Analyzing code coverage...
  Coverage threshold of 70% achieved

All parity gates passed successfully!
```

## Troubleshooting

### Common Issues

1. **PowerShell Execution Policy**:
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

2. **Missing Dependencies**:
   ```bash
   cargo install cargo-llvm-cov cargo-expand cargo-audit
   ```

3. **Node.js/TypeScript Issues**:
   ```bash
   cd tooling/ts-fixture-exporter/
   npm install
   ```

### Debug Mode
```bash
# Enable debug output
RUST_LOG=debug ./scripts/run_parity_gate.ps1 -Verbose

# Step-by-step execution
./scripts/run_parity_gate.ps1 -StepByStep
```

## Adding New Scripts

### Script Template
```powershell
#!/usr/bin/env pwsh
param(
    [switch]$Verbose,
    [switch]$Help
)

# Script metadata
$ScriptName = "new_script"
$ScriptVersion = "1.0"
$ScriptDescription = "Description of what this script does"

# Help function
function Show-Help {
    Write-Host "$ScriptDescription"
    Write-Host ""
    Write-Host "Usage: $ScriptName [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Verbose    Enable detailed output"
    Write-Host "  -Help       Show this help message"
}

# Main script logic
if ($Help) {
    Show-Help
    exit 0
}

# Implementation here...
```

### Best Practices
1. **Error Handling**: Implement robust error handling and cleanup
2. **Logging**: Provide clear progress indicators and error messages
3. **Parameterization**: Make scripts configurable via parameters
4. **Documentation**: Include inline documentation and help functions
5. **Cross-Platform**: Consider cross-platform compatibility where possible
6. **Testing**: Test scripts in different environments and scenarios