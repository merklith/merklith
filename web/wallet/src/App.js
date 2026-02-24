import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import styled, { createGlobalStyle } from 'styled-components';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

import WalletProvider from './context/WalletContext';
import Dashboard from './pages/Dashboard';
import CreateWallet from './pages/CreateWallet';
import ImportWallet from './pages/ImportWallet';
import Send from './pages/Send';
import Receive from './pages/Receive';
import History from './pages/History';
import Settings from './pages/Settings';

const GlobalStyle = createGlobalStyle`
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  
  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
  }
`;

const AppContainer = styled.div`
  min-height: 100vh;
  display: flex;
  flex-direction: column;
`;

const Header = styled.header`
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  padding: 1rem 2rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const Logo = styled.div`
  font-size: 1.5rem;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  display: flex;
  align-items: center;
  gap: 0.5rem;
`;

const NetworkSelector = styled.select`
  padding: 0.5rem 1rem;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
  background: white;
  font-size: 0.9rem;
  cursor: pointer;
`;

const Main = styled.main`
  flex: 1;
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
`;

function App() {
  const [network, setNetwork] = useState('mainnet');

  return (
    <WalletProvider>
      <GlobalStyle />
      <ToastContainer position="top-right" theme="colored" />
      <Router>
        <AppContainer>
          <Header>
            <Logo>
              <span>⚒️</span>
              MERKLITH Wallet
            </Logo>
            <NetworkSelector 
              value={network} 
              onChange={(e) => setNetwork(e.target.value)}
            >
              <option value="mainnet">MERKLITH Mainnet</option>
              <option value="testnet">MERKLITH Testnet</option>
              <option value="devnet">MERKLITH Devnet</option>
              <option value="localhost">Localhost:8545</option>
            </NetworkSelector>
          </Header>
          
          <Main>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/create" element={<CreateWallet />} />
              <Route path="/import" element={<ImportWallet />} />
              <Route path="/send" element={<Send />} />
              <Route path="/receive" element={<Receive />} />
              <Route path="/history" element={<History />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </Main>
        </AppContainer>
      </Router>
    </WalletProvider>
  );
}

export default App;