use rayon::prelude::*;
use aho_corasick::{AhoCorasick, MatchKind};
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::sync::OnceLock;
use tree_sitter::Parser;

static FUNC_REGEX: OnceLock<Regex> = OnceLock::new();
static CLASS_REGEX: OnceLock<Regex> = OnceLock::new();
static DOCSTRING_REGEX: OnceLock<Regex> = OnceLock::new();
static TYPEHINT_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct AnalyzerStats {
    pub path: PathBuf,
    pub complexity: f64,
    #[allow(dead_code)]
    pub todo_count: usize,
    #[allow(dead_code)]
    pub functions_count: usize,
    #[allow(dead_code)]
    pub naming_violations: usize,
    pub naming_violations_details: Vec<String>,
    pub docstrings_count: usize,
    pub typehints_count: usize,
}

// Simplified analyzer that scans concurrently using Aho-Corasick for extreme speed
pub fn analyze_files(paths: &[PathBuf], check_naming: bool) -> Vec<AnalyzerStats> {
    let patterns = &[
        // Complexity (0..10)
        " if ", " for ", " while ", " case ", " catch ", " try ", " except ", "&&", "||", "?:",
        // Functions (10..13)
        "def ", "function ", "fn ",
        // TODOs (13..15)
        "todo", "fixme",
    ];

    let ac = aho_corasick::AhoCorasickBuilder::new()
        .ascii_case_insensitive(true)
        .match_kind(MatchKind::Standard)
        .build(patterns)
        .unwrap();

    paths
        .into_par_iter()
        .filter_map(|p| analyze_file(p, &ac, check_naming))
        .collect()
}

fn extract_rust_functions(content: &str) -> Vec<String> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() {
        return Vec::new();
    }
    
    let mut funcs = Vec::new();
    if let Some(tree) = parser.parse(content, None) {
        let mut queue = vec![tree.root_node()];
        
        while let Some(node) = queue.pop() {
            if node.kind() == "function_item" {
                // Find identifier child
                let mut c = node.walk();
                for child in node.children(&mut c) {
                    if child.kind() == "identifier" {
                        if let Ok(name) = child.utf8_text(content.as_bytes()) {
                            funcs.push(name.to_string());
                        }
                    }
                }
            }
            
            let mut c = node.walk();
            for child in node.children(&mut c) {
                queue.push(child);
            }
        }
    }
    funcs
}

