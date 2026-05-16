use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileStats {
    pub path: PathBuf,
    pub language: String,
    pub lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
    pub complexity: f64,
    pub todo_count: usize,
    pub secrets_found: usize,
    pub size_bytes: u64,
    pub created_at: Option<SystemTime>,
    pub modified_at: Option<SystemTime>,
}

pub fn get_language_name(path: &Path) -> String {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
    
    // Check exact filenames first
    match file_name.as_str() {
        "makefile" | "gnumakefile" | "cmakelists.txt" => return "Makefile".to_string(),
        "dockerfile" => return "Dockerfile".to_string(),
        "docker-compose.yml" | "docker-compose.yaml" => return "Docker Compose".to_string(),
        "license" | "license.txt" | "license.md" => return "License".to_string(),
        "cargo.toml" => return "Cargo (Rust)".to_string(),
        "cargo.lock" => return "Cargo Lock".to_string(),
        "package.json" => return "NPM Package".to_string(),
        "package-lock.json" => return "NPM Lock".to_string(),
        "yarn.lock" => return "Yarn Lock".to_string(),
        "gemfile" => return "Ruby Gemfile".to_string(),
        "gemfile.lock" => return "Ruby Gemfile Lock".to_string(),
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "Git Config".to_string(),
        "requirements.txt" => return "Python Requirements".to_string(),
        _ => {}
    }

    // Check extensions
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        return match ext_lower.as_str() {
            "rs" => "Rust",
            "py" | "pyw" => "Python",
            "js" | "mjs" | "cjs" => "JavaScript",
            "ts" | "mts" | "cts" => "TypeScript",
            "jsx" => "JavaScript JSX",
            "tsx" => "TypeScript JSX",
            "c" => "C",
            "h" => "C Header",
            "cpp" | "cxx" | "cc" => "C++",
            "hpp" | "hxx" | "hh" => "C++ Header",
            "cs" => "C#",
            "java" => "Java",
            "go" => "Go",
            "rb" => "Ruby",
            "php" => "PHP",
            "swift" => "Swift",
            "kt" | "kts" => "Kotlin",
            "scala" => "Scala",
            "html" | "htm" => "HTML",
            "css" => "CSS",
            "scss" => "SCSS",
            "sass" => "Sass",
            "less" => "Less",
            "md" | "markdown" => "Markdown",
            "json" => "JSON",
            "xml" => "XML",
            "yaml" | "yml" => "YAML",
            "toml" => "TOML",
            "sh" | "bash" | "zsh" | "bat" | "cmd" | "ps1" => "Shell",
            "sql" => "SQL",
            "vue" => "Vue",
            "svelte" => "Svelte",
            "dart" => "Dart",
            "r" => "R",
            "lua" => "Dart",
            "pl" | "pm" | "t" => "Perl",
            "txt" => "Text",
            "ini" => "INI",
            "cfg" | "conf" | "config" => "Config",
            "csv" => "CSV",
            "env" => "Dotenv",
            _ => {
                // Capitalize first letter of unknown extension
                let mut chars = ext_lower.chars();
                return match chars.next() {
                    None => "Unknown".to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                };
            }
        }.to_string();
    }

    // Check shebang
    if let Ok(mut file) = fs::File::open(path) {
        use std::io::{Read, Seek, SeekFrom};
        let mut buffer = [0; 64];
        if file.read_exact(&mut buffer).is_ok() || file.seek(SeekFrom::Start(0)).is_ok() {
            let content = String::from_utf8_lossy(&buffer);
            if content.starts_with("#!") {
                if content.contains("python") { return "Python".to_string(); }
                if content.contains("node") { return "JavaScript".to_string(); }
                if content.contains("sh") || content.contains("bash") || content.contains("zsh") { return "Shell".to_string(); }
                if content.contains("ruby") { return "Ruby".to_string(); }
                if content.contains("perl") { return "Perl".to_string(); }
            }
        }
    }

    "Unknown".to_string()
}

