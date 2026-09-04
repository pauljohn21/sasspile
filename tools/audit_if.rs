// Audit bare `if` statements in src/ — excludes `if let`, comments, `else if`
use std::fs;
use std::path::Path;

fn audit_file(path: &Path) -> Vec<(usize, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut results = vec![];
    let mut in_block_comment = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track block comments
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            continue;
        }

        // Skip line comments
        if trimmed.starts_with("//") || trimmed.starts_with("#![") {
            continue;
        }

        // Match lines starting with `if ` but NOT `if let`
        // Also exclude `else if` (those are part of if-else chains, handled separately)
        if trimmed.starts_with("if ") && !trimmed.starts_with("if let") {
            results.push((i + 1, trimmed.to_string()));
        }
    }

    results
}

fn walk_dir(dir: &Path, root: &Path, results: &mut Vec<(String, Vec<(usize, String)>)>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut subdirs = vec![];
    let mut files = vec![];

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }

    files.sort();
    for file in &files {
        let hits = audit_file(file);
        if !hits.is_empty() {
            let rel = file.strip_prefix(root).unwrap_or(file).to_string_lossy().to_string();
            results.push((rel, hits));
        }
    }

    subdirs.sort();
    for subdir in &subdirs {
        walk_dir(subdir, root, results);
    }
}

fn main() {
    let src_dir = Path::new("src");
    let mut results = vec![];
    walk_dir(src_dir, Path::new("."), &mut results);

    // Sort by count descending
    results.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let total: usize = results.iter().map(|(_, hits)| hits.len()).sum();
    println!("Total bare `if` statements: {}\n", total);

    for (file, hits) in &results {
        println!("=== {} ({} hits) ===", file, hits.len());
        for (lineno, line) in hits {
            println!("  L{}: {}", lineno, line);
        }
        println!();
    }
}
