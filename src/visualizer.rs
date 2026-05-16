use comfy_table::{Table, Cell, Color, Attribute, CellAlignment};
use crate::scanner::ExtStats;
use std::collections::HashMap;

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
