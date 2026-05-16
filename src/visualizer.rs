use comfy_table::{Table, Cell, Color, Attribute, CellAlignment};
use crate::scanner::ExtStats;
use std::collections::HashMap;
use rust_xlsxwriter::{Workbook, Format, Color as XlsxColor};

#[derive(serde::Serialize)]
pub struct UnifiedStats {
    pub path: String,
    pub ext: String,
    pub lines: usize,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub complexity: f64,
    pub size_bytes: u64,
    #[serde(skip)]
    pub created_at: Option<std::time::SystemTime>,
    #[serde(skip)]
    pub modified_at: Option<std::time::SystemTime>,
}

pub fn print_compare_table(agg1: &HashMap<String, ExtStats>, agg2: &HashMap<String, ExtStats>) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Extension").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files (Dir 1)").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files (Dir 2)").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files Diff").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines (Dir 1)").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines (Dir 2)").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines Diff").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    let mut all_exts: std::collections::HashSet<String> = std::collections::HashSet::new();
    for ext in agg1.keys() {
        all_exts.insert(ext.clone());
    }
    for ext in agg2.keys() {
        all_exts.insert(ext.clone());
    }

    let mut sorted_exts: Vec<String> = all_exts.into_iter().collect();
    sorted_exts.sort();

    let mut total_files1 = 0;
    let mut total_files2 = 0;
    let mut total_lines1 = 0;
    let mut total_lines2 = 0;

    for ext in sorted_exts {
        let s1 = agg1.get(&ext);
        let s2 = agg2.get(&ext);

        let f1 = s1.map(|s| s.file_count).unwrap_or(0);
        let f2 = s2.map(|s| s.file_count).unwrap_or(0);
        let l1 = s1.map(|s| s.lines).unwrap_or(0);
        let l2 = s2.map(|s| s.lines).unwrap_or(0);

        let f_diff = f2 as isize - f1 as isize;
        let l_diff = l2 as isize - l1 as isize;

        let f_diff_str = if f_diff > 0 { format!("+{}", f_diff) } else { f_diff.to_string() };
        let l_diff_str = if l_diff > 0 { format!("+{}", l_diff) } else { l_diff.to_string() };

        let f_diff_cell = if f_diff > 0 { Cell::new(f_diff_str).fg(Color::Red) } else if f_diff < 0 { Cell::new(f_diff_str).fg(Color::Green) } else { Cell::new(f_diff_str) };
        let l_diff_cell = if l_diff > 0 { Cell::new(l_diff_str).fg(Color::Red) } else if l_diff < 0 { Cell::new(l_diff_str).fg(Color::Green) } else { Cell::new(l_diff_str) };

        table.add_row(vec![
            Cell::new(&ext).fg(Color::Green),
            Cell::new(f1).set_alignment(CellAlignment::Right),
            Cell::new(f2).set_alignment(CellAlignment::Right),
            f_diff_cell.set_alignment(CellAlignment::Right),
            Cell::new(l1).set_alignment(CellAlignment::Right),
            Cell::new(l2).set_alignment(CellAlignment::Right),
            l_diff_cell.set_alignment(CellAlignment::Right),
        ]);

        total_files1 += f1;
        total_files2 += f2;
        total_lines1 += l1;
        total_lines2 += l2;
    }

    let f_diff_total = total_files2 as isize - total_files1 as isize;
    let l_diff_total = total_lines2 as isize - total_lines1 as isize;
    
    let f_diff_total_str = if f_diff_total > 0 { format!("+{}", f_diff_total) } else { f_diff_total.to_string() };
    let l_diff_total_str = if l_diff_total > 0 { format!("+{}", l_diff_total) } else { l_diff_total.to_string() };

    let f_diff_total_cell = if f_diff_total > 0 { Cell::new(f_diff_total_str).fg(Color::Red) } else if f_diff_total < 0 { Cell::new(f_diff_total_str).fg(Color::Green) } else { Cell::new(f_diff_total_str) };
    let l_diff_total_cell = if l_diff_total > 0 { Cell::new(l_diff_total_str).fg(Color::Red) } else if l_diff_total < 0 { Cell::new(l_diff_total_str).fg(Color::Green) } else { Cell::new(l_diff_total_str) };

    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
        Cell::new(total_files1).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_files2).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        f_diff_total_cell.add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right),
        Cell::new(total_lines1).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_lines2).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        l_diff_total_cell.add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right),
    ]);

    println!("\n=== Directory Comparison ===");
    println!("{table}");
}

