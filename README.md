# SWHID Core - Minimal Reference Implementation

A minimal, clean reference implementation of Software Heritage Identifier (SWHID) computation in Rust.

## Overview

This library provides the **core SWHID functionality** without extra features like archive processing, complex Git operations, or performance optimizations. It serves as a clean reference implementation that can be used as a dependency for the full-featured `swhid-rs` library.

## Core SWHID Types

SWHIDs are persistent identifiers for software artifacts that follow the format:
```
swh:1:<object_type>:<40_character_hex_hash>
```

Where:
- `swh` is the namespace (always "swh")
- `1` is the scheme version (always 1)
- `<object_type>` is one of: `cnt`, `dir`
- `<40_character_hex_hash>` is the SHA1 hash of the object

## Features

- **Content SWHID**: Compute SWHIDs for individual files
- **Directory SWHID**: Compute SWHIDs for directory trees
- **Git-compatible**: Uses Git's object format for hashing
- **Minimal Dependencies**: Only essential crates (sha1-checked, hex)
- **Reference Implementation**: Clean, readable code for SWHID specification

## What's NOT Included

- Archive processing (tar, zip, etc.)
- Git repository operations (snapshot, revision, release)
- Extended SWHID types (Origin, Raw Extrinsic Metadata)
- Qualified SWHIDs with anchors, paths, and line ranges
- Performance optimizations (caching, statistics)
- Command-line interface
- Complex recursive traversal

## Installation

### From Source

```bash
git clone <repository-url>
cd swhid-rs
git checkout minimal-reference-impl
cargo build
```

### Using Cargo

Add to your `Cargo.toml`:
```toml
[dependencies]
swhid-core = "0.1.0"
```

## Usage

### Basic SWHID Computation

```rust
use swhid_core::{SwhidComputer, Swhid};

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

### Direct Object Usage

```rust
use swhid_core::{Content, Directory, Swhid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create content from data
    let content = Content::from_data(b"Hello, World!".to_vec());
    let content_swhid = content.swhid();
    println!("Content SWHID: {}", content_swhid);
    
    // Create directory from disk
    let mut dir = Directory::from_disk("/path/to/directory", &[], true)?;
    let dir_swhid = dir.swhid();
    println!("Directory SWHID: {}", dir_swhid);
    
    Ok(())
}
```

## Architecture

```
src/
├── lib.rs          # Core library exports
├── swhid.rs        # SWHID structs and formatting
├── hash.rs         # Basic hash computation
├── content.rs      # Content object handling
├── directory.rs    # Directory object handling
├── error.rs        # Core error types
└── computer.rs     # Minimal SWHIDComputer
```

## Testing

Run the core conformance tests:

```bash
cargo test --test core_tests
```

## Dependencies

- **sha1-checked**: Collision-resistant SHA1 hashing
- **hex**: Hexadecimal encoding/decoding

## Use Cases

- **Reference Implementation**: Clean code for SWHID specification
- **Core Library**: Foundation for full-featured SWHID implementations
- **Testing**: Base implementation for conformance testing
- **Learning**: Simple, readable SWHID computation code

## Relationship to Full Implementation

This minimal implementation serves as the **core foundation** for the full `swhid-rs` library:

```
swhid-core (this crate)
    ↓
swhid-rs (full implementation)
    ├── Archive processing
    ├── Git operations
    ├── Extended SWHID types
    ├── Performance optimizations
    └── CLI interface
```

The full implementation will depend on this core crate and add the additional features on top.

## License

MIT License - see LICENSE file for details. 