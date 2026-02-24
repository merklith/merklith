//! UI rendering for the TUI block explorer.

use super::{App, AppState, View};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block as TuiBlock, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
};

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());
    
    // Draw header
    draw_header(frame, app, main_layout[0]);
    
    // Draw main content based on current view
    match app.current_view {
        View::Blocks => draw_blocks_view(frame, app, main_layout[1]),
        View::BlockDetail => draw_block_detail(frame, app, main_layout[1]),
        View::Transactions => draw_transactions_view(frame, app, main_layout[1]),
        View::Accounts => draw_accounts_view(frame, app, main_layout[1]),
        View::Search => draw_search_view(frame, app, main_layout[1]),
        View::Help => draw_help_view(frame, app, main_layout[1]),
        _ => draw_blocks_view(frame, app, main_layout[1]),
    }
    
    // Draw footer
    draw_footer(frame, app, main_layout[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let header_block = TuiBlock::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let connection_status = if app.connected {
        Span::styled("● Connected", Style::default().fg(Color::Green))
    } else {
        Span::styled("● Disconnected", Style::default().fg(Color::Red))
    };
    
    let header_text = Text::from(vec![
        Line::from(vec![
            Span::styled(" Merklith Block Explorer ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(format!("Chain ID: {}", app.chain_id), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Block: {}", app.latest_block), Style::default().fg(Color::Green)),
            Span::raw(" | "),
            connection_status,
        ]),
    ]);
    
    let header = Paragraph::new(header_text)
        .block(header_block)
        .alignment(Alignment::Left);
    
    frame.render_widget(header, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let footer_block = TuiBlock::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let help_text = match app.current_view {
        View::Help => " Press 'q' to close help ",
        View::Search => " Enter: Search | Esc: Cancel ",
        View::BlockDetail => " ←: Back | j/k: Scroll ",
        _ => " q: Quit | h: Help | r: Refresh | b: Blocks | t: TXs | a: Accounts | s: Search | ↑↓: Navigate | Enter: Select ",
    };
    
    let footer = Paragraph::new(help_text)
        .block(footer_block)
        .alignment(Alignment::Center);
    
    frame.render_widget(footer, area);
}

fn draw_blocks_view(frame: &mut Frame, app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Recent Blocks ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    if app.blocks.is_empty() {
        let text = Paragraph::new("No blocks loaded. Press 'r' to refresh.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
        return;
    }
    
    let header = Row::new(vec!["Block #", "Hash", "Timestamp", "TXs", "Proposer"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1);
    
    let rows: Vec<Row> = app.blocks.iter().enumerate().map(|(i, block)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };
        
        Row::new(vec![
            block.number.to_string(),
            block.hash.clone(),
            format_timestamp(block.timestamp),
            block.tx_count.to_string(),
            block.proposer.clone(),
        ])
        .style(style)
    }).collect();
    
    let table = Table::new(rows, vec![
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Length(12),
        Constraint::Length(6),
        Constraint::Min(20),
    ])
    .header(header)
    .block(block)
    .highlight_spacing(HighlightSpacing::Always);
    
    frame.render_widget(table, area);
}

fn draw_block_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Block Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    if let Some(block_data) = &app.selected_block {
        // Pretty print JSON
        let json_str = serde_json::to_string_pretty(block_data).unwrap_or_default();
        let text = Paragraph::new(json_str)
            .block(block)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(text, area);
    } else {
        let text = Paragraph::new("No block selected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    }
}

fn draw_transactions_view(frame: &mut Frame, app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Recent Transactions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    if app.transactions.is_empty() {
        let text = Paragraph::new("No transactions loaded. Press 'r' to refresh.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
        return;
    }
    
    let header = Row::new(vec!["Hash", "From", "To", "Value (MERK)", "Nonce"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    let rows: Vec<Row> = app.transactions.iter().enumerate().map(|(i, tx)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };
        
        let value_merk = format!("{:.6}", tx.value.to_f64_lossy() / 1e18);
        
        Row::new(vec![
            tx.hash.clone(),
            tx.from.clone(),
            tx.to.clone().unwrap_or_else(|| "Contract Creation".to_string()),
            value_merk,
            tx.nonce.to_string(),
        ])
        .style(style)
    }).collect();
    
    let table = Table::new(rows, vec![
        Constraint::Length(18),
        Constraint::Length(15),
        Constraint::Length(20),
        Constraint::Length(15),
        Constraint::Length(8),
    ])
    .header(header)
    .block(block);
    
    frame.render_widget(table, area);
}

fn draw_accounts_view(frame: &mut Frame, _app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Accounts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    let text = Paragraph::new("Account view not yet implemented. Press 'b' to go back to blocks.")
        .block(block)
        .alignment(Alignment::Center);
    
    frame.render_widget(text, area);
}

fn draw_search_view(frame: &mut Frame, app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let text = Text::from(vec![
        Line::from("Enter block number, hash, or address:"),
        Line::from(""),
        Line::from(Span::styled(&app.search_query, 
            if app.search_mode { 
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) 
            } else { 
                Style::default() 
            }
        )),
    ]);
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    
    frame.render_widget(paragraph, area);
}

fn draw_help_view(frame: &mut Frame, _app: &App, area: Rect) {
    let block = TuiBlock::default()
        .title(" Help - Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let text = Text::from(vec![
        Line::from(vec![Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("  ↑ / k    - Move up"),
        Line::from("  ↓ / j    - Move down"),
        Line::from("  ← / h    - Previous page"),
        Line::from("  → / l    - Next page"),
        Line::from("  Enter    - Select item"),
        Line::from("  Backspace- Go back"),
        Line::from(""),
        Line::from(vec![Span::styled("Views", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("  b        - Blocks view"),
        Line::from("  t        - Transactions view"),
        Line::from("  a        - Accounts view"),
        Line::from("  s        - Search"),
        Line::from(""),
        Line::from(vec![Span::styled("Actions", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("  r        - Refresh data"),
        Line::from("  h        - Show this help"),
        Line::from("  q / Esc  - Quit"),
    ]);
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{UNIX_EPOCH, SystemTime};
    
    let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    let now = SystemTime::now();
    
    if let Ok(duration) = now.duration_since(datetime) {
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
        format!("{}", timestamp)
    }
}