pub fn print_details_table(stats: &[UnifiedStats], top: Option<usize>, failures_only: bool, show_size: bool, show_age: bool, complexity_threshold: f64) {
    let mut table = Table::new();
    let mut headers = vec![
        Cell::new("Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Extension").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Code").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Comments").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Blanks").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Complexity").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ];
    
    if show_size {
        headers.push(Cell::new("Size (B)").add_attribute(Attribute::Bold).fg(Color::Cyan));
    }
    if show_age {
        headers.push(Cell::new("Modified").add_attribute(Attribute::Bold).fg(Color::Cyan));
        headers.push(Cell::new("Created").add_attribute(Attribute::Bold).fg(Color::Cyan));
    }

    table.set_header(headers);

    let mut filtered_stats: Vec<&UnifiedStats> = stats.iter().filter(|s| {
        if failures_only {
            s.lines > 300 || s.complexity >= complexity_threshold
        } else {
            true
        }
    }).collect();

    filtered_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    if let Some(n) = top {
        filtered_stats.truncate(n);
    }

    for s in filtered_stats {
        let mut row = vec![
            Cell::new(&s.path),
            Cell::new(&s.ext).fg(Color::Green),
            Cell::new(s.lines).set_alignment(CellAlignment::Right),
            Cell::new(s.code).set_alignment(CellAlignment::Right),
            Cell::new(s.comments).set_alignment(CellAlignment::Right),
            Cell::new(s.blanks).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.1}", s.complexity)).set_alignment(CellAlignment::Right),
        ];

        if show_size {
            row.push(Cell::new(s.size_bytes).set_alignment(CellAlignment::Right));
        }
        if show_age {
            let format_time = |t: Option<std::time::SystemTime>| {
                if let Some(time) = t {
                    if let Ok(duration) = time.elapsed() {
                        let secs = duration.as_secs();
                        if secs < 60 {
                            format!("{}s ago", secs)
                        } else if secs < 3600 {
                            format!("{}m ago", secs / 60)
                        } else if secs < 86400 {
                            format!("{}h ago", secs / 3600)
                        } else {
                            format!("{}d ago", secs / 86400)
                        }
                    } else {
                        "-".to_string()
                    }
                } else {
                    "-".to_string()
                }
            };
            row.push(Cell::new(format_time(s.modified_at)).set_alignment(CellAlignment::Right));
            row.push(Cell::new(format_time(s.created_at)).set_alignment(CellAlignment::Right));
        }

        table.add_row(row);
    }

    println!("\n{table}");
}

pub fn print_extensions_list(stats: &HashMap<String, ExtStats>) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Extension").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Files %").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines %").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let total_files: usize = sorted_stats.iter().map(|s| s.file_count).sum();
    let total_lines: usize = sorted_stats.iter().map(|s| s.lines).sum();

    for s in sorted_stats {
        let files_pct = if total_files > 0 { (s.file_count as f64 / total_files as f64) * 100.0 } else { 0.0 };
        let lines_pct = if total_lines > 0 { (s.lines as f64 / total_lines as f64) * 100.0 } else { 0.0 };
        
        table.add_row(vec![
            Cell::new(&s.ext).fg(Color::Green),
            Cell::new(s.file_count).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.1}%", files_pct)).set_alignment(CellAlignment::Right),
            Cell::new(s.lines).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.1}%", lines_pct)).set_alignment(CellAlignment::Right),
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

pub fn print_contributors_detail(stats: &[crate::git::ContributorDetail]) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Contributor").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Commits").add_attribute(Attribute::Bold).fg(Color::Green),
        Cell::new("Insertions").add_attribute(Attribute::Bold).fg(Color::Green),
        Cell::new("Deletions").add_attribute(Attribute::Bold).fg(Color::Red),
        Cell::new("Files Changed").add_attribute(Attribute::Bold).fg(Color::Yellow),
    ]);

    let mut total_commits = 0;
    let mut total_insertions = 0;
    let mut total_deletions = 0;

    for s in stats {
        table.add_row(vec![
            Cell::new(&s.name),
            Cell::new(s.commits).set_alignment(CellAlignment::Right),
            Cell::new(s.insertions).set_alignment(CellAlignment::Right),
            Cell::new(s.deletions).set_alignment(CellAlignment::Right),
            Cell::new(s.files_changed.len()).set_alignment(CellAlignment::Right),
        ]);
        total_commits += s.commits;
        total_insertions += s.insertions;
        total_deletions += s.deletions;
    }

    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
        Cell::new(total_commits).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_insertions).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new(total_deletions).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        Cell::new("-").set_alignment(CellAlignment::Right).fg(Color::Yellow),
    ]);

    println!("\n=== Detailed Contributor Statistics ===");
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

