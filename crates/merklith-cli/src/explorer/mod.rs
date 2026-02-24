//! TUI Block Explorer for Merklith blockchain.
//!
//! An interactive terminal-based blockchain explorer with real-time updates.

use crate::rpc_client::RpcClient;
use merklith_types::{Block, Transaction, Address, U256, Hash};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block as TuiBlock, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
};
use std::io;

mod app;
mod ui;
mod widgets;

pub use app::{App, AppState, View};

/// Run the TUI block explorer
pub async fn run_explorer(rpc_url: String) -> anyhow::Result<()> {
    // Setup terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;
    
    // Create app
    let client = RpcClient::new(rpc_url);
    let mut app = App::new(client);
    
    // Initial data load
    app.load_initial_data().await?;
    
    // Main loop
    let result = run_app(&mut terminal, &mut app).await;
    
    // Restore terminal
    ratatui::restore();
    
    result
}

async fn run_app<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_millis(250);
    
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;
        
        // Handle events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.state {
                        AppState::Running => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.state = AppState::Quitting;
                                    break;
                                }
                                KeyCode::Char('h') => app.show_help(),
                                KeyCode::Char('r') => app.refresh_data().await?,
                                KeyCode::Char('b') => app.set_view(View::Blocks),
                                KeyCode::Char('t') => app.set_view(View::Transactions),
                                KeyCode::Char('a') => app.set_view(View::Accounts),
                                KeyCode::Char('s') => app.toggle_search(),
                                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                                KeyCode::Down | KeyCode::Char('j') => app.next(),
                                KeyCode::Left | KeyCode::Char('h') => app.previous_page(),
                                KeyCode::Right | KeyCode::Char('l') => app.next_page(),
                                KeyCode::Enter => app.select().await?,
                                KeyCode::Backspace => app.back(),
                                _ => {}
                            }
                        }
                        AppState::Quitting => break,
                        _ => {}
                    }
                }
            }
        }
        
        // Periodic refresh
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = std::time::Instant::now();
        }
    }
    
    Ok(())
}