//! HRX (Human Readable Archive) parser and writer.
//!
//! HRX is a plain-text archive format that represents a virtual filesystem.
//! See the sass-spec documentation and the original Google HRX spec for details.
//!
//! # Format
//!
//! ```hrx
//! <===> path/to/file.txt
//! file contents here
//! can span multiple lines
//!
//! <===> another/file.txt
//! more contents
//! ```
//!
//! Directory boundaries are expressed as HRX comments with 80 `=` characters:
//!
//! ```hrx
//! <===> dir1/file.txt
//! content
//!
//! <===>
//! ================================================================================
//! <===> dir2/file.txt
//! content
//! ```

pub mod error;
pub mod models;
pub mod parser;
pub mod writer;
pub mod test_util;

pub use error::HrxError;
pub use models::{Archive, Entry, FileEntry, DirEntry};
pub use parser::parse;
pub use writer::write;
