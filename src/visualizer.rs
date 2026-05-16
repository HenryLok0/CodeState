use comfy_table::{Table, Cell, Color, Attribute, CellAlignment};
use crate::scanner::ExtStats;
use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct UnifiedStats {
    pub path: String,
    pub ext: String,
    pub lines: usize,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub complexity: f64,
}

pub fn print_details_table(stats: &[UnifiedStats], top: Option<usize>, failures_only: bool) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Extension").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Code").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Comments").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Blanks").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Complexity").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    let mut filtered_stats: Vec<&UnifiedStats> = stats.iter().filter(|s| {
        if failures_only {
            s.lines > 300 || s.complexity > 10.0
        } else {
            true
        }
    }).collect();

    filtered_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    if let Some(n) = top {
        filtered_stats.truncate(n);
    }

    for s in filtered_stats {
        table.add_row(vec![
            Cell::new(&s.path),
            Cell::new(&s.ext).fg(Color::Green),
            Cell::new(s.lines).set_alignment(CellAlignment::Right),
            Cell::new(s.code).set_alignment(CellAlignment::Right),
            Cell::new(s.comments).set_alignment(CellAlignment::Right),
            Cell::new(s.blanks).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.1}", s.complexity)).set_alignment(CellAlignment::Right),
        ]);
    }

    println!("\n{table}");
}

pub fn print_summary_table(stats: &HashMap<String, ExtStats>) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Extension").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Code").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Comments").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Blanks").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_code = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;

    for s in sorted_stats {
        table.add_row(vec![
            Cell::new(&s.ext).fg(Color::Green),
            Cell::new(s.file_count).set_alignment(CellAlignment::Right),
            Cell::new(s.lines).set_alignment(CellAlignment::Right),
            Cell::new(s.code_lines).set_alignment(CellAlignment::Right),
            Cell::new(s.comment_lines).set_alignment(CellAlignment::Right),
            Cell::new(s.blank_lines).set_alignment(CellAlignment::Right),
        ]);
        total_files += s.file_count;
        total_lines += s.lines;
        total_code += s.code_lines;
        total_comments += s.comment_lines;
        total_blanks += s.blank_lines;
    }

    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
        Cell::new(total_files).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_lines).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_code).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_comments).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_blanks).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
    ]);

    println!("\n{table}");
}

pub fn print_git_hotspots(hotspots: &[crate::git::Hotspot]) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Commits (Hotspot)").add_attribute(Attribute::Bold).fg(Color::Red),
    ]);

    for h in hotspots {
        table.add_row(vec![
            Cell::new(&h.path),
            Cell::new(h.commits).set_alignment(CellAlignment::Right),
        ]);
    }

    println!("\n{table}");
}

pub fn print_file_authors(authors_map: &HashMap<String, String>) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Last Author").add_attribute(Attribute::Bold).fg(Color::Yellow),
    ]);

    let mut sorted_entries: Vec<(&String, &String)> = authors_map.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (path, author) in sorted_entries {
        table.add_row(vec![
            Cell::new(path),
            Cell::new(author),
        ]);
    }

    println!("\n=== File Authors ===");
    println!("{table}");
}

pub fn print_contributor_stats(stats: &[crate::git::ContributorStat]) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Contributor").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Commits").add_attribute(Attribute::Bold).fg(Color::Green),
    ]);

    let mut total_commits = 0;
    for s in stats {
        table.add_row(vec![
            Cell::new(&s.name),
            Cell::new(s.commits).set_alignment(CellAlignment::Right),
        ]);
        total_commits += s.commits;
    }

    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
        Cell::new(total_commits).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
    ]);

    println!("\n=== Contributor Statistics ===");
    println!("{table}");
}

pub fn print_recent_churn(hotspots: &[crate::git::Hotspot], days: u64) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Changes").add_attribute(Attribute::Bold).fg(Color::Red),
    ]);

    for h in hotspots {
        table.add_row(vec![
            Cell::new(&h.path),
            Cell::new(h.commits).set_alignment(CellAlignment::Right),
        ]);
    }

    println!("\n=== Most Changed Files (Last {} Days) ===", days);
    if hotspots.is_empty() {
        println!("No changes found in the last {} days.", days);
    } else {
        println!("{table}");
    }
}

pub fn generate_markdown(stats: &HashMap<String, ExtStats>, details: Option<&[UnifiedStats]>) -> String {
    let mut md = String::new();
    md.push_str("## Summary Statistics\n\n");
    md.push_str("| Extension | Files | Lines | Code | Comments | Blanks |\n");
    md.push_str("|-----------|-------|-------|------|----------|--------|\n");
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_code = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;

    for s in sorted_stats {
        md.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n",
            s.ext, s.file_count, s.lines, s.code_lines, s.comment_lines, s.blank_lines));
        total_files += s.file_count;
        total_lines += s.lines;
        total_code += s.code_lines;
        total_comments += s.comment_lines;
        total_blanks += s.blank_lines;
    }
    md.push_str(&format!("| **Total** | **{}** | **{}** | **{}** | **{}** | **{}** |\n",
        total_files, total_lines, total_code, total_comments, total_blanks));

    if let Some(details) = details {
        md.push_str("\n## Detailed Statistics\n\n");
        md.push_str("| Path | Extension | Lines | Code | Comments | Blanks | Complexity |\n");
        md.push_str("|------|-----------|-------|------|----------|--------|------------|\n");
        for s in details {
            md.push_str(&format!("| {} | {} | {} | {} | {} | {} | {:.1} |\n",
                s.path, s.ext, s.lines, s.code, s.comments, s.blanks, s.complexity));
        }
    }

    md
}

