//! HRX writer implementation.
//!
//! Writes an [`Archive`](crate::models::Archive) to HRX format.

use crate::models::{Archive, Entry, BOUNDARY_MARKER};
use tracing::{debug, trace};

/// The directory boundary separator (80 `=` characters).
const DIR_BOUNDARY: &str =
    "================================================================================";

/// Write an archive to an HRX-formatted string.
pub fn write(archive: &Archive) -> String {
    debug!(entries = archive.len(), "writing archive to HRX");
    let mut output = String::new();
    write_entries(archive.entries(), &mut output);
    output
}

/// Write an archive to an HRX-formatted string with a trailing newline.
pub fn write_with_newline(archive: &Archive) -> String {
    let mut s = write(archive);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn write_entries(entries: &[Entry], output: &mut String) {
    for (i, entry) in entries.iter().enumerate() {
        match entry {
            Entry::File(file) => {
                trace!(path = %file.path, "writing file entry");
                // Write boundary
                output.push_str(BOUNDARY_MARKER);
                output.push(' ');
                output.push_str(&file.path);
                output.push('\n');

                // Write contents
                if !file.contents.is_empty() {
                    output.push_str(&file.contents);
                    // Ensure contents end with newline before next boundary
                    if !file.contents.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
            Entry::Dir(dir) => {
                trace!(path = %dir.path, children = dir.children.len(), "writing directory entry");
                // Write directory boundary marker
                output.push_str(BOUNDARY_MARKER);
                output.push('\n');
                output.push_str(DIR_BOUNDARY);
                output.push('\n');

                // Write children
                write_entries(&dir.children, output);
            }
        }

        // Add blank line separator between entries (but not after the last one)
        if i < entries.len() - 1 {
            output.push('\n');
        }
    }
}

/// Write a single file as a minimal HRX archive.
pub fn write_file(path: &str, contents: &str) -> String {
    let mut output = String::new();
    output.push_str(BOUNDARY_MARKER);
    output.push(' ');
    output.push_str(path);
    output.push('\n');
    output.push_str(contents);
    if !contents.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Archive;

    #[test]
    fn test_write_simple() {
        let mut archive = Archive::new();
        archive.add_file("input.scss", "a { color: red }");
        archive.add_file("output.css", "a {\n  color: red;\n}");

        let hrx = write(&archive);
        assert!(hrx.contains("<===> input.scss"));
        assert!(hrx.contains("<===> output.css"));
        assert!(hrx.contains("a { color: red }"));
    }

    #[test]
    fn test_roundtrip() {
        let original = "<===> input.scss\na { color: red }\n\n<===> output.css\na {\n  color: red;\n}\n";
        let archive = crate::parser::parse(original).unwrap();
        let written = write(&archive);

        // Parse again to verify
        let archive2 = crate::parser::parse(&written).unwrap();
        assert_eq!(archive.len(), archive2.len());
    }
}