fn is_word_character(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn analyze_file(path: &Path, ac: &AhoCorasick, check_naming: bool) -> Option<AnalyzerStats> {
    let content = fs::read_to_string(path).ok()?;
    
    let mut complexity = 0.0;
    let mut todo_count = 0;
    let mut functions_count = 0;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let is_rust = ext == "rs";
    let is_py = ext == "py";
    let is_rust_or_py = is_rust || is_py;
    
    for line in content.lines() {
        let padded = format!(" {} ", line);
        let bytes = padded.as_bytes();
        
        for mat in ac.find_iter(&padded) {
            let pid = mat.pattern().as_usize();
            
            if pid < 10 {
                // Complexity keywords
                complexity += 1.0;
            } else if pid < 13 {
                // Functions (regex-based fallback)
                if !is_rust {
                    let start = mat.start();
                    if start == 0 || !is_word_character(bytes[start - 1]) {
                        functions_count += 1;
                    }
                }
            } else {
                // TODO / FIXME
                // Enforce word boundaries
                let start = mat.start();
                let end = mat.end();
                let prev_ok = start == 0 || !is_word_character(bytes[start - 1]);
                let next_ok = end == bytes.len() || !is_word_character(bytes[end]);
                
                if prev_ok && next_ok {
                    todo_count += 1;
                }
            }
        }
    }
    
    if is_rust {
        functions_count = extract_rust_functions(&content).len();
    }
    
    let mut naming_violations_details = Vec::new();
    
    if check_naming {
        let func_re = FUNC_REGEX.get_or_init(|| Regex::new(r"(?:fn|def|function)\s+([a-zA-Z0-9_]+)").unwrap());
        let class_re = CLASS_REGEX.get_or_init(|| Regex::new(r"class\s+([a-zA-Z0-9_]+)").unwrap());
        
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let is_rust_or_py = ext == "rs" || ext == "py";
        
        for cap in func_re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str();
                if is_rust_or_py {
                    // Check snake_case (lowercase, digits, underscores)
                    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                        naming_violations_details.push(format!("Function '{}' in {:?} should be snake_case", name, path));
                    }
                }
            }
        }
        
        for cap in class_re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str();
                // Check PascalCase (starts with uppercase)
                if !name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                    naming_violations_details.push(format!("Class '{}' in {:?} should be PascalCase", name, path));
                }
            }
        }
    }
    
    let naming_violations = naming_violations_details.len();

    let mut docstrings_count = 0;
    let mut typehints_count = 0;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let is_rust_or_py = ext == "rs" || ext == "py";

    if is_rust_or_py {
        let doc_re = DOCSTRING_REGEX.get_or_init(|| Regex::new(r"(?m)^\s*(///|#|\x22\x22\x22)").unwrap());
        let type_re = TYPEHINT_REGEX.get_or_init(|| Regex::new(r"(?m)(:\s*[a-zA-Z_]|->\s*[a-zA-Z_])").unwrap());
        
        docstrings_count = doc_re.find_iter(&content).count();
        typehints_count = type_re.find_iter(&content).count();
    }

    Some(AnalyzerStats {
        path: path.to_path_buf(),
        complexity,
        todo_count,
        functions_count,
        naming_violations,
        naming_violations_details,
        docstrings_count,
        typehints_count,
    })
}

pub fn find_deadcode(paths: &[PathBuf]) -> Vec<String> {
    let func_re = FUNC_REGEX.get_or_init(|| Regex::new(r"(?:fn|def|function)\s+([a-zA-Z0-9_]+)").unwrap());
    
    // Pass 1: Extract all function names
    let mut all_functions: Vec<String> = paths.into_par_iter()
        .filter_map(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                let mut local_funcs = Vec::new();
                let is_rust = path.extension().map_or(false, |e| e == "rs");
                
                if is_rust {
                    local_funcs = extract_rust_functions(&content);
                } else {
                    for cap in func_re.captures_iter(&content) {
                        if let Some(m) = cap.get(1) {
                            local_funcs.push(m.as_str().to_string());
                        }
                    }
                }
                Some(local_funcs)
            } else {
                None
            }
        })
        .flatten()
        .collect();
        
    all_functions.sort();
    all_functions.dedup();
    
    if all_functions.is_empty() {
        return Vec::new();
    }
    
    // Pass 2: Count occurrences
    let ac = aho_corasick::AhoCorasickBuilder::new()
        .match_kind(MatchKind::Standard)
        .build(&all_functions)
        .unwrap();
        
    let counts = paths.into_par_iter().map(|path| {
        let mut local_counts = vec![0usize; all_functions.len()];
        if let Ok(content) = fs::read_to_string(path) {
            let bytes = content.as_bytes();
            for mat in ac.find_iter(&content) {
                let start = mat.start();
                let end = mat.end();
                
                let prev_ok = start == 0 || !is_word_character(bytes[start - 1]);
                let next_ok = end == bytes.len() || !is_word_character(bytes[end]);
                
                if prev_ok && next_ok {
                    local_counts[mat.pattern().as_usize()] += 1;
                }
            }
        }
        local_counts
    }).reduce(
        || vec![0usize; all_functions.len()],
        |mut a, b| {
            for (i, v) in b.iter().enumerate() {
                a[i] += v;
            }
            a
        }
    );
    
    let mut deadcode = Vec::new();
    for (i, count) in counts.iter().enumerate() {
        if *count == 1 {
            deadcode.push(all_functions[i].clone());
        }
    }
    
    deadcode
}

