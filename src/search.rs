use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Print a tree view of the directory structure
pub fn print_tree(dir: &str) {
    println!("Project Tree for '{}':", dir);
    let mut builder = WalkBuilder::new(dir);
    builder.hidden(false).ignore(true).git_ignore(true);
    builder.filter_entry(|e| e.file_name().to_string_lossy() != ".git");
    let walker = builder.build();

    let root_depth = Path::new(dir).components().count();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                let depth = path.components().count();
                if depth >= root_depth {
                    let indent_level = depth - root_depth;
                    if indent_level == 0 {
                        println!("{}", path.display());
                    } else {
                        let indent = "  ".repeat(indent_level - 1);
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        println!("{}|-- {}", indent, file_name);
                    }
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }
}

/// Search for a regex pattern across all files
pub fn find_pattern(dir: &str, pattern: &str, excludes: Option<&Vec<String>>, exts: Option<&Vec<String>>) {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Invalid regex pattern: {}", e);
            return;
        }
    };

    println!("Searching for '{}' in '{}'...", pattern, dir);

    let mut builder = WalkBuilder::new(dir);
    builder.hidden(false).ignore(true).git_ignore(true);

    if let Some(exc_list) = excludes {
        let exc_list = exc_list.clone();
        builder.filter_entry(move |e| {
            let file_name = e.file_name().to_string_lossy();
            if file_name == ".git" {
                return false;
            }
            !exc_list.iter().any(|exc| file_name == *exc)
        });
    } else {
        builder.filter_entry(|e| e.file_name().to_string_lossy() != ".git");
    }

    let walker = builder.build();
    let paths: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .map(|e| e.into_path())
        .filter(|p| {
            if let Some(ext_filter) = exts {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    let ext_with_dot = format!(".{}", ext);
                    ext_filter.contains(&ext_with_dot) || ext_filter.contains(&ext.to_string())
                } else {
                    false
                }
            } else {
                true
            }
        })
        .collect();

    let matches: Vec<(PathBuf, Vec<(usize, String)>)> = paths
        .into_par_iter()
        .filter_map(|path| {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => String::from_utf8_lossy(&fs::read(&path).unwrap_or_default()).into_owned(),
            };

            let mut file_matches = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    file_matches.push((i + 1, line.trim().to_string()));
                }
            }

            if file_matches.is_empty() {
                None
            } else {
                Some((path, file_matches))
            }
        })
        .collect();

    if matches.is_empty() {
        println!("No matches found.");
        return;
    }

    let mut total_matches = 0;
    for (path, file_matches) in matches {
        println!("\n{}", path.display());
        for (line_num, line_content) in file_matches {
            println!("  {}: {}", line_num, line_content);
            total_matches += 1;
        }
    }
    println!("\nFound {} matches.", total_matches);
}

/// Detect duplicate code blocks (5+ lines) across the codebase
pub fn detect_duplicates(dir: &str, excludes: Option<&Vec<String>>, exts: Option<&Vec<String>>) {
    println!("Detecting duplicate code blocks (5+ lines) in '{}'...", dir);

    let mut builder = WalkBuilder::new(dir);
    builder.hidden(false).ignore(true).git_ignore(true);

    if let Some(exc_list) = excludes {
        let exc_list = exc_list.clone();
        builder.filter_entry(move |e| {
            let file_name = e.file_name().to_string_lossy();
            if file_name == ".git" {
                return false;
            }
            !exc_list.iter().any(|exc| file_name == *exc)
        });
    } else {
        builder.filter_entry(|e| e.file_name().to_string_lossy() != ".git");
    }

    let walker = builder.build();
    let paths: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .map(|e| e.into_path())
        .filter(|p| {
            if let Some(ext_filter) = exts {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    let ext_with_dot = format!(".{}", ext);
                    ext_filter.contains(&ext_with_dot) || ext_filter.contains(&ext.to_string())
                } else {
                    false
                }
            } else {
                true
            }
        })
        .collect();

    // Collect all blocks
    let window_size = 5;
    
    // We cannot easily parallelize insertion into a single HashMap without a mutex.
    // Instead, map paths in parallel to their blocks, then reduce sequentially.
    let file_blocks: Vec<Vec<(String, PathBuf, usize)>> = paths
        .into_par_iter()
        .filter_map(|path| {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => return None, // Skip non-utf8 for duplication detection
            };

            // Filter out empty lines or very short lines to reduce noise
            let lines: Vec<(usize, String)> = content
                .lines()
                .enumerate()
                .filter(|(_, l)| l.trim().len() > 3)
                .map(|(i, l)| (i + 1, l.trim().to_string()))
                .collect();

            if lines.len() < window_size {
                return None;
            }

            let mut blocks = Vec::new();
            for i in 0..=(lines.len() - window_size) {
                let mut block_content = String::new();
                for j in 0..window_size {
                    block_content.push_str(&lines[i + j].1);
                    block_content.push('\n');
                }
                blocks.push((block_content, path.clone(), lines[i].0));
            }
            Some(blocks)
        })
        .collect();

    let mut block_map: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();
    for blocks in file_blocks {
        for (content, path, line_num) in blocks {
            block_map.entry(content).or_default().push((path, line_num));
        }
    }

    let mut duplicates: Vec<_> = block_map
        .into_iter()
        .filter(|(_, locs)| locs.len() > 1)
        .collect();

    // Sort by number of occurrences descending
    duplicates.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    if duplicates.is_empty() {
        println!("No significant duplicate code blocks found.");
        return;
    }

    let display_limit = 10;
    println!("\nTop {} duplicated blocks:\n", display_limit.min(duplicates.len()));

    for (i, (block, locs)) in duplicates.iter().take(display_limit).enumerate() {
        println!("--- Duplicate Block {} ({} occurrences) ---", i + 1, locs.len());
        for (path, line_num) in locs.iter().take(5) {
            println!("  Found in {} at line {}", path.display(), line_num);
        }
        if locs.len() > 5 {
            println!("  ... and {} more locations", locs.len() - 5);
        }
        println!("Preview:");
        let preview: Vec<&str> = block.lines().take(window_size).collect();
        for line in preview {
            println!("> {}", line);
        }
        println!();
    }
}
