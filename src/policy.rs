use serde::Deserialize;
use std::fs;
use std::path::Path;
use crate::visualizer::UnifiedStats;

#[derive(Deserialize, Debug, Default)]
pub struct Policy {
    pub rules: Option<Rules>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Rules {
    pub max_lines_per_file: Option<usize>,
    pub max_complexity: Option<f64>,
    pub max_todo_count: Option<usize>,
    pub allow_secrets: Option<bool>,
}

pub fn check_policy(dir: &str, details: &[UnifiedStats]) -> bool {
    let policy_path = Path::new(dir).join(".codestate.yml");
    if !policy_path.exists() {
        println!("No .codestate.yml found. Skipping policy check.");
        return true;
    }

    println!("\n=== Policy Check (.codestate.yml) ===");
    let content = match fs::read_to_string(&policy_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to read policy file: {}", e);
            return false;
        }
    };

    // We use serde_yaml to parse. We need to add it to Cargo.toml.
    // Wait, to minimize dependencies, let's parse simple yaml manually if possible or add serde_yaml.
    // Since we only need very basic fields, we can do a naive parse or just add serde_yaml.
    let mut max_lines = 1000;
    let mut max_comp = 20.0;
    let mut max_todos = 50;
    let mut allow_sec = false;

    // Naive fallback parse if we don't have serde_yaml:
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("max_lines_per_file:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(num) = val.trim().parse() { max_lines = num; }
            }
        } else if line.starts_with("max_complexity:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(num) = val.trim().parse() { max_comp = num; }
            }
        } else if line.starts_with("max_todo_count:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(num) = val.trim().parse() { max_todos = num; }
            }
        } else if line.starts_with("allow_secrets:") {
            if let Some(val) = line.split(':').nth(1) {
                allow_sec = val.trim() == "true";
            }
        }
    }

    println!("Policies applied:");
    println!("  - Max lines per file: {}", max_lines);
    println!("  - Max complexity: {:.1}", max_comp);
    println!("  - Max TODOs per file: {}", max_todos);
    println!("  - Allow secrets: {}", allow_sec);

    let mut passed = true;
    for s in details {
        if s.lines > max_lines {
            println!("  [FAIL] {} has {} lines (> {})", s.path, s.lines, max_lines);
            passed = false;
        }
        if s.complexity > max_comp {
            println!("  [FAIL] {} has complexity {:.1} (> {:.1})", s.path, s.complexity, max_comp);
            passed = false;
        }
        if s.todo_count > max_todos {
            println!("  [FAIL] {} has {} TODOs (> {})", s.path, s.todo_count, max_todos);
            passed = false;
        }
        if !allow_sec && s.secrets_found > 0 {
            println!("  [FAIL] {} contains {} potential secret(s)", s.path, s.secrets_found);
            passed = false;
        }
    }

    if passed {
        println!("Policy check PASSED! \u{2714}");
    } else {
        println!("Policy check FAILED! \u{274C}");
    }
    
    passed
}
