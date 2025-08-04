# SWHID Rust Library

A Rust implementation of Software Heritage Identifier (SWHID) computation, extracted from the Python `swh-model` package.

## Overview

SWHIDs are persistent identifiers for software artifacts that follow the format:
```
swh:1:<object_type>:<40_character_hex_hash>
```

Where:
- `swh` is the namespace (always "swh")
- `1` is the scheme version (always 1)
- `<object_type>` is one of: `cnt`, `dir`, `rev`, `rel`, `snp`
- `<40_character_hex_hash>` is the SHA1 hash of the object

## Features

- **Content SWHID**: Compute SWHIDs for individual files
- **Directory SWHID**: Compute SWHIDs for directory trees
- **Git-compatible**: Uses Git's object format for hashing
- **CLI tool**: Command-line interface for SWHID computation
- **Library API**: Rust library for integration into other projects

## Installation

### From Source

```bash
git clone <repository-url>
cd swhid-rs
cargo build --release
```

### Using Cargo

Add to your `Cargo.toml`:
```toml
[dependencies]
swhid = "0.1.0"
```

## Usage

### Command Line Interface

```bash
# Compute SWHID for a file
./target/release/swhid-cli /path/to/file.txt

# Compute SWHID for a directory
./target/release/swhid-cli /path/to/directory

# Compute SWHID from stdin
echo "Hello, World!" | ./target/release/swhid-cli -
```

### Library API

```rust
use swhid::{SwhidComputer, Swhid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let computer = SwhidComputer::new();
    
    // Compute SWHID for a file
    let file_swhid = computer.compute_file_swhid("/path/to/file.txt")?;
    println!("File SWHID: {}", file_swhid);
    
    // Compute SWHID for a directory
    let dir_swhid = computer.compute_directory_swhid("/path/to/directory")?;
    println!("Directory SWHID: {}", dir_swhid);
    
    // Auto-detect and compute SWHID
    let swhid = computer.compute_swhid("/path/to/object")?;
    println!("SWHID: {}", swhid);
    
    Ok(())
}
```

## API Reference

### SwhidComputer

The main entry point for SWHID computation.

```rust
pub struct SwhidComputer {
    exclude_patterns: Vec<String>,
    follow_symlinks: bool,
}
```

#### Methods

- `new()` - Create a new SWHID computer with default settings
- `with_exclude_patterns(patterns)` - Set patterns to exclude from directory processing
- `with_follow_symlinks(follow)` - Configure symlink following behavior
- `compute_file_swhid(path)` - Compute SWHID for a file
- `compute_directory_swhid(path)` - Compute SWHID for a directory
- `compute_swhid(path)` - Auto-detect object type and compute SWHID

### Swhid

Represents a Software Heritage Identifier.

```rust
pub struct Swhid {
    namespace: String,
    scheme_version: u32,
    object_type: ObjectType,
    object_id: [u8; 20],
}
```

#### Methods

- `new(object_type, object_id)` - Create a new SWHID
- `from_string(s)` - Parse SWHID from string
- `namespace()` - Get the namespace
- `scheme_version()` - Get the scheme version
- `object_type()` - Get the object type
- `object_id()` - Get the object ID hash

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running Tests with Output

```bash
cargo test -- --nocapture
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## Acknowledgments

This implementation is based on the Python `swh-model` package from the Software Heritage project. 