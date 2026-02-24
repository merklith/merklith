/**
 * MERKLITH Block Explorer - Professional Edition
 * Real-time blockchain explorer with modern UI
 */

const CONFIG = {
    RPC_ENDPOINTS: [
        'http://localhost:8545',
        'http://localhost:8547',
        'http://localhost:8549'
    ],
    UPDATE_INTERVAL: 3000, // 3 seconds
    BLOCKS_PER_PAGE: 25,
    CHAIN_ID: 17001
};

let appState = {
    currentPage: 'home',
    blocks: [],
    stats: null,
    nodes: [],
    lastUpdate: null,
    isLoading: false
};

// Utility Functions
const utils = {
    formatHash: (hash, start = 8, end = 8) => {
        if (!hash || hash.length < start + end + 3) return hash;
        return `${hash.slice(0, start)}...${hash.slice(-end)}`;
    },

    formatTime: (timestamp) => {
        if (!timestamp) return 'N/A';
        const date = new Date(parseInt(timestamp, 16) * 1000);
        const now = new Date();
        const diff = Math.floor((now - date) / 1000);
        
        if (diff < 60) return `${diff}s ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
        return date.toLocaleDateString();
    },

    formatNumber: (num) => {
        if (num === null || num === undefined) return '0';
        return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    },

    formatBalance: (wei) => {
        if (!wei) return '0 ANV';
        const anv = parseInt(wei, 16) / 1e18;
        return `${anv.toFixed(4)} ANV`;
    },

    copyToClipboard: async (text) => {
        try {
            await navigator.clipboard.writeText(text);
            showToast('Copied to clipboard!', 'success');
        } catch (err) {
            showToast('Failed to copy', 'error');
        }
    }
};

// Toast Notification
function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.style.cssText = `
        position: fixed;
        bottom: 2rem;
        right: 2rem;
        padding: 1rem 1.5rem;
        background: ${type === 'success' ? 'rgba(16, 185, 129, 0.9)' : type === 'error' ? 'rgba(239, 68, 68, 0.9)' : 'rgba(0, 212, 255, 0.9)'};
        color: white;
        border-radius: 12px;
        font-weight: 600;
        z-index: 10000;
        animation: slideIn 0.3s ease;
        backdrop-filter: blur(10px);
    `;
    toast.textContent = message;
    document.body.appendChild(toast);
    
    setTimeout(() => {
        toast.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

// RPC Client
const rpcClient = {
    async call(method, params = [], endpoint = CONFIG.RPC_ENDPOINTS[0], timeout = 5000) {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);
        
        try {
            const response = await fetch(endpoint, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method,
                    params,
                    id: Date.now()
                }),
                signal: controller.signal
            });
            
            clearTimeout(timeoutId);
            
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }
            
            const data = await response.json();
            
            if (data.error) {
                throw new Error(data.error.message);
            }
            
            return data.result;
        } catch (error) {
            clearTimeout(timeoutId);
            console.error(`RPC Error (${method}):`, error.message);
            throw error;
        }
    },

    async getChainStats() {
        const stats = await this.call('merklith_getChainStats');
        if (stats) {
            return {
                blockNumber: parseInt(stats.blockNumber, 16),
                accounts: stats.accounts,
                chainId: parseInt(stats.chainId, 16),
                blockHash: stats.blockHash
            };
        }
        return null;
    },

    async getBlock(blockNumber) {
        const hexNum = typeof blockNumber === 'number' 
            ? `0x${blockNumber.toString(16)}` 
            : blockNumber;
        return await this.call('merklith_getBlockByNumber', [hexNum, true]);
    },

    async getLatestBlocks(count = 10) {
        try {
            const stats = await this.getChainStats();
            if (!stats) return [];
            
            const promises = [];
            for (let i = 0; i < count && stats.blockNumber - i >= 0; i++) {
                promises.push(
                    this.getBlock(stats.blockNumber - i).catch(() => null)
                );
            }
            
            const blocks = await Promise.all(promises);
            return blocks.filter(b => b !== null);
        } catch (error) {
            console.error('Failed to fetch blocks:', error);
            return [];
        }
    },

    async getNodeStatus() {
        const nodes = [];
        for (let i = 0; i < CONFIG.RPC_ENDPOINTS.length; i++) {
            const start = Date.now();
            try {
                const blockNum = await this.call('merklith_blockNumber', [], CONFIG.RPC_ENDPOINTS[i], 3000);
                const latency = Date.now() - start;
                nodes.push({
                    id: i + 1,
                    name: `Node ${i + 1}`,
                    endpoint: CONFIG.RPC_ENDPOINTS[i].replace('http://', ''),
                    wsPort: 8546 + (i * 2),
                    blockNumber: parseInt(blockNum, 16),
                    online: true,
                    latency: latency
                });
            } catch (error) {
                nodes.push({
                    id: i + 1,
                    name: `Node ${i + 1}`,
                    endpoint: CONFIG.RPC_ENDPOINTS[i].replace('http://', ''),
                    wsPort: 8546 + (i * 2),
                    blockNumber: null,
                    online: false,
                    latency: null
                });
            }
        }
        return nodes;
    },

    async search(query) {
        const results = [];
        
        // Try as block number
        const blockNum = parseInt(query);
        if (!isNaN(blockNum)) {
            try {
                const block = await this.getBlock(blockNum);
                if (block) {
                    results.push({
                        type: 'block',
                        title: `Block #${blockNum}`,
                        id: blockNum,
                        subtitle: `Hash: ${utils.formatHash(block.hash)}`,
                        data: block
                    });
                }
            } catch (e) {}
        }
        
        // Try as transaction hash (66 chars, starts with 0x)
        if (query.startsWith('0x') && query.length === 66) {
            try {
                const tx = await this.call('merklith_getTransactionByHash', [query]);
                if (tx) {
                    results.push({
                        type: 'transaction',
                        title: `Transaction`,
                        id: tx.hash,
                        subtitle: `From: ${utils.formatHash(tx.from)} → To: ${utils.formatHash(tx.to)}`,
                        data: tx
                    });
                }
            } catch (e) {}
        }
        
        // Try as address (42 chars, starts with 0x)
        if (query.startsWith('0x') && query.length === 42) {
            try {
                const [balance, nonce] = await Promise.all([
                    this.call('merklith_getBalance', [query]).catch(() => '0x0'),
                    this.call('merklith_getNonce', [query]).catch(() => '0x0')
                ]);
                
                results.push({
                    type: 'address',
                    title: `Account`,
                    id: query,
                    subtitle: `Balance: ${utils.formatBalance(balance)}`,
                    data: { address: query, balance, nonce: parseInt(nonce, 16) }
                });
            } catch (e) {}
        }
        
        return results;
    }
};

