# SWHID Testing Harness for Minimal Implementation

A simplified testing harness for comparing SWHID implementations on standardized test payloads, adapted for the minimal implementation.

## Purpose

This testing harness allows you to:
- Compare SWHID outputs from different implementations (Rust minimal, Python swh-model, Git)
- Validate that implementations produce identical results for the same inputs
- Test edge cases and complex scenarios consistently across implementations
- Ensure compatibility with the Software Heritage specification

## Structure

```
test_harness/
├── README.md              # This file
├── payloads/              # Test payloads (files, directories)
│   ├── content/           # Content object test files
│   └── directory/         # Directory object test directories
├── harness.py             # Main testing harness
├── config.yaml            # Configuration file
├── requirements.txt       # Python dependencies
└── results/               # Test results output
```

## Test Payloads

The harness includes various test payloads to validate different SWHID types:

### Content Objects
- Empty files (1 byte, 0 bytes)
- Small text files
- Files with Unicode characters
- Files with special characters

### Directory Objects
- Empty directories
- Directories with files
- Directories with subdirectories
- Nested directory structures

## Supported Implementations

1. **Rust Minimal Implementation** (`rust-minimal`)
   - Uses our CLI: `cargo run --bin swhid-cli`
   - Supports content and directory SWHIDs

2. **Python swh-model** (`python-swh-model`)
   - Uses: `python -m swh.model.cli swhid`
   - Reference implementation for comparison

3. **Git Command Line** (`git-cmd`)
   - Uses: `git hash-object`
   - For content SWHID verification

## Usage

### Basic Usage

```bash
# Run all tests with all implementations
python test_harness/harness.py

# Run specific implementation
python test_harness/harness.py --impl rust-minimal

# Run specific test category
python test_harness/harness.py --category content

# Save results to specific file
python test_harness/harness.py --output my_results.json
```

### Configuration

Edit `config.yaml` to configure:
- Test payloads to include/exclude
- Output formats
- Comparison tolerances

### Prerequisites

```bash
# Install Python dependencies
pip install -r test_harness/requirements.txt

# Ensure Rust implementation is built
cargo build --bin swhid-cli

# Ensure Python swh-model is available
python -m swh.model.cli --help
```

## Output

The harness produces:
- Detailed test reports in console
- JSON results files in `results/` directory
- Performance metrics (execution time)
- Compatibility matrices

## Example Output

```
2025-01-27 10:30:15,123 - INFO - Testing category: content
2025-01-27 10:30:15,124 - INFO - Testing payload: empty_file
2025-01-27 10:30:15,234 - INFO - ✓ empty_file: All implementations match
  rust-minimal: swh:1:cnt:0519ecba6ea913e21689ec692e81e9e4973fbf73 (0.110s)
  python-swh-model: swh:1:cnt:0519ecba6ea913e21689ec692e81e9e4973fbf73 (0.089s)
  git-cmd: swh:1:cnt:0519ecba6ea913e21689ec692e81e9e4973fbf73 (0.002s)

============================================================
TEST SUMMARY
============================================================
Total tests: 7
Successful: 7
Failed: 0
Success rate: 100.0%
============================================================
```

## Adding New Test Payloads

1. Add files to `payloads/content/` or `payloads/directory/`
2. Update `config.yaml` with new payload information
3. Optionally add expected SWHID for validation
4. Test with all implementations

## Contributing

To add new test payloads:
1. Add files to appropriate `payloads/` subdirectory
2. Update `config.yaml` with payload metadata
3. Test with all implementations
4. Ensure consistent results across implementations 