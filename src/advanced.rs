use std::path::PathBuf;
use std::fs;
use regex::Regex;
use std::sync::OnceLock;
use serde_json::json;

pub fn style_check(paths: &[PathBuf]) {
    println!("\n[--style-check] Checking for style issues...");
    let mut total_issues = 0;
    
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            let mut issues = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            
            for (i, line) in lines.iter().enumerate() {
                let line_num = i + 1;
                
                // Check trailing whitespace
                if line.ends_with(' ') || line.ends_with('\t') {
                    issues.push(format!("Line {}: Trailing whitespace", line_num));
                }
                
                // Check line length > 100
                if line.chars().count() > 100 {
                    issues.push(format!("Line {}: Line exceeds 100 characters", line_num));
                }
            }
            
            // Check EOF newline missing
            if !content.is_empty() && !content.ends_with('\n') {
                issues.push("EOF: Missing newline at end of file".to_string());
            }
            
            if !issues.is_empty() {
                println!("  {:?}:", path);
                for issue in issues {
                    println!("    - {}", issue);
                    total_issues += 1;
                }
            }
        }
    }
    
    if total_issues == 0 {
        println!("  No style issues found!");
    } else {
        println!("  Found {} style issues in total.", total_issues);
    }
}

static ROUTE_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn generate_openapi(paths: &[PathBuf]) {
    println!("\n[--openapi] Generating OpenAPI skeleton from Python files...");
    
    let route_re = ROUTE_REGEX.get_or_init(|| {
        Regex::new(r#"@app\.(get|post|put|delete|patch)\(["']([^"']+)["']\)"#).unwrap()
    });
    
    let mut paths_obj = serde_json::Map::new();
    
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) == Some("py") {
            if let Ok(content) = fs::read_to_string(path) {
                for cap in route_re.captures_iter(&content) {
                    let method = cap.get(1).unwrap().as_str().to_lowercase();
                    let route = cap.get(2).unwrap().as_str();
                    
                    let route_entry = paths_obj.entry(route.to_string()).or_insert(json!({}));
                    if let Some(route_map) = route_entry.as_object_mut() {
                        route_map.insert(method, json!({
                            "summary": format!("Auto-generated {} route", route),
                            "responses": {
                                "200": {
                                    "description": "Successful response"
                                }
                            }
                        }));
                    }
                }
            }
        }
    }
    
    let openapi_json = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Auto-generated API",
            "version": "1.0.0"
        },
        "paths": paths_obj
    });
    
    println!("{}", serde_json::to_string_pretty(&openapi_json).unwrap());
}

static COVERAGE_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn parse_test_coverage(coverage_path: &str) {
    println!("\n[--test-coverage] Parsing coverage file: {}", coverage_path);
    
    if let Ok(content) = fs::read_to_string(coverage_path) {
        let coverage_re = COVERAGE_REGEX.get_or_init(|| {
            Regex::new(r#"<coverage[^>]*line-rate="([^"]+)""#).unwrap()
        });
        
        if let Some(cap) = coverage_re.captures(&content) {
            if let Ok(line_rate) = cap.get(1).unwrap().as_str().parse::<f64>() {
                println!("  Overall Line Coverage: {:.2}%", line_rate * 100.0);
            } else {
                println!("  Could not parse line-rate as a number.");
            }
        } else {
            println!("  Could not find line-rate attribute in <coverage> tag.");
        }
    } else {
        println!("  Failed to read coverage file: {}", coverage_path);
    }
}
