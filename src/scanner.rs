use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

pub struct FileStats {
    pub path: PathBuf,
    pub ext: String,
    pub lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
}

pub fn scan_directory(dir: &str, excludes: Option<&Vec<String>>, exts: Option<&Vec<String>>) -> Vec<FileStats> {
    let mut builder = WalkBuilder::new(dir);
    builder.hidden(false).ignore(true).git_ignore(true);

    if let Some(exc_list) = excludes {
        let exc_list = exc_list.clone();
        builder.filter_entry(move |e| {
            let file_name = e.file_name().to_string_lossy();
            !exc_list.iter().any(|exc| file_name == *exc)
        });
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

    paths
        .into_par_iter()
        .filter_map(|path| analyze_file(&path))
        .collect()
}

fn analyze_file(path: &Path) -> Option<FileStats> {
    // Read the file as raw bytes to handle potentially non-utf8 cleanly, 
    // or just read_to_string and skip if invalid. We'll stick to string for simplicity 
    // but a production tool would read lossy strings or bytes.
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => String::from_utf8_lossy(&fs::read(path).ok()?).into_owned(),
    };
    
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| "".to_string());

    let mut lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut in_block_comment = false;

    let is_c_style = matches!(ext.as_str(), ".c" | ".cpp" | ".h" | ".hpp" | ".js" | ".ts" | ".jsx" | ".tsx" | ".java" | ".rs" | ".go" | ".cs" | ".swift" | ".kt" | ".php");
    let is_py_style = matches!(ext.as_str(), ".py" | ".sh" | ".rb" | ".pl" | ".yml" | ".yaml");
    let is_html_style = matches!(ext.as_str(), ".html" | ".xml" | ".vue" | ".svelte" | ".md");

    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        if in_block_comment {
            comment_lines += 1;
            if (is_c_style && trimmed.contains("*/")) || 
               (is_py_style && (trimmed.contains("\"\"\"") || trimmed.contains("'''"))) ||
               (is_html_style && trimmed.contains("-->")) {
                in_block_comment = false;
            }
            continue;
        }

        let is_comment = if is_c_style {
            if trimmed.starts_with("/*") {
                if !trimmed.contains("*/") {
                    in_block_comment = true;
                }
                true
            } else {
                trimmed.starts_with("//")
            }
        } else if is_py_style {
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                let quote = &trimmed[0..3];
                // Check if it's closed on the same line
                if trimmed.len() == 3 || !trimmed[3..].contains(quote) {
                    in_block_comment = true;
                }
                true
            } else {
                trimmed.starts_with('#')
            }
        } else if is_html_style {
            if trimmed.starts_with("<!--") {
                if !trimmed.contains("-->") {
                    in_block_comment = true;
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        if is_comment {
            comment_lines += 1;
        }
    }

    let code_lines = lines - blank_lines - comment_lines;

    Some(FileStats {
        path: path.to_path_buf(),
        ext,
        lines,
        blank_lines,
        comment_lines,
        code_lines,
    })
}

pub fn aggregate_by_ext(stats: &[FileStats]) -> HashMap<String, ExtStats> {
    let mut map: HashMap<String, ExtStats> = HashMap::new();
    for stat in stats {
        let entry = map.entry(stat.ext.clone()).or_insert_with(|| ExtStats {
            ext: stat.ext.clone(),
            file_count: 0,
            lines: 0,
            blank_lines: 0,
            comment_lines: 0,
            code_lines: 0,
        });
        entry.file_count += 1;
        entry.lines += stat.lines;
        entry.blank_lines += stat.blank_lines;
        entry.comment_lines += stat.comment_lines;
        entry.code_lines += stat.code_lines;
    }
    map
}

#[derive(Debug, Clone)]
pub struct ExtStats {
    pub ext: String,
    pub file_count: usize,
    pub lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
}