pub fn print_apidoc_stats(stats: &[crate::analyzer::AnalyzerStats]) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Docstrings").add_attribute(Attribute::Bold).fg(Color::Green),
    ]);

    let mut total_docstrings = 0;
    for s in stats {
        if s.docstrings_count > 0 {
            table.add_row(vec![
                Cell::new(s.path.to_string_lossy()),
                Cell::new(s.docstrings_count).set_alignment(CellAlignment::Right),
            ]);
        }
        total_docstrings += s.docstrings_count;
    }

    if total_docstrings > 0 {
        table.add_row(vec![
            Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
            Cell::new(total_docstrings).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        ]);
        println!("\n=== API Documentation Stats ===\n{table}");
    } else {
        println!("\n=== API Documentation Stats ===\nNo docstrings found.");
    }
}

pub fn print_typestats(stats: &[crate::analyzer::AnalyzerStats]) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Type Hints").add_attribute(Attribute::Bold).fg(Color::Green),
    ]);

    let mut total_typehints = 0;
    for s in stats {
        if s.typehints_count > 0 {
            table.add_row(vec![
                Cell::new(s.path.to_string_lossy()),
                Cell::new(s.typehints_count).set_alignment(CellAlignment::Right),
            ]);
        }
        total_typehints += s.typehints_count;
    }

    if total_typehints > 0 {
        table.add_row(vec![
            Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
            Cell::new(total_typehints).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
        ]);
        println!("\n=== Type Hint Stats ===\n{table}");
    } else {
        println!("\n=== Type Hint Stats ===\nNo type hints found.");
    }
}

pub fn print_refactor_map(details: &[UnifiedStats], complexity_threshold: f64) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Complexity").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Reason").add_attribute(Attribute::Bold).fg(Color::Yellow),
    ]);

    let mut found = false;
    for s in details {
        let mut reasons = Vec::new();
        if s.lines > 300 {
            reasons.push("High lines");
        }
        if s.complexity >= complexity_threshold {
            reasons.push("High complexity");
        }

        if !reasons.is_empty() {
            table.add_row(vec![
                Cell::new(&s.path),
                Cell::new(s.lines).set_alignment(CellAlignment::Right),
                Cell::new(format!("{:.1}", s.complexity)).set_alignment(CellAlignment::Right),
                Cell::new(reasons.join(", ")),
            ]);
            found = true;
        }
    }

    if found {
        println!("\n=== Refactor Candidates Map ===\n{table}");
    } else {
        println!("\n=== Refactor Candidates Map ===\nNo refactor candidates found.");
    }
}

pub fn print_refactor_suggest(details: &[UnifiedStats], complexity_threshold: f64) {
    let mut found = false;
    println!("\n=== Refactor Suggestions ===");
    for s in details {
        if s.lines > 300 || s.complexity >= complexity_threshold {
            found = true;
            println!("- {}:", s.path);
            if s.lines > 300 {
                println!("  * Has {} lines (threshold: 300). Consider breaking it into smaller files.", s.lines);
            }
            if s.complexity >= complexity_threshold {
                println!("  * Has high complexity ({:.1} >= {:.1}). Consider simplifying logic or extracting functions.", s.complexity, complexity_threshold);
            }
        }
    }
    if !found {
        println!("No refactoring suggestions at this time.");
    }
}

