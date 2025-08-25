//! Minimal reference implementation of Software Heritage Identifier (SWHID) computation
//! 
//! This library provides the core SWHID functionality without extra features like
//! archive processing, complex Git operations, or performance optimizations.
//! 
//! ## Core SWHID Types
//! 
//! - **Content SWHID**: Compute SWHIDs for individual files
//! - **Directory SWHID**: Compute SWHIDs for directory trees
//! - **Basic SWHID**: Core SWHID format: `swh:1:obj_type:hash`
//! 
//! ## Usage
//! 
//! ```rust
//! use swhid_core::{SwhidComputer, Swhid};
//! 
//! let computer = SwhidComputer::new();
//! 
//! // Compute SWHID for a file
//! let file_swhid = computer.compute_file_swhid("/path/to/file.txt")?;
//! 
//! // Compute SWHID for a directory
//! let dir_swhid = computer.compute_directory_swhid("/path/to/directory")?;
//! ```

pub mod swhid;
pub mod hash;
pub mod content;
pub mod directory;
pub mod error;
pub mod computer;

pub use swhid::{Swhid, ObjectType};
pub use error::SwhidError;
pub use computer::SwhidComputer;
pub use content::Content;
pub use directory::Directory; 