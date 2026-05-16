use std::io::{self, stdout};
use std::collections::HashMap;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};
use crate::scanner::LangStats;
use crate::visualizer::UnifiedStats;

pub fn run_tui(aggregated: HashMap<String, LangStats>, _details: Vec<UnifiedStats>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut sorted_stats: Vec<LangStats> = aggregated.into_values().collect();
    sorted_stats.sort_by(|a, b| b.lines.cmp(&a.lines));

    let res = run_app(&mut terminal, &mut table_state, &mut sorted_stats);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut TableState,
    items: &mut Vec<LangStats>,
) -> anyhow::Result<()> {
    let mut sort_by = 0; // 0: lines, 1: files, 2: code, 3: comments, 4: blanks, 5: complexity, 6: todos, 7: language

    loop {
        // Sort items based on current sort criteria
        match sort_by {
            0 => items.sort_by(|a, b| b.lines.cmp(&a.lines)),
            1 => items.sort_by(|a, b| b.file_count.cmp(&a.file_count)),
            2 => items.sort_by(|a, b| b.code_lines.cmp(&a.code_lines)),
            3 => items.sort_by(|a, b| b.comment_lines.cmp(&a.comment_lines)),
            4 => items.sort_by(|a, b| b.blank_lines.cmp(&a.blank_lines)),
            5 => items.sort_by(|a, b| b.complexity.partial_cmp(&a.complexity).unwrap_or(std::cmp::Ordering::Equal)),
            6 => items.sort_by(|a, b| b.todo_count.cmp(&a.todo_count)),
            7 => items.sort_by(|a, b| a.language.cmp(&b.language)),
            _ => {}
        }

        terminal.draw(|f| {
            let rects = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
                .split(f.area());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED).fg(Color::Cyan);
            let normal_style = Style::default().bg(Color::Reset);
            
            let headers = [
                ("Language", 7),
                ("Files", 1),
                ("Lines", 0),
                ("Code", 2),
                ("Comments", 3),
                ("Blanks", 4),
                ("Cmplx", 5),
                ("TODOs", 6),
            ];

            let header_cells = headers.iter().map(|(h, idx)| {
                let mut style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                let mut text = h.to_string();
                if sort_by == *idx {
                    style = style.fg(Color::White).add_modifier(Modifier::UNDERLINED);
                    text.push_str(" ▼");
                }
                Cell::from(text).style(style)
            });

            let header = Row::new(header_cells).style(normal_style).height(1).bottom_margin(1);

            let rows = items.iter().map(|item| {
                let height = 1;
                let cells = vec![
                    Cell::from(item.language.clone()).style(Style::default().fg(Color::Green)),
                    Cell::from(item.file_count.to_string()),
                    Cell::from(item.lines.to_string()),
                    Cell::from(item.code_lines.to_string()),
                    Cell::from(item.comment_lines.to_string()),
                    Cell::from(item.blank_lines.to_string()),
                    Cell::from(format!("{:.1}", item.complexity)),
                    Cell::from(item.todo_count.to_string()),
                ];
                Row::new(cells).height(height as u16).bottom_margin(0)
            });

            let t = Table::new(rows, [
                Constraint::Percentage(16),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
            ])
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("CodeState Interactive Explorer"))
            .row_highlight_style(selected_style)
            .highlight_symbol(">> ");

            f.render_stateful_widget(t, rects[0], state);

            let help_text = vec![
                Line::from(vec![
                    Span::styled("Navigation: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Up/Down (k/j)  ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("Sort: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("'s' ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("to cycle sort column  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Quit: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("q/Esc", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ])
            ];
            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help / Controls"));
            f.render_widget(help_paragraph, rects[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('s') => {
                            sort_by = (sort_by + 1) % 8;
                            // Reset selection to top when sorting changes
                            state.select(Some(0));
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i >= items.len().saturating_sub(1) {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = match state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        items.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            state.select(Some(i));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}