pub fn generate_html(stats: &HashMap<String, ExtStats>, details: Option<&[UnifiedStats]>) -> String {
    let mut html = String::new();
    html.push_str("<h2>Summary Statistics</h2>\n");
    html.push_str("<table border=\"1\">\n");
    html.push_str("<tr><th>Extension</th><th>Files</th><th>Lines</th><th>Code</th><th>Comments</th><th>Blanks</th></tr>\n");
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_code = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;

    for s in sorted_stats {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            s.ext, s.file_count, s.lines, s.code_lines, s.comment_lines, s.blank_lines));
        total_files += s.file_count;
        total_lines += s.lines;
        total_code += s.code_lines;
        total_comments += s.comment_lines;
        total_blanks += s.blank_lines;
    }
    html.push_str(&format!("<tr><td><b>Total</b></td><td><b>{}</b></td><td><b>{}</b></td><td><b>{}</b></td><td><b>{}</b></td><td><b>{}</b></td></tr>\n",
        total_files, total_lines, total_code, total_comments, total_blanks));
    html.push_str("</table>\n");

    if let Some(details) = details {
        html.push_str("<h2>Detailed Statistics</h2>\n");
        html.push_str("<table border=\"1\">\n");
        html.push_str("<tr><th>Path</th><th>Extension</th><th>Lines</th><th>Code</th><th>Comments</th><th>Blanks</th><th>Complexity</th></tr>\n");
        for s in details {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}</td></tr>\n",
                s.path, s.ext, s.lines, s.code, s.comments, s.blanks, s.complexity));
        }
        html.push_str("</table>\n");
    }

    html
}

pub fn generate_csv(stats: &HashMap<String, ExtStats>) -> String {
    let mut csv = String::new();
    csv.push_str("Extension,Files,Lines,Code,Comments,Blanks\n");
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    for s in sorted_stats {
        csv.push_str(&format!("{},{},{},{},{},{}\n",
            s.ext, s.file_count, s.lines, s.code_lines, s.comment_lines, s.blank_lines));
    }
    csv
}

pub fn generate_json(stats: &HashMap<String, ExtStats>) -> String {
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));
    serde_json::to_string_pretty(&sorted_stats).unwrap_or_else(|_| "[]".to_string())
}

pub fn save_or_print(content: &str, output: Option<&String>) {
    if let Some(path) = output {
        if let Err(e) = std::fs::write(path, content) {
            eprintln!("! Failed to write to file {}: {}", path, e);
        } else {
            println!("✓ Output successfully saved to {}", path);
        }
    } else {
        println!("{}", content);
    }
}

pub fn print_health_score(stats: &HashMap<String, ExtStats>, details: &[UnifiedStats]) {
    let mut total_lines = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;
    
    for s in stats.values() {
        total_lines += s.lines;
        total_comments += s.comment_lines;
        total_blanks += s.blank_lines;
    }
    
    let total_complexity: f64 = details.iter().map(|d| d.complexity).sum();
    let avg_complexity = if !details.is_empty() { total_complexity / details.len() as f64 } else { 0.0 };
    
    let comment_density = if total_lines > 0 { total_comments as f64 / total_lines as f64 } else { 0.0 };
    let blank_density = if total_lines > 0 { total_blanks as f64 / total_lines as f64 } else { 0.0 };
    
    let mut score = 100.0;
    let mut suggestions = Vec::new();
    
    // Complexity penalty
    if avg_complexity > 5.0 {
        let penalty = ((avg_complexity - 5.0) * 2.0).min(30.0);
        score -= penalty;
        suggestions.push(format!("Average complexity is high ({:.1}). Consider refactoring complex functions.", avg_complexity));
    }
    
    // Comment density penalty
    if comment_density < 0.1 {
        let penalty = ((0.1 - comment_density) * 200.0).min(20.0);
        score -= penalty;
        suggestions.push(format!("Comment density is low ({:.1}%). Consider adding more documentation.", comment_density * 100.0));
    }
    
    // Blank line density penalty (readability)
    if blank_density < 0.05 {
        let penalty = ((0.05 - blank_density) * 200.0).min(10.0);
        score -= penalty;
        suggestions.push(format!("Blank line density is low ({:.1}%). Consider adding blank lines to separate logical blocks.", blank_density * 100.0));
    }
    
    score = score.max(0.0);
    
    println!("\n=== Project Health Score ===");
    println!("Score: {:.1} / 100.0", score);
    
    if score >= 90.0 {
        println!("Status: Excellent \u{1F31F}");
    } else if score >= 75.0 {
        println!("Status: Good \u{2714}");
    } else if score >= 60.0 {
        println!("Status: Fair \u{26A0}");
    } else {
        println!("Status: Needs Improvement \u{274C}");
    }
    
    if !suggestions.is_empty() {
        println!("\nSuggestions:");
        for sug in suggestions {
            println!("- {}", sug);
        }
    }
}

