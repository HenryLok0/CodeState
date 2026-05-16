use std::fs;
use std::path::{Path, PathBuf};

pub fn analyze_lockfiles(paths: &[PathBuf]) {
    let mut found = false;
    for path in paths {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        if file_name == "Cargo.lock" {
            if !found {
                println!("\n=== Software Composition Analysis (SCA) ===");
                found = true;
            }
            analyze_cargo_lock(path);
        } else if file_name == "package-lock.json" {
            if !found {
                println!("\n=== Software Composition Analysis (SCA) ===");
                found = true;
            }
            analyze_package_lock(path);
        } else if file_name == "requirements.txt" {
            if !found {
                println!("\n=== Software Composition Analysis (SCA) ===");
                found = true;
            }
            analyze_requirements_txt(path);
        }
    }
}

fn analyze_cargo_lock(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        let mut packages = 0;
        for line in content.lines() {
            if line.trim() == "[[package]]" {
                packages += 1;
            }
        }
        println!("  - Cargo.lock: {} total dependencies", packages);
    }
}

fn analyze_package_lock(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut count = 0;
            // v2/v3 lockfiles
            if let Some(packages) = json.get("packages").and_then(|p| p.as_object()) {
                count += packages.len().saturating_sub(1); // subtract root ""
            } 
            // v1 lockfiles
            else if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                count += count_npm_deps_v1(&json);
            }
            println!("  - package-lock.json: {} total dependencies", count);
        }
    }
}

fn count_npm_deps_v1(json: &serde_json::Value) -> usize {
    let mut count = 0;
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        count += deps.len();
        for (_, val) in deps {
            count += count_npm_deps_v1(val);
        }
    }
    count
}

fn analyze_requirements_txt(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        let count = content.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .count();
        println!("  - requirements.txt: {} defined dependencies", count);
    }
}