// Page Renderers
const pages = {
    async home() {
        const [stats, blocks, nodes] = await Promise.all([
            rpcClient.getChainStats(),
            rpcClient.getLatestBlocks(10),
            rpcClient.getNodeStatus()
        ]);
        
        appState.stats = stats;
        appState.blocks = blocks;
        appState.nodes = nodes;
        
        const avgBlockTime = blocks.length > 1 
            ? ((parseInt(blocks[0].timestamp, 16) - parseInt(blocks[blocks.length - 1].timestamp, 16)) / (blocks.length - 1)).toFixed(1)
            : '2.0';
        
        return `
            <div class="page">
                <div class="hero">
                    <h1>MERKLITH Blockchain Explorer</h1>
                    <p>Real-time explorer for the Proof of Contribution blockchain</p>
                </div>

                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-header">
                            <div class="stat-icon">📦</div>
                            <span class="stat-change">+${blocks.length} new</span>
                        </div>
                        <div class="stat-label">Block Height</div>
                        <div class="stat-value" id="stat-block-height">${utils.formatNumber(stats?.blockNumber || 0)}</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <div class="stat-icon">⏱️</div>
                            <span class="stat-change">~${avgBlockTime}s</span>
                        </div>
                        <div class="stat-label">Block Time</div>
                        <div class="stat-value">${avgBlockTime}s</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <div class="stat-icon">👥</div>
                            <span class="stat-change">Active</span>
                        </div>
                        <div class="stat-label">Total Accounts</div>
                        <div class="stat-value">${utils.formatNumber(stats?.accounts || 0)}</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <div class="stat-icon">🌐</div>
                            <span class="stat-change" style="color: var(--accent-success)">● Online</span>
                        </div>
                        <div class="stat-label">Network Status</div>
                        <div class="stat-value" style="font-size: 1.5rem;">${nodes.filter(n => n.online).length}/${nodes.length} Nodes</div>
                    </div>
                </div>

                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">🔗</span>
                            Latest Blocks
                        </h2>
                        <button class="btn" onclick="showPage('blocks')">View All →</button>
                    </div>
                    
                    <div class="data-table">
                        <div class="table-header">
                            <div>Block</div>
                            <div>Hash</div>
                            <div>Age</div>
                            <div>Validator</div>
                            <div style="text-align: right">Txs</div>
                        </div>
                        
                        ${blocks.length > 0 ? blocks.map(block => `
                            <div class="table-row" onclick="showBlockDetail(${parseInt(block.number, 16)})">
                                <div class="col-block">#${parseInt(block.number, 16)}</div>
                                <div class="col-hash">${utils.formatHash(block.hash)}</div>
                                <div class="col-time">${utils.formatTime(block.timestamp)}</div>
                                <div class="col-hash">${block.miner ? utils.formatHash(block.miner) : 'System'}</div>
                                <div class="col-txs">
                                    <span class="tx-badge">${block.transactions?.length || 0} txs</span>
                                </div>
                            </div>
                        `).join('') : `
                            <div class="loading">
                                <div class="loading-spinner"></div>
                                <p>Loading blocks...</p>
                            </div>
                        `}
                    </div>
                </div>

                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">🖥️</span>
                            Network Nodes
                        </h2>
                        <button class="btn" onclick="showPage('nodes')">View Details →</button>
                    </div>
                    
                    <div class="nodes-grid">
                        ${nodes.map(node => `
                            <div class="node-card">
                                <div class="node-header">
                                    <span class="node-name">${node.name}</span>
                                    <span class="node-status ${node.online ? 'online' : 'offline'}">
                                        ● ${node.online ? 'Online' : 'Offline'}
                                    </span>
                                </div>
                                <div class="node-info">
                                    <div class="node-info-row">
                                        <span class="node-info-label">Endpoint</span>
                                        <span class="node-info-value">${node.endpoint}</span>
                                    </div>
                                    <div class="node-info-row">
                                        <span class="node-info-label">Latency</span>
                                        <span class="node-info-value">${node.online ? `${node.latency}ms` : 'N/A'}</span>
                                    </div>
                                </div>
                                ${node.online ? `
                                    <div class="node-block">#${utils.formatNumber(node.blockNumber)}</div>
                                ` : ''}
                            </div>
                        `).join('')}
                    </div>
                </div>
            </div>
        `;
    },

    async blocks() {
        const blocks = await rpcClient.getLatestBlocks(CONFIG.BLOCKS_PER_PAGE);
        
        return `
            <div class="page">
                <div class="hero">
                    <h1>Blocks</h1>
                    <p>All blocks on the MERKLITH blockchain</p>
                </div>
                
                <div class="section">
                    <div class="data-table">
                        <div class="table-header">
                            <div>Block</div>
                            <div>Hash</div>
                            <div>Age</div>
                            <div>Validator</div>
                            <div style="text-align: right">Txs</div>
                        </div>
                        
                        ${blocks.map(block => `
                            <div class="table-row" onclick="showBlockDetail(${parseInt(block.number, 16)})">
                                <div class="col-block">#${parseInt(block.number, 16)}</div>
                                <div class="col-hash">${utils.formatHash(block.hash)}</div>
                                <div class="col-time">${utils.formatTime(block.timestamp)}</div>
                                <div class="col-hash">${block.miner ? utils.formatHash(block.miner) : 'System'}</div>
                                <div class="col-txs">
                                    <span class="tx-badge">${block.transactions?.length || 0} txs</span>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            </div>
        `;
    },

    async nodes() {
        const nodes = await rpcClient.getNodeStatus();
        const onlineCount = nodes.filter(n => n.online).length;
        
        return `
            <div class="page">
                <div class="hero">
                    <h1>Network Nodes</h1>
                    <p>${onlineCount} of ${nodes.length} nodes online</p>
                </div>
                
                <div class="nodes-grid">
                    ${nodes.map(node => `
                        <div class="node-card">
                            <div class="node-header">
                                <span class="node-name">${node.name}</span>
                                <span class="node-status ${node.online ? 'online' : 'offline'}">
                                    ● ${node.online ? 'Online' : 'Offline'}
                                </span>
                            </div>
                            
                            <div class="node-info">
                                <div class="node-info-row">
                                    <span class="node-info-label">RPC Endpoint</span>
                                    <span class="node-info-value">${node.endpoint}</span>
                                </div>
                                <div class="node-info-row">
                                    <span class="node-info-label">WebSocket</span>
                                    <span class="node-info-value">ws://localhost:${node.wsPort}</span>
                                </div>
                                <div class="node-info-row">
                                    <span class="node-info-label">Response Time</span>
                                    <span class="node-info-value">${node.online ? `${node.latency}ms` : 'Timeout'}</span>
                                </div>
                            </div>
                            
                            ${node.online ? `
                                <div class="node-block">#${utils.formatNumber(node.blockNumber)}</div>
                            ` : `
                                <div class="node-block" style="color: var(--accent-danger); font-size: 1.25rem;">
                                    Node Unreachable
                                </div>
                            `}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    api() {
        return `
            <div class="page">
                <div class="hero">
                    <h1>API Documentation</h1>
                    <p>MERKLITH JSON-RPC API reference</p>
                </div>
                
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">📚</span>
                            Base Configuration
                        </h2>
                    </div>
                    
                    <div class="data-table" style="margin-bottom: 2rem;">
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">HTTP Endpoint</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">
                                http://localhost:8545
                            </div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">WebSocket</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">
                                ws://localhost:8546
                            </div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Chain ID</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">17001 (0x4269)</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Content-Type</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">
                                application/json
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">🔧</span>
                            Common Methods
                        </h2>
                    </div>
                    
                    <div class="data-table">
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">merklith_chainId</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">Returns the chain ID</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">merklith_blockNumber</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">Returns current block number</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">merklith_getBalance</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">Returns address balance</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">merklith_getBlockByNumber</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">Returns block by number</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">merklith_getTransactionByHash</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">Returns transaction details</div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }
};

// Search Handler
async function handleSearch(event) {
    if (event.key !== 'Enter') return;
    
    const query = event.target.value.trim();
    if (!query) return;
    
    showToast('Searching...', 'info');
    
    try {
        const results = await rpcClient.search(query);
        
        if (results.length === 0) {
            showToast('No results found', 'error');
            return;
        }
        
        if (results.length === 1) {
            // Navigate directly to result
            const result = results[0];
            if (result.type === 'block') {
                showBlockDetail(result.id);
            } else if (result.type === 'address') {
                showAddressDetail(result.id);
            } else {
                showToast(`Found ${result.type}: ${utils.formatHash(result.id)}`, 'success');
            }
        } else {
            // Show search results page
            showSearchResults(results);
        }
    } catch (error) {
        showToast('Search failed: ' + error.message, 'error');
    }
}

function showSearchResults(results) {
    const content = `
        <div class="page">
            <div class="hero">
                <h1>Search Results</h1>
                <p>Found ${results.length} result(s)</p>
            </div>
            
            <div class="section">
                <div class="data-table">
                    ${results.map(result => `
                        <div class="table-row" onclick="handleSearchResult('${result.type}', '${result.id}')">
                            <div class="col-block" style="text-transform: capitalize;">${result.type}</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">
                                <div style="font-weight: 600; color: var(--accent-primary); margin-bottom: 0.25rem;">
                                    ${result.title}
                                </div>
                                <div style="color: var(--text-muted); font-size: 0.9rem;">${result.subtitle}</div>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
    `;
    
    document.getElementById('content').innerHTML = content;
}

function handleSearchResult(type, id) {
    if (type === 'block') {
        showBlockDetail(id);
    } else if (type === 'address') {
        showAddressDetail(id);
    }
}

async function showBlockDetail(blockNumber) {
    try {
        const block = await rpcClient.getBlock(blockNumber);
        if (!block) {
            showToast('Block not found', 'error');
            return;
        }
        
        const content = `
            <div class="page">
                <div class="hero">
                    <h1>Block #${blockNumber}</h1>
                    <p>${new Date(parseInt(block.timestamp, 16) * 1000).toLocaleString()}</p>
                </div>
                
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">📋</span>
                            Block Details
                        </h2>
                        <button class="btn" onclick="showPage('blocks')">← Back to Blocks</button>
                    </div>
                    
                    <div class="data-table">
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Block Hash</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace; cursor: pointer;"
                                 onclick="utils.copyToClipboard('${block.hash}')">
                                ${block.hash} 📋
                            </div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Parent Hash</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">${block.parentHash}</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Validator</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">${block.miner || 'System'}</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Transactions</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">${block.transactions?.length || 0}</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Gas Used</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">${parseInt(block.gasUsed, 16).toLocaleString()}</div>
                        </div>
                    </div>
                </div>
                
                ${block.transactions?.length > 0 ? `
                    <div class="section">
                        <div class="section-header">
                            <h2 class="section-title">
                                <span class="section-icon">💸</span>
                                Transactions (${block.transactions.length})
                            </h2>
                        </div>
                        
                        <div class="data-table">
                            ${block.transactions.map((tx, idx) => `
                                <div class="table-row" style="cursor: default;">
                                    <div class="col-block">#${idx + 1}</div>
                                    <div class="col-hash">${typeof tx === 'string' ? utils.formatHash(tx) : utils.formatHash(tx.hash)}</div>
                                    <div class="col-time"></div>
                                    <div class="col-hash"></div>
                                    <div class="col-txs">
                                        <span class="tx-badge">View</span>
                                    </div>
                                </div>
                            `).join('')}
                        </div>
                    </div>
                ` : ''}
            </div>
        `;
        
        document.getElementById('content').innerHTML = content;
        
        // Update navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });
        
    } catch (error) {
        showToast('Failed to load block: ' + error.message, 'error');
    }
}

async function showAddressDetail(address) {
    try {
        const [balance, nonce] = await Promise.all([
            rpcClient.call('merklith_getBalance', [address]).catch(() => '0x0'),
            rpcClient.call('merklith_getNonce', [address]).catch(() => '0x0')
        ]);
        
        const content = `
            <div class="page">
                <div class="hero">
                    <h1>Account</h1>
                    <p style="font-family: 'JetBrains Mono', monospace;">${address}</p>
                </div>
                
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-icon">💰</div>
                        <div class="stat-label">Balance</div>
                        <div class="stat-value">${utils.formatBalance(balance)}</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-icon">📝</div>
                        <div class="stat-label">Transaction Count</div>
                        <div class="stat-value">${parseInt(nonce, 16)}</div>
                    </div>
                </div>
                
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">
                            <span class="section-icon">📋</span>
                            Account Details
                        </h2>
                        <button class="btn" onclick="utils.copyToClipboard('${address}')">Copy Address 📋</button>
                    </div>
                    
                    <div class="data-table">
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Address</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">${address}</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Balance (wei)</div>
                            <div class="col-hash" style="grid-column: 2 / -1; font-family: 'JetBrains Mono', monospace;">${balance}</div>
                        </div>
                        <div class="table-row" style="cursor: default;">
                            <div class="col-block">Nonce</div>
                            <div class="col-hash" style="grid-column: 2 / -1;">${parseInt(nonce, 16)}</div>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        document.getElementById('content').innerHTML = content;
        
    } catch (error) {
        showToast('Failed to load account: ' + error.message, 'error');
    }
}

// Page Navigation
async function showPage(pageName) {
    appState.currentPage = pageName;
    
    // Update navigation
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.remove('active');
        if (item.dataset.page === pageName) {
            item.classList.add('active');
        }
    });
    
    // Show loading state
    document.getElementById('content').innerHTML = `
        <div class="loading">
            <div class="loading-spinner"></div>
            <p>Loading...</p>
        </div>
    `;
    
    // Render page
    try {
        if (pages[pageName]) {
            const content = await pages[pageName]();
            document.getElementById('content').innerHTML = content;
        }
    } catch (error) {
        document.getElementById('content').innerHTML = `
            <div class="loading" style="color: var(--accent-danger);">
                <p>Failed to load page: ${error.message}</p>
            </div>
        `;
    }
}

// Real-time Updates
function startRealtimeUpdates() {
    setInterval(async () => {
        if (appState.currentPage === 'home') {
            // Update stats silently in background
            try {
                const stats = await rpcClient.getChainStats();
                if (stats && document.getElementById('stat-block-height')) {
                    document.getElementById('stat-block-height').textContent = utils.formatNumber(stats.blockNumber);
                }
            } catch (e) {}
        }
    }, CONFIG.UPDATE_INTERVAL);
}

// Initialize
async function init() {
    // Load initial page
    await showPage('home');
    
    // Start real-time updates
    startRealtimeUpdates();
    
    console.log('🚀 MERKLITH Explorer initialized');
    console.log('📦 Connected to:', CONFIG.RPC_ENDPOINTS[0]);
}

// Start when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
