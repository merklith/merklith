import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import styled from 'styled-components';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

import Home from './pages/Home';
import BlockList from './pages/BlockList';
import BlockDetail from './pages/BlockDetail';
import TransactionList from './pages/TransactionList';
import TransactionDetail from './pages/TransactionDetail';
import AccountDetail from './pages/AccountDetail';
import SearchResults from './pages/SearchResults';
import Validators from './pages/Validators';
import Stats from './pages/Stats';
import APIDocs from './pages/APIDocs';

const AppContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  color: #ffffff;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
`;

const Header = styled.header`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding: 1rem 2rem;
  position: sticky;
  top: 0;
  z-index: 1000;
`;

const HeaderContent = styled.div`
  max-width: 1400px;
  margin: 0 auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const Logo = styled(Link)`
  display: flex;
  align-items: center;
  gap: 0.75rem;
  text-decoration: none;
  color: #ffffff;
  font-size: 1.5rem;
  font-weight: 700;
  
  &:hover {
    color: #00d4ff;
  }
`;

const LogoIcon = styled.div`
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #00d4ff, #0099cc);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
`;

const Nav = styled.nav`
  display: flex;
  gap: 2rem;
  align-items: center;
`;

const NavLink = styled(Link)`
  color: rgba(255, 255, 255, 0.7);
  text-decoration: none;
  font-weight: 500;
  transition: color 0.2s;
  
  &:hover {
    color: #00d4ff;
  }
`;

const SearchBar = styled.div`
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 0.5rem 1rem;
  width: 400px;
  max-width: 100%;
`;

const SearchInput = styled.input`
  background: transparent;
  border: none;
  color: white;
  width: 100%;
  font-size: 0.9rem;
  
  &::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }
  
  &:focus {
    outline: none;
  }
`;

const Main = styled.main`
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem;
`;

const Footer = styled.footer`
  background: rgba(0, 0, 0, 0.3);
  padding: 2rem;
  margin-top: 4rem;
  text-align: center;
  color: rgba(255, 255, 255, 0.5);
`;

const NetworkBadge = styled.div`
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  padding: 0.5rem 1rem;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 500;
`;

const StatusDot = styled.div`
  width: 8px;
  height: 8px;
  background: #00ff88;
  border-radius: 50%;
  animation: pulse 2s infinite;
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
`;

function App() {
  const [searchQuery, setSearchQuery] = useState('');
  const [networkStatus, setNetworkStatus] = useState({ connected: true, chainId: 1337 });

  useEffect(() => {
    // Check network status periodically
    const checkStatus = async () => {
      try {
        const response = await fetch('http://localhost:8545', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'eth_chainId',
            params: [],
            id: 1
          })
        });
        
        if (response.ok) {
          const data = await response.json();
          setNetworkStatus({ connected: true, chainId: parseInt(data.result, 16) });
        }
      } catch (error) {
        setNetworkStatus({ connected: false, chainId: 0 });
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleSearch = (e) => {
    if (e.key === 'Enter' && searchQuery.trim()) {
      window.location.href = `/search?q=${encodeURIComponent(searchQuery)}`;
    }
  };

  return (
    <AppContainer>
      <ToastContainer position="top-right" theme="dark" />
      <Router>
        <Header>
          <HeaderContent>
            <Logo to="/">
              <LogoIcon>⚒️</LogoIcon>
              MERKLITH Explorer
            </Logo>
            
            <SearchBar>
              <SearchInput
                type="text"
                placeholder="Search by block, tx hash, address..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyPress={handleSearch}
              />
            </SearchBar>
            
            <Nav>
              <NavLink to="/">Home</NavLink>
              <NavLink to="/blocks">Blocks</NavLink>
              <NavLink to="/transactions">Transactions</NavLink>
              <NavLink to="/validators">Validators</NavLink>
              <NavLink to="/stats">Stats</NavLink>
              <NavLink to="/api">API</NavLink>
            </Nav>
            
            <NetworkBadge>
              <StatusDot />
              {networkStatus.connected ? `Chain ${networkStatus.chainId}` : 'Disconnected'}
            </NetworkBadge>
          </HeaderContent>
        </Header>

        <Main>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/blocks" element={<BlockList />} />
            <Route path="/block/:number" element={<BlockDetail />} />
            <Route path="/transactions" element={<TransactionList />} />
            <Route path="/tx/:hash" element={<TransactionDetail />} />
            <Route path="/address/:address" element={<AccountDetail />} />
            <Route path="/search" element={<SearchResults />} />
            <Route path="/validators" element={<Validators />} />
            <Route path="/stats" element={<Stats />} />
            <Route path="/api" element={<APIDocs />} />
          </Routes>
        </Main>

        <Footer>
          <p>MERKLITH Blockchain Explorer © 2024 - Where Trust is Forged</p>
          <p style={{ marginTop: '0.5rem', fontSize: '0.9rem' }} >
            Built with ❤️ by the MERKLITH team
          </p>
        </Footer>
      </Router>
    </AppContainer>
  );
}

export default App;