pub fn scan_directory(dir: &str, excludes: Option<&Vec<String>>, exts: Option<&Vec<String>>, use_cache: bool) -> Vec<FileStats> {
    let cache_file = Path::new(".codestate/cache.json");
    let mut cache: HashMap<PathBuf, FileStats> = HashMap::new();

    if use_cache {
        if let Ok(content) = fs::read_to_string(cache_file) {
            if let Ok(cached_stats) = serde_json::from_str::<Vec<FileStats>>(&content) {
                for stat in cached_stats {
                    cache.insert(stat.path.clone(), stat);
                }
            }
        }
    }

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

    let stats: Vec<FileStats> = paths
        .into_par_iter()
        .filter_map(|path| {
            if use_cache {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Some(cached) = cache.get(&path) {
                            if cached.modified_at == Some(modified) {
                                return Some(cached.clone());
                            }
                        }
                    }
                }
            }
            analyze_file(&path)
        })
        .collect();

    if use_cache {
        if let Err(_) = fs::create_dir_all(".codestate") {}
        if let Ok(json) = serde_json::to_string(&stats) {
            let _ = fs::write(cache_file, json);
        }
    }

    stats
}

fn analyze_file(path: &Path) -> Option<FileStats> {
    // 1. Zero-Allocation Byte Scanner
    // Read raw bytes instead of allocating and splitting strings for extreme performance.
    let bytes = fs::read(path).ok()?;
    
    let language = get_language_name(path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_c_style = matches!(ext.as_str(), "c" | "cpp" | "h" | "hpp" | "js" | "ts" | "jsx" | "tsx" | "java" | "rs" | "go" | "cs" | "swift" | "kt" | "php");
    let is_py_style = matches!(ext.as_str(), "py" | "sh" | "rb" | "pl" | "yml" | "yaml");
    let is_html_style = matches!(ext.as_str(), "html" | "xml" | "vue" | "svelte" | "md");

    let mut lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut code_lines = 0;
    let mut complexity = 0.0;
    let mut todo_count = 0;
    let mut secrets_found = 0;
    
    // 2. Semantic Accuracy State Machine
    let mut in_string = false;
    let mut string_char = b'\0';
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut block_char = b'\0';
    
    let mut line_has_code = false;
    let mut line_has_comment = false;
    let mut escaped = false;
    
    let mut word_start = 0;
    let mut in_word = false;

    let len = bytes.len();
    let mut i = 0;

    if len > 0 {
        lines = 1;
    }

    while i < len {
        let b = bytes[i];
        
        let is_alpha = b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
        
        if is_alpha {
            if !in_word {
                in_word = true;
                word_start = i;
            }
        } else {
            if in_word {
                let word = &bytes[word_start..i];
                if !in_string && !in_block_comment && !in_line_comment {
                    match word {
                        b"if" | b"for" | b"while" | b"match" | b"catch" | b"case" | b"try" | b"except" => {
                            complexity += 1.0;
                        }
                        _ => {}
                    }
                } else if in_block_comment || in_line_comment {
                    if word.len() == 4 && word.eq_ignore_ascii_case(b"TODO") {
                        todo_count += 1;
                    } else if word.len() == 4 && word.eq_ignore_ascii_case(b"HACK") {
                        todo_count += 1;
                    } else if word.len() == 5 && word.eq_ignore_ascii_case(b"FIXME") {
                        todo_count += 1;
                    }
                }
                
                // Secret Scanning
                if word.starts_with(b"AKIA") || word.starts_with(b"ghp_") || word.starts_with(b"sk_live_") || word.starts_with(b"xoxb-") {
                    secrets_found += 1;
                }
                
                in_word = false;
            }
            
            if !in_string && !in_block_comment && !in_line_comment {
                if b == b'?' {
                    complexity += 1.0;
                } else if b == b'&' && i + 1 < len && bytes[i + 1] == b'&' {
                    complexity += 1.0;
                    // Don't skip i+=1 here because we still need to process b'\n' and other logic.
                    // Instead, we just let it process the second '&' normally, it won't trigger anything.
                } else if b == b'|' && i + 1 < len && bytes[i + 1] == b'|' {
                    complexity += 1.0;
                }
            }
        }

        if b == b'\n' {
            if !line_has_code && !line_has_comment {
                blank_lines += 1;
            } else if line_has_comment && !line_has_code {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
            if i + 1 < len {
                lines += 1;
            }
            in_line_comment = false;
            line_has_code = false;
            line_has_comment = false;
            escaped = false;
            i += 1;
            continue;
        }

        if b == b'\r' {
            i += 1;
            continue;
        }

        let is_ws = b == b' ' || b == b'\t';

        if in_line_comment {
            i += 1;
            continue;
        }

        if in_block_comment {
            line_has_comment = true;
            if is_c_style && b == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            } else if is_html_style && b == b'-' && i + 2 < len && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
                in_block_comment = false;
                i += 3;
                continue;
            } else if is_py_style && b == block_char && i + 2 < len && bytes[i + 1] == block_char && bytes[i + 2] == block_char {
                in_block_comment = false;
                i += 3;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            line_has_code = true;
            if b == b'\\' && !escaped {
                escaped = true;
            } else if b == string_char && !escaped {
                in_string = false;
                escaped = false;
            } else {
                escaped = false;
            }
            i += 1;
            continue;
        }

        if !is_ws {
            if is_py_style && (b == b'"' || b == b'\'') && i + 2 < len && bytes[i + 1] == b && bytes[i + 2] == b {
                in_block_comment = true;
                block_char = b;
                line_has_comment = true;
                i += 3;
                continue;
            }

            if b == b'"' || b == b'\'' || b == b'`' {
                in_string = true;
                string_char = b;
                line_has_code = true;
                i += 1;
                continue;
            }

            if is_c_style && b == b'/' && i + 1 < len {
                if bytes[i + 1] == b'/' {
                    in_line_comment = true;
                    line_has_comment = true;
                    i += 2;
                    continue;
                } else if bytes[i + 1] == b'*' {
                    in_block_comment = true;
                    line_has_comment = true;
                    i += 2;
                    continue;
                }
            } else if is_py_style && b == b'#' {
                in_line_comment = true;
                line_has_comment = true;
                i += 1;
                continue;
            } else if is_html_style && b == b'<' && i + 3 < len && bytes[i + 1] == b'!' && bytes[i + 2] == b'-' && bytes[i + 3] == b'-' {
                in_block_comment = true;
                line_has_comment = true;
                i += 4;
                continue;
            }

            line_has_code = true;
        }

        i += 1;
    }
    
    // Check if ended while in word
    if in_word {
        let word = &bytes[word_start..len];
        if !in_string && !in_block_comment && !in_line_comment {
            match word {
                b"if" | b"for" | b"while" | b"match" | b"catch" | b"case" | b"try" | b"except" => {
                    complexity += 1.0;
                }
                _ => {}
            }
            // Secret Scanning
            if word.starts_with(b"AKIA") || word.starts_with(b"ghp_") || word.starts_with(b"sk_live_") || word.starts_with(b"xoxb-") {
                secrets_found += 1;
            }
        } else if in_block_comment || in_line_comment {
            if word.len() == 4 && word.eq_ignore_ascii_case(b"TODO") {
                todo_count += 1;
            } else if word.len() == 4 && word.eq_ignore_ascii_case(b"HACK") {
                todo_count += 1;
            } else if word.len() == 5 && word.eq_ignore_ascii_case(b"FIXME") {
                todo_count += 1;
            }
        }
        
        // Secret Scanning at EOF
        if word.starts_with(b"AKIA") || word.starts_with(b"ghp_") || word.starts_with(b"sk_live_") || word.starts_with(b"xoxb-") {
            secrets_found += 1;
        }
    }

    if len > 0 && bytes[len - 1] != b'\n' {
        if !line_has_code && !line_has_comment {
            blank_lines += 1;
        } else if line_has_comment && !line_has_code {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    let (size_bytes, created_at, modified_at) = fs::metadata(path).map(|m| {
        (
            m.len(),
            m.created().ok(),
            m.modified().ok()
        )
    }).unwrap_or((0, None, None));

    Some(FileStats {
        path: path.to_path_buf(),
        language,
        lines,
        blank_lines,
        comment_lines,
        code_lines,
        complexity,
        todo_count,
        secrets_found,
        size_bytes,
        created_at,
        modified_at,
    })
}

pub fn aggregate_by_ext(stats: &[FileStats]) -> HashMap<String, LangStats> {
    let mut map: HashMap<String, LangStats> = HashMap::new();
    for stat in stats {
        let entry = map.entry(stat.language.clone()).or_insert_with(|| LangStats {
            language: stat.language.clone(),
            file_count: 0,
            lines: 0,
            blank_lines: 0,
            comment_lines: 0,
            code_lines: 0,
            complexity: 0.0,
            todo_count: 0,
            secrets_found: 0,
        });
        entry.file_count += 1;
        entry.lines += stat.lines;
        entry.blank_lines += stat.blank_lines;
        entry.comment_lines += stat.comment_lines;
        entry.code_lines += stat.code_lines;
        entry.complexity += stat.complexity;
        entry.todo_count += stat.todo_count;
        entry.secrets_found += stat.secrets_found;
    }
    map
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LangStats {
    pub language: String,
    pub file_count: usize,
    pub lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
    pub complexity: f64,
    pub todo_count: usize,
    pub secrets_found: usize,
}