pub fn print_autofix_suggest(analysis_stats: &[crate::analyzer::AnalyzerStats]) {
    let mut found = false;
    println!("\n=== Auto-fix Suggestions ===");
    for a in analysis_stats {
        if a.naming_violations > 0 {
            found = true;
            println!("- {}:", a.path.to_string_lossy());
            for v in &a.naming_violations_details {
                // E.g. "Function 'FooBar' in ... should be snake_case"
                // We extract the name and suggest a fix.
                let parts: Vec<&str> = v.split('\'').collect();
                if parts.len() >= 3 {
                    let name = parts[1];
                    if v.contains("snake_case") {
                        // Very simple mock conversion to snake_case
                        let mut snake = String::new();
                        for (i, c) in name.chars().enumerate() {
                            if c.is_ascii_uppercase() {
                                if i > 0 {
                                    snake.push('_');
                                }
                                snake.push(c.to_ascii_lowercase());
                            } else {
                                snake.push(c);
                            }
                        }
                        println!("  * Suggestion: Rename '{}' to '{}'", name, snake);
                    } else if v.contains("PascalCase") {
                        // Very simple mock conversion to PascalCase
                        let mut pascal = String::new();
                        let mut capitalize_next = true;
                        for c in name.chars() {
                            if c == '_' {
                                capitalize_next = true;
                            } else if capitalize_next {
                                pascal.push(c.to_ascii_uppercase());
                                capitalize_next = false;
                            } else {
                                pascal.push(c);
                            }
                        }
                        println!("  * Suggestion: Rename '{}' to '{}'", name, pascal);
                    } else {
                        println!("  * Suggestion for: {}", v);
                    }
                } else {
                    println!("  * Suggestion for: {}", v);
                }
            }
        }
    }
    if !found {
        println!("No auto-fix suggestions at this time.");
    }
}

pub fn print_open(path: &str, stats: Option<&ExtStats>, detail: Option<&UnifiedStats>, analysis: Option<&crate::analyzer::AnalyzerStats>) {
    println!("\n=== File Analysis: {} ===", path);
    if let Some(s) = detail {
        println!("Lines: {}", s.lines);
        println!("Code: {}", s.code);
        println!("Comments: {}", s.comments);
        println!("Blanks: {}", s.blanks);
        println!("Size (Bytes): {}", s.size_bytes);
        if let Some(ext) = stats {
            println!("Language Extension: {}", ext.ext);
        }
    } else {
        println!("File basic stats not found.");
    }

    if let Some(a) = analysis {
        println!("Complexity: {:.1}", a.complexity);
        println!("Functions: {}", a.functions_count);
        println!("TODOs: {}", a.todo_count);
        println!("Naming Violations: {}", a.naming_violations);
        for v in &a.naming_violations_details {
            println!("  - {}", v);
        }
        println!("Docstrings: {}", a.docstrings_count);
        println!("Type Hints: {}", a.typehints_count);
    } else {
        println!("File analysis stats not found.");
    }
}

pub fn print_file_blame(file_path: &str, blame_stats: &HashMap<String, usize>) {
    println!("\n=== Git Blame Stats: {} ===", file_path);
    if blame_stats.is_empty() {
        println!("No blame statistics available.");
        return;
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Author").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines Owned").add_attribute(Attribute::Bold).fg(Color::Green),
    ]);

    let mut sorted_stats: Vec<(&String, &usize)> = blame_stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

    let mut total_lines = 0;
    for (author, lines) in sorted_stats {
        table.add_row(vec![
            Cell::new(author),
            Cell::new(lines).set_alignment(CellAlignment::Right),
        ]);
        total_lines += lines;
    }

    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold).fg(Color::Yellow),
        Cell::new(total_lines).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right).fg(Color::Yellow),
    ]);
    
    println!("{table}");
}

