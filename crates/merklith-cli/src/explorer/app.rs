//! Application state and logic for the TUI block explorer.

use crate::rpc_client::RpcClient;
use merklith_types::{Address, U256, Hash};
use std::collections::VecDeque;

/// Current application view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Blocks,
    BlockDetail,
    Transactions,
    TransactionDetail,
    Accounts,
    AccountDetail,
    Search,
    Help,
}

/// Application state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Running,
    Loading,
    Error,
    Quitting,
}

/// Block summary for display
#[derive(Debug, Clone)]
pub struct BlockSummary {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub tx_count: usize,
    pub proposer: String,
}

/// Transaction summary for display
#[derive(Debug, Clone)]
pub struct TransactionSummary {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: U256,
    pub nonce: u64,
}

/// Account summary for display
#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub address: String,
    pub balance: U256,
    pub nonce: u64,
}

/// Main application struct
pub struct App {
    pub state: AppState,
    pub current_view: View,
    pub previous_view: Option<View>,
    
    // RPC Client
    pub client: RpcClient,
    
    // Data
    pub blocks: Vec<BlockSummary>,
    pub transactions: Vec<TransactionSummary>,
    pub accounts: Vec<AccountSummary>,
    pub selected_block: Option<serde_json::Value>,
    pub selected_transaction: Option<serde_json::Value>,
    pub selected_account: Option<AccountSummary>,
    
    // Navigation
    pub selected_index: usize,
    pub block_table_state: ratatui::widgets::TableState,
    pub tx_table_state: ratatui::widgets::TableState,
    pub account_table_state: ratatui::widgets::TableState,
    pub scroll_offset: usize,
    
    // Search
    pub search_query: String,
    pub search_mode: bool,
    
    // Stats
    pub chain_id: u64,
    pub latest_block: u64,
    pub connected: bool,
    pub last_error: Option<String>,
    
    // View history for navigation
    view_stack: Vec<View>,
}

impl App {
    pub fn new(client: RpcClient) -> Self {
        Self {
            state: AppState::Running,
            current_view: View::Blocks,
            previous_view: None,
            client,
            blocks: Vec::new(),
            transactions: Vec::new(),
            accounts: Vec::new(),
            selected_block: None,
            selected_transaction: None,
            selected_account: None,
            selected_index: 0,
            block_table_state: ratatui::widgets::TableState::default(),
            tx_table_state: ratatui::widgets::TableState::default(),
            account_table_state: ratatui::widgets::TableState::default(),
            scroll_offset: 0,
            search_query: String::new(),
            search_mode: false,
            chain_id: 0,
            latest_block: 0,
            connected: false,
            last_error: None,
            view_stack: Vec::new(),
        }
    }
    
    pub async fn load_initial_data(&mut self) -> anyhow::Result<()> {
        self.state = AppState::Loading;
        
        // Try to connect and load basic info
        match self.client.chain_id().await {
            Ok(chain_id) => {
                self.chain_id = chain_id;
                self.connected = true;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to connect: {}", e));
                self.connected = false;
            }
        }
        
        if self.connected {
            self.refresh_data().await?;
        }
        
        self.state = AppState::Running;
        Ok(())
    }
    
