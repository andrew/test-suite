# API Documentation

This directory contains the Sphinx API documentation for the SWHID Testing Harness.

## Building the Documentation

### Prerequisites

Install the documentation dependencies:

```bash
pip install -e .[dev]
```

### Build Commands

```bash
# Build HTML documentation
cd docs/api
make html

# Build and open in browser (Linux/Mac)
make html && open _build/html/index.html

# Clean build directory
make clean
```

The generated documentation will be in `docs/api/_build/html/`.

## Documentation Structure

- `conf.py`: Sphinx configuration
- `index.rst`: Main documentation index
- `harness/`: Harness module documentation (auto-generated)
- `implementations/`: Implementation documentation (auto-generated)

## Adding Documentation

Documentation is automatically generated from docstrings in the source code. To improve documentation:

1. Add comprehensive docstrings to classes and functions
2. Use Google or NumPy style docstrings
3. Include type hints for better documentation
4. Rebuild documentation after changes

## Viewing Documentation

After building, open `_build/html/index.html` in your web browser.