pub fn print_file_trend(file_path: Option<&str>, trend: &[(String, usize)]) {
    let target = file_path.unwrap_or("Project Total");
    println!("\n=== Line Count Trend (Last {} Commits): {} ===", trend.len(), target);
    
    if trend.is_empty() {
        println!("No trend data available.");
        return;
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Date/Time").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Lines").add_attribute(Attribute::Bold).fg(Color::Green),
    ]);

    for (date, lines) in trend {
        table.add_row(vec![
            Cell::new(date),
            Cell::new(lines).set_alignment(CellAlignment::Right),
        ]);
    }
    
    println!("{table}");
}

pub fn print_structure_mermaid(details: &[UnifiedStats]) {
    println!("\n=== Mermaid Directory Structure ===");
    println!("```mermaid");
    println!("graph TD;");
    println!("    root[Project Root];");
    
    let mut dirs = std::collections::HashSet::new();
    let mut files = Vec::new();
    
    for d in details {
        let path = std::path::Path::new(&d.path);
        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                files.push(("root".to_string(), path.file_name().unwrap_or_default().to_string_lossy().into_owned()));
            } else {
                let mut current = String::new();
                for comp in parent.components() {
                    let comp_str = comp.as_os_str().to_string_lossy().into_owned();
                    let next = if current.is_empty() { comp_str.clone() } else { format!("{}/{}", current, comp_str) };
                    
                    if current.is_empty() {
                        dirs.insert(("root".to_string(), next.clone(), comp_str));
                    } else {
                        dirs.insert((current.clone(), next.clone(), comp_str));
                    }
                    current = next;
                }
                files.push((current, path.file_name().unwrap_or_default().to_string_lossy().into_owned()));
            }
        }
    }
    
    let mut sorted_dirs: Vec<_> = dirs.into_iter().collect();
    sorted_dirs.sort();
    
    for (parent, full, name) in sorted_dirs {
        let parent_id = parent.replace("/", "_").replace(".", "_").replace("-", "_").replace("\\", "_");
        let full_id = full.replace("/", "_").replace(".", "_").replace("-", "_").replace("\\", "_");
        println!("    {} --> {}[{}];", parent_id, full_id, name);
    }
    
    for (parent, name) in files {
        let parent_id = parent.replace("/", "_").replace(".", "_").replace("-", "_").replace("\\", "_");
        let file_id = format!("{}_{}", parent_id, name).replace(".", "_").replace("-", "_").replace("\\", "_");
        println!("    {} --> {}[{}]:::file;", parent_id, file_id, name);
    }
    
    println!("    classDef file fill:#f9f,stroke:#333,stroke-width:1px;");
    println!("```");
}

pub fn print_complexitymap(details: &[UnifiedStats]) {
    println!("\n=== Complexity Heatmap ===");
    println!("Legend: . (<2)   o (<5)   O (<10)   @ (>=10)");
    
    let mut sorted_details: Vec<&UnifiedStats> = details.iter().collect();
    sorted_details.sort_by(|a, b| a.path.cmp(&b.path));
    
    for d in sorted_details {
        let symbol = if d.complexity < 2.0 {
            "."
        } else if d.complexity < 5.0 {
            "o"
        } else if d.complexity < 10.0 {
            "O"
        } else {
            "@"
        };
        
        println!("{} {}", symbol, d.path);
    }
}

