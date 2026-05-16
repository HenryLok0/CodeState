use rayon::prelude::*;
use aho_corasick::{AhoCorasick, MatchKind};
use std::fs;
use std::path::{Path, PathBuf};

pub struct AnalyzerStats {
    pub path: PathBuf,
    pub complexity: f64,
    pub todo_count: usize,
    pub functions_count: usize,
    pub naming_violations: usize,
}

// Simplified analyzer that scans concurrently using Aho-Corasick for extreme speed
pub fn analyze_files(paths: &[PathBuf]) -> Vec<AnalyzerStats> {
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
        .filter_map(|p| analyze_file(p, &ac))
        .collect()
}

fn is_word_character(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn analyze_file(path: &Path, ac: &AhoCorasick) -> Option<AnalyzerStats> {
    let content = fs::read_to_string(path).ok()?;
    
    let mut complexity = 0.0;
    let mut todo_count = 0;
    let mut functions_count = 0;
    
    for line in content.lines() {
        let padded = format!(" {} ", line);
        let bytes = padded.as_bytes();
        
        for mat in ac.find_iter(&padded) {
            let pid = mat.pattern().as_usize();
            
            if pid < 10 {
                // Complexity keywords
                complexity += 1.0;
            } else if pid < 13 {
                // Functions
                // Ensure it's not part of another word (e.g. "my_def " shouldn't match "def ")
                let start = mat.start();
                if start == 0 || !is_word_character(bytes[start - 1]) {
                    functions_count += 1;
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
    
    let naming_violations = 0;

    Some(AnalyzerStats {
        path: path.to_path_buf(),
        complexity,
        todo_count,
        functions_count,
        naming_violations,
    })
}
