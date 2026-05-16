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
    // Read the file as raw bytes to handle potentially non-utf8 cleanly, 
    // or just read_to_string and skip if invalid. We'll stick to string for simplicity 
    // but a production tool would read lossy strings or bytes.
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => String::from_utf8_lossy(&fs::read(path).ok()?).into_owned(),
    };
    
    let language = get_language_name(path);
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
        });
        entry.file_count += 1;
        entry.lines += stat.lines;
        entry.blank_lines += stat.blank_lines;
        entry.comment_lines += stat.comment_lines;
        entry.code_lines += stat.code_lines;
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
}