pub fn print_complexity_graph(details: &[UnifiedStats]) {
    if details.is_empty() {
        println!("\nNo data for complexity graph.");
        return;
    }
    
    let mut sorted_details: Vec<&UnifiedStats> = details.iter().collect();
    sorted_details.sort_by(|a, b| b.complexity.partial_cmp(&a.complexity).unwrap_or(std::cmp::Ordering::Equal));
    
    let top_n = 15.min(sorted_details.len());
    let top_details = &sorted_details[..top_n];
    
    let max_complexity = top_details.iter().map(|d| d.complexity).fold(0.0, f64::max);
    
    println!("\n=== Top {} Most Complex Files ===", top_n);
    
    if max_complexity <= 0.0 {
        println!("All files have 0 complexity.");
        return;
    }
    
    let max_bar_len = 40;
    let max_path_len = top_details.iter().map(|d| d.path.len()).max().unwrap_or(10).min(50);
    
    for d in top_details {
        let bar_len = ((d.complexity / max_complexity) * max_bar_len as f64).round() as usize;
        let bar = "\u{2588}".repeat(bar_len);
        
        let path_display = if d.path.len() > 50 {
            format!("...{}", &d.path[d.path.len() - 47..])
        } else {
            d.path.clone()
        };
        
        println!("{:<width$} | {:<bar_width$} | {:.1}", 
            path_display, 
            bar, 
            d.complexity, 
            width = max_path_len, 
            bar_width = max_bar_len
        );
    }
}

pub fn print_langdist_chart(stats: &HashMap<String, ExtStats>) {
    if stats.is_empty() {
        println!("\nNo data for language distribution.");
        return;
    }
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    let total_lines: usize = sorted_stats.iter().map(|s| s.lines).sum();
    
    println!("\n=== Language Distribution (by lines of code) ===");
    
    if total_lines == 0 {
        println!("Total lines is 0.");
        return;
    }
    
    let max_bar_len = 40;
    let max_ext_len = sorted_stats.iter().map(|s| s.ext.len()).max().unwrap_or(5).max(5);
    
    for s in sorted_stats {
        let percentage = (s.lines as f64 / total_lines as f64) * 100.0;
        if percentage < 0.1 && s.lines > 0 {
            continue; // Skip very small percentages to keep it clean, unless you want all
        }
        
        let bar_len = ((s.lines as f64 / total_lines as f64) * max_bar_len as f64).round() as usize;
        let bar = "\u{2588}".repeat(bar_len);
        
        println!("{:<width$} | {:<bar_width$} | {:>5.1}% ({})", 
            s.ext, 
            bar, 
            percentage, 
            s.lines,
            width = max_ext_len, 
            bar_width = max_bar_len
        );
    }
}

pub fn generate_badges(stats: &HashMap<String, ExtStats>) -> String {
    let mut badges = String::new();
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let top_langs: Vec<&ExtStats> = sorted_stats.into_iter()
        .filter(|s| !s.ext.trim().is_empty())
        .take(5)
        .collect();
    for stat in top_langs {
        let lang = stat.ext.replace(" ", "%20").replace("-", "--");
        badges.push_str(&format!("![{ext}](https://img.shields.io/badge/Language-{lang}-blue) ", ext = stat.ext, lang = lang));
    }
    badges.push('\n');
    badges
}

pub fn generate_readme_template(stats: &HashMap<String, ExtStats>) -> String {
    let mut readme = String::new();
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    let primary_lang = sorted_stats.iter()
        .find(|s| !s.ext.trim().is_empty())
        .map_or("Unknown", |s| s.ext.as_str());
    
    let total_lines: usize = stats.values().map(|s| s.lines).sum();
    let total_files: usize = stats.values().map(|s| s.file_count).sum();

    readme.push_str("# Project Name\n\n");
    readme.push_str(&generate_badges(stats));
    readme.push_str("\n## Overview\n\n");
    readme.push_str(&format!("This project is primarily written in **{}**.\n\n", primary_lang));
    readme.push_str("### Statistics\n\n");
    readme.push_str(&format!("- **Total Files**: {}\n", total_files));
    readme.push_str(&format!("- **Total Lines**: {}\n\n", total_lines));
    
    readme.push_str("## Getting Started\n\n");
    readme.push_str("Instructions on how to build and run the project go here.\n\n");
    
    readme.push_str("## License\n\n");
    readme.push_str("License information goes here.\n");
    
    readme
}