pub fn generate_excel(stats: &HashMap<String, ExtStats>, details: Option<&[UnifiedStats]>, output: Option<&String>) {
    let mut workbook = Workbook::new();
    let header_format = Format::new().set_bold().set_font_color(XlsxColor::White).set_background_color(XlsxColor::Blue);
    
    // Summary Sheet
    let sheet = workbook.add_worksheet().set_name("Summary").unwrap();
    sheet.write_string_with_format(0, 0, "Extension", &header_format).unwrap();
    sheet.write_string_with_format(0, 1, "Files", &header_format).unwrap();
    sheet.write_string_with_format(0, 2, "Lines", &header_format).unwrap();
    sheet.write_string_with_format(0, 3, "Code", &header_format).unwrap();
    sheet.write_string_with_format(0, 4, "Comments", &header_format).unwrap();
    sheet.write_string_with_format(0, 5, "Blanks", &header_format).unwrap();
    
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    for (row, s) in sorted_stats.iter().enumerate() {
        let r = (row + 1) as u32;
        sheet.write_string(r, 0, &s.ext).unwrap();
        sheet.write_number(r, 1, s.file_count as f64).unwrap();
        sheet.write_number(r, 2, s.lines as f64).unwrap();
        sheet.write_number(r, 3, s.code_lines as f64).unwrap();
        sheet.write_number(r, 4, s.comment_lines as f64).unwrap();
        sheet.write_number(r, 5, s.blank_lines as f64).unwrap();
    }
    
    // Details Sheet
    if let Some(details) = details {
        let sheet = workbook.add_worksheet().set_name("Details").unwrap();
        sheet.write_string_with_format(0, 0, "Path", &header_format).unwrap();
        sheet.write_string_with_format(0, 1, "Extension", &header_format).unwrap();
        sheet.write_string_with_format(0, 2, "Lines", &header_format).unwrap();
        sheet.write_string_with_format(0, 3, "Code", &header_format).unwrap();
        sheet.write_string_with_format(0, 4, "Comments", &header_format).unwrap();
        sheet.write_string_with_format(0, 5, "Blanks", &header_format).unwrap();
        sheet.write_string_with_format(0, 6, "Complexity", &header_format).unwrap();
        
        for (row, s) in details.iter().enumerate() {
            let r = (row + 1) as u32;
            sheet.write_string(r, 0, &s.path).unwrap();
            sheet.write_string(r, 1, &s.ext).unwrap();
            sheet.write_number(r, 2, s.lines as f64).unwrap();
            sheet.write_number(r, 3, s.code as f64).unwrap();
            sheet.write_number(r, 4, s.comments as f64).unwrap();
            sheet.write_number(r, 5, s.blanks as f64).unwrap();
            sheet.write_number(r, 6, s.complexity).unwrap();
        }
    }
    
    let path = output.map(|s| s.as_str()).unwrap_or("output/codestate_report.xlsx");
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = workbook.save(path) {
        eprintln!("! Failed to write Excel file {}: {}", path, e);
    } else {
        println!("✓ Excel report successfully saved to {}", path);
    }
}

pub fn generate_details_csv(details: &[UnifiedStats]) -> String {
    let mut csv = String::new();
    csv.push_str("Path,Extension,Lines,Code,Comments,Blanks,Complexity\n");
    for s in details {
        csv.push_str(&format!("{},{},{},{},{},{},{:.1}\n",
            s.path.replace(",", ";"), s.ext, s.lines, s.code, s.comments, s.blanks, s.complexity));
    }
    csv
}

pub fn generate_groupdir_csv(details: &[UnifiedStats]) -> String {
    let mut dir_stats: HashMap<String, (usize, usize, usize, usize)> = HashMap::new();
    for s in details {
        let path = std::path::Path::new(&s.path);
        let dir = path.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| ".".to_string());
        let dir = if dir.is_empty() { ".".to_string() } else { dir };
        
        let entry = dir_stats.entry(dir).or_insert((0, 0, 0, 0));
        entry.0 += 1; // files
        entry.1 += s.lines;
        entry.2 += s.code;
        entry.3 += s.comments;
    }
    
    let mut csv = String::new();
    csv.push_str("Directory,Files,Lines,Code,Comments\n");
    let mut sorted_dirs: Vec<_> = dir_stats.into_iter().collect();
    sorted_dirs.sort_by(|a, b| a.0.cmp(&b.0));
    
    for (dir, (files, lines, code, comments)) in sorted_dirs {
        csv.push_str(&format!("{},{},{},{},{}\n", dir.replace(",", ";"), files, lines, code, comments));
    }
    csv
}

#[derive(serde::Serialize)]
pub struct IssueReport {
    pub path: String,
    pub issue_type: String,
    pub description: String,
}