    pub async fn refresh_data(&mut self) -> anyhow::Result<()> {
        if !self.connected {
            return Ok(());
        }
        
        // Load latest block number
        match self.client.block_number().await {
            Ok(number) => self.latest_block = number,
            Err(e) => {
                self.last_error = Some(format!("Failed to get block number: {}", e));
                return Ok(());
            }
        }
        
        // Load recent blocks (last 20)
        self.blocks.clear();
        let start = if self.latest_block > 20 { self.latest_block - 20 } else { 0 };
        
        for num in (start..=self.latest_block).rev() {
            if let Ok(Some(block)) = self.client.get_block_by_number(num).await {
                // Extract data from JSON
                let number = block.get("number")
                    .and_then(|v| v.as_str())
                    .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                    .unwrap_or(num);
                
                let hash = block.get("hash")
                    .and_then(|v| v.as_str())
                    .map(|s| format!("{:.12}...", s))
                    .unwrap_or_else(|| "unknown".to_string());
                
                let timestamp = block.get("timestamp")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                
                let tx_count = block.get("transactions")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);
                
                let proposer = block.get("miner")
                    .or_else(|| block.get("proposer"))
                    .and_then(|v| v.as_str())
                    .map(|s| format!("{:.10}...", s))
                    .unwrap_or_else(|| "unknown".to_string());
                
                self.blocks.push(BlockSummary {
                    number,
                    hash,
                    timestamp,
                    tx_count,
                    proposer,
                });
            }
        }
        
        // Load recent transactions from blocks
        self.transactions.clear();
        for block_summary in self.blocks.iter().take(5) {
            if let Ok(Some(block)) = self.client.get_block_by_number(block_summary.number).await {
                if let Some(txs) = block.get("transactions").and_then(|v| v.as_array()) {
                    for tx in txs.iter().take(10) { // Limit to 10 txs per block
                        let hash = tx.get("hash")
                            .and_then(|v| v.as_str())
                            .map(|s| format!("{:.14}...", s))
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        let from = tx.get("from")
                            .and_then(|v| v.as_str())
                            .map(|s| format!("{:.10}...", s))
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        let to = tx.get("to")
                            .and_then(|v| v.as_str())
                            .map(|s| format!("{:.10}...", s));
                        
                        let value_str = tx.get("value")
                            .and_then(|v| v.as_str())
                            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                            .unwrap_or(0);
                        let value = U256::from(value_str);
                        
                        let nonce = tx.get("nonce")
                            .and_then(|v| v.as_u64())
                            .or_else(|| tx.get("nonce").and_then(|v| v.as_str()).and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()))
                            .unwrap_or(0);
                        
                        self.transactions.push(TransactionSummary {
                            hash,
                            from,
                            to,
                            value,
                            nonce,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn set_view(&mut self, view: View) {
        self.view_stack.push(self.current_view);
        self.previous_view = Some(self.current_view);
        self.current_view = view;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
    
    pub fn back(&mut self) {
        if let Some(view) = self.view_stack.pop() {
            self.current_view = view;
            self.selected_index = 0;
        }
    }
    
    pub fn show_help(&mut self) {
        self.set_view(View::Help);
    }
    
    pub fn toggle_search(&mut self) {
        self.search_mode = !self.search_mode;
        if self.search_mode {
            self.set_view(View::Search);
        } else {
            self.back();
        }
    }
    
    pub fn next(&mut self) {
        let max = match self.current_view {
            View::Blocks => self.blocks.len().saturating_sub(1),
            View::Transactions => self.transactions.len().saturating_sub(1),
            View::Accounts => self.accounts.len().saturating_sub(1),
            _ => 0,
        };
        
        if self.selected_index < max {
            self.selected_index += 1;
        }
    }
    
    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn next_page(&mut self) {
        self.scroll_offset += 10;
    }
    
    pub fn previous_page(&mut self) {
        if self.scroll_offset >= 10 {
            self.scroll_offset -= 10;
        } else {
            self.scroll_offset = 0;
        }
    }
    
    pub async fn select(&mut self) -> anyhow::Result<()> {
        match self.current_view {
            View::Blocks => {
                if let Some(block_summary) = self.blocks.get(self.selected_index) {
                    // Load full block details
                    if let Ok(Some(block)) = self.client.get_block_by_number(block_summary.number).await {
                        self.selected_block = Some(block);
                        self.set_view(View::BlockDetail);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    pub async fn on_tick(&mut self) -> anyhow::Result<()> {
        // Auto-refresh every 30 ticks (about 7.5 seconds)
        // This could be made configurable
        Ok(())
    }
}