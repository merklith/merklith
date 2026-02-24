import React, { useState } from 'react';
import styled from 'styled-components';

const Container = styled.div`
  max-width: 1400px;
  margin: 0 auto;
`;

const Header = styled.div`
  margin-bottom: 2rem;
`;

const Title = styled.h1`
  font-size: 2rem;
  margin-bottom: 0.5rem;
`;

const Subtitle = styled.p`
  color: rgba(255, 255, 255, 0.6);
`;

const EndpointCard = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
`;

const EndpointHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
  cursor: pointer;
`;

const MethodBadge = styled.span`
  background: ${props => {
    if (props.method === 'POST') return 'rgba(0, 212, 255, 0.2)';
    if (props.method === 'GET') return 'rgba(0, 255, 136, 0.2)';
    return 'rgba(255, 193, 7, 0.2)';
  }};
  color: ${props => {
    if (props.method === 'POST') return '#00d4ff';
    if (props.method === 'GET') return '#00ff88';
    return '#ffc107';
  }};
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.85rem;
  font-weight: 600;
  font-family: monospace;
`;

const EndpointPath = styled.code`
  color: #ffffff;
  font-size: 1rem;
`;

const EndpointDescription = styled.p`
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 1rem;
`;

const CodeBlock = styled.pre`
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  padding: 1rem;
  overflow-x: auto;
  font-family: 'Monaco', 'Consolas', monospace;
  font-size: 0.9rem;
  color: #00d4ff;
  margin: 0.5rem 0;
`;

const SectionTitle = styled.h2`
  font-size: 1.5rem;
  margin: 2rem 0 1rem;
  color: #ffffff;
`;

const ExpandIcon = styled.span`
  margin-left: auto;
  color: rgba(255, 255, 255, 0.5);
  transition: transform 0.2s;
  transform: ${props => props.expanded ? 'rotate(180deg)' : 'rotate(0deg)'};
`;

const APIDocs = () => {
  const [expandedSections, setExpandedSections] = useState({});

  const toggleSection = (id) => {
    setExpandedSections(prev => ({
      ...prev,
      [id]: !prev[id]
    }));
  };

  const endpoints = [
    {
      id: 'chainId',
      method: 'POST',
      path: 'merklith_chainId',
      description: 'Returns the chain ID of the current network.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_chainId",
  "params": [],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": "0x4269",
  "id": 1
}`
    },
    {
      id: 'blockNumber',
      method: 'POST',
      path: 'merklith_blockNumber',
      description: 'Returns the current block number.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_blockNumber",
  "params": [],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": "0x1a4",
  "id": 1
}`
    },
    {
      id: 'getBalance',
      method: 'POST',
      path: 'merklith_getBalance',
      description: 'Returns the balance of an address.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_getBalance",
  "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": "0x56bc75e2d63100000",
  "id": 1
}`
    },
    {
      id: 'getBlockByNumber',
      method: 'POST',
      path: 'merklith_getBlockByNumber',
      description: 'Returns block information by block number.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_getBlockByNumber",
  "params": ["0x1a4", true],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": {
    "number": "0x1a4",
    "hash": "0x...",
    "parentHash": "0x...",
    "timestamp": "0x...",
    "transactions": []
  },
  "id": 1
}`
    },
    {
      id: 'getTransactionByHash',
      method: 'POST',
      path: 'merklith_getTransactionByHash',
      description: 'Returns transaction information by hash.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_getTransactionByHash",
  "params": ["0x..."],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": {
    "hash": "0x...",
    "from": "0x...",
    "to": "0x...",
    "value": "0x...",
    "gas": "0x..."
  },
  "id": 1
}`
    },
    {
      id: 'sendTransaction',
      method: 'POST',
      path: 'merklith_sendSignedTransaction',
      description: 'Submit a signed transaction to the network.',
      request: `{
  "jsonrpc": "2.0",
  "method": "merklith_sendSignedTransaction",
  "params": ["0x..."],
  "id": 1
}`,
      response: `{
  "jsonrpc": "2.0",
  "result": "0x...",
  "id": 1
}`
    }
  ];

  return (
    <Container>
      <Header>
        <Title>API Documentation</Title>
        <Subtitle>
          MERKLITH JSON-RPC API reference. All requests should be sent to 
          <code style={{ color: '#00d4ff' }}>http://localhost:8545</code>
        </Subtitle>
      </Header>

      <SectionTitle>Base URL</SectionTitle>
      <CodeBlock>http://localhost:8545</CodeBlock>

      <SectionTitle>Common Headers</SectionTitle>
      <CodeBlock>{`Content-Type: application/json`}</CodeBlock>

      <SectionTitle>Endpoints</SectionTitle>

      {endpoints.map((endpoint) => (
        <EndpointCard key={endpoint.id}>
          <EndpointHeader onClick={() => toggleSection(endpoint.id)}>
            <MethodBadge method={endpoint.method}>{endpoint.method}</MethodBadge>
            <EndpointPath>{endpoint.path}</EndpointPath>
            <ExpandIcon expanded={expandedSections[endpoint.id]}>▼</ExpandIcon>
          </EndpointHeader>
          
          <EndpointDescription>{endpoint.description}</EndpointDescription>

          {expandedSections[endpoint.id] && (
            <>
              <div style={{ marginBottom: '1rem' }}>
                <strong style={{ color: 'rgba(255,255,255,0.8)' }}>Request:</strong>
                <CodeBlock>{endpoint.request}</CodeBlock>
              </div>
              
              <div>
                <strong style={{ color: 'rgba(255,255,255,0.8)' }}>Response:</strong>
                <CodeBlock>{endpoint.response}</CodeBlock>
              </div>
            </>
          )}
        </EndpointCard>
      ))}

      <SectionTitle>Ethereum Compatibility</SectionTitle>
      <p style={{ color: 'rgba(255,255,255,0.7)', marginBottom: '1rem' }}>
        MERKLITH also supports standard Ethereum JSON-RPC methods with 
        <code style={{ color: '#00d4ff' }}>eth_</code> prefix:
      </p>
      <CodeBlock>{`eth_chainId, eth_blockNumber, eth_getBalance, 
eth_getTransactionByHash, eth_sendTransaction, 
eth_call, eth_estimateGas, eth_gasPrice`}</CodeBlock>
    </Container>
  );
};

export default APIDocs;