pub fn print_report_issues(details: &[UnifiedStats], analysis_stats: &[crate::analyzer::AnalyzerStats], complexity_threshold: f64, format_json: bool, output: Option<&String>) {
    let mut issues = Vec::new();
    
    for s in details {
        if s.lines > 300 {
            issues.push(IssueReport {
                path: s.path.clone(),
                issue_type: "Large File".to_string(),
                description: format!("File has {} lines (threshold: 300)", s.lines),
            });
        }
        if s.complexity >= complexity_threshold {
            issues.push(IssueReport {
                path: s.path.clone(),
                issue_type: "High Complexity".to_string(),
                description: format!("Complexity is {:.1} (threshold: {:.1})", s.complexity, complexity_threshold),
            });
        }
    }
    
    for a in analysis_stats {
        if a.naming_violations > 0 {
            for v in &a.naming_violations_details {
                issues.push(IssueReport {
                    path: a.path.to_string_lossy().into_owned(),
                    issue_type: "Naming Violation".to_string(),
                    description: v.clone(),
                });
            }
        }
    }
    
    if format_json {
        let json = serde_json::to_string_pretty(&issues).unwrap_or_else(|_| "[]".to_string());
        save_or_print(&json, output);
    } else {
        let mut md = String::new();
        md.push_str("## CodeState Issues Report\n\n");
        if issues.is_empty() {
            md.push_str("No issues found! \u{1F389}\n");
        } else {
            md.push_str("| Path | Issue Type | Description |\n");
            md.push_str("|------|------------|-------------|\n");
            for i in &issues {
                md.push_str(&format!("| {} | {} | {} |\n", i.path, i.issue_type, i.description));
            }
        }
        save_or_print(&md, output);
    }
}

pub fn generate_badge_sustainability(details: &[UnifiedStats]) -> String {
    let total_complexity: f64 = details.iter().map(|d| d.complexity).sum();
    let avg_complexity = if !details.is_empty() { total_complexity / details.len() as f64 } else { 0.0 };
    
    let color = if avg_complexity < 3.0 {
        "brightgreen"
    } else if avg_complexity < 5.0 {
        "yellow"
    } else {
        "red"
    };
    
    let color_hex = if color == "brightgreen" { "#4c1" } else if color == "yellow" { "#dfb317" } else { "#e05d44" };
    
    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="20">
  <linearGradient id="b" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <mask id="a">
    <rect width="140" height="20" rx="3" fill="#fff"/>
  </mask>
  <g mask="url(#a)">
    <path fill="#555" d="M0 0h80v20H0z"/>
    <path fill="{color_hex}" d="M80 0h60v20H80z"/>
    <path fill="url(#b)" d="M0 0h140v20H0z"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11">
    <text x="40" y="15" fill="#010101" fill-opacity=".3">sustainability</text>
    <text x="40" y="14">sustainability</text>
    <text x="110" y="15" fill="#010101" fill-opacity=".3">{score:.1}</text>
    <text x="110" y="14">{score:.1}</text>
  </g>
</svg>"##, color_hex = color_hex, score = avg_complexity)
}

pub fn generate_lang_card_svg(stats: &HashMap<String, ExtStats>) -> String {
    let mut sorted_stats: Vec<&ExtStats> = stats.values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    let primary_lang = sorted_stats.iter()
        .find(|s| !s.ext.trim().is_empty())
        .map_or("Unknown", |s| s.ext.as_str());
        
    let total_lines: usize = stats.values().map(|s| s.lines).sum();
    
    format!(r##"<svg width="300" height="150" xmlns="http://www.w3.org/2000/svg">
  <rect width="100%" height="100%" fill="#282c34" rx="10" />
  <text x="20" y="40" font-family="Arial" font-size="20" fill="#61dafb" font-weight="bold">CodeState Stats</text>
  <text x="20" y="80" font-family="Arial" font-size="16" fill="#abb2bf">Primary Lang: {}</text>
  <text x="20" y="110" font-family="Arial" font-size="16" fill="#abb2bf">Total Lines: {}</text>
</svg>"##, primary_lang, total_lines)
}

