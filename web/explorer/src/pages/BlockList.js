import React, { useState } from 'react';
import styled from 'styled-components';
import { Link } from 'react-router-dom';
import { useBlocks, useBlockNumber } from '../hooks/useMerklith';

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

const Controls = styled.div`
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
  align-items: center;
`;

const Button = styled.button`
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: #00d4ff;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;

  &:hover:not(:disabled) {
    background: rgba(0, 212, 255, 0.2);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const BlockTable = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  overflow: hidden;
`;

const TableHeader = styled.div`
  display: grid;
  grid-template-columns: 100px 2fr 1fr 1fr 100px;
  gap: 1rem;
  padding: 1rem 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  font-weight: 600;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.7);
`;

const BlockRow = styled(Link)`
  display: grid;
  grid-template-columns: 100px 2fr 1fr 1fr 100px;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  text-decoration: none;
  color: inherit;
  transition: background 0.2s;
  align-items: center;

  &:last-child {
    border-bottom: none;
  }

  &:hover {
    background: rgba(255, 255, 255, 0.05);
  }
`;

const BlockNumber = styled.div`
  font-weight: 600;
  color: #00d4ff;
`;

const BlockHash = styled.div`
  font-family: monospace;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.7);
`;

const BlockTime = styled.div`
  color: rgba(255, 255, 255, 0.5);
`;

const Validator = styled.div`
  color: rgba(255, 255, 255, 0.6);
  font-family: monospace;
  font-size: 0.85rem;
`;

const TxCount = styled.div`
  text-align: right;
`;

const LoadingText = styled.div`
  text-align: center;
  padding: 3rem;
  color: rgba(255, 255, 255, 0.5);
`;

const BlockList = () => {
  const currentBlock = useBlockNumber();
  const [page, setPage] = useState(0);
  const blocksPerPage = 25;
  
  const { blocks, loading } = useBlocks(blocksPerPage);

  const formatHash = (hash) => {
    if (!hash) return '';
    return `${hash.slice(0, 20)}...${hash.slice(-8)}`;
  };

  const formatTime = (timestamp) => {
    if (!timestamp) return '';
    const date = new Date(parseInt(timestamp) * 1000);
    return date.toLocaleString();
  };

  const formatValidator = (address) => {
    if (!address) return '';
    return `${address.slice(0, 12)}...${address.slice(-4)}`;
  };

  return (
    <Container>
      <Header>
        <Title>Blocks</Title>
        <Subtitle>
          Latest blocks on the MERKLITH blockchain
          {currentBlock && ` • Block #${currentBlock.toLocaleString()}`}
        </Subtitle>
      </Header>

      <Controls>
        <Button 
          onClick={() => setPage(p => Math.max(0, p - 1))}
          disabled={page === 0}
        >
          ← Previous
        </Button>
        <span style={{ color: 'rgba(255,255,255,0.6)' }}>
          Page {page + 1}
        </span>
        <Button onClick={() => setPage(p => p + 1)}>
          Next →
        </Button>
      </Controls>

      {loading ? (
        <LoadingText>Loading blocks...</LoadingText>
      ) : (
        <BlockTable>
          <TableHeader>
            <div>Block</div>
            <div>Hash</div>
            <div>Time</div>
            <div>Validator</div>
            <div style={{ textAlign: 'right' }}>Txs</div>
          </TableHeader>
          
          {blocks.map((block) => (
            <BlockRow key={block.number} to={`/block/${parseInt(block.number, 16)}`}>
              <BlockNumber>#{parseInt(block.number, 16)}</BlockNumber>
              <BlockHash>{formatHash(block.hash)}</BlockHash>
              <BlockTime>{formatTime(block.timestamp)}</BlockTime>
              <Validator>{formatValidator(block.miner)}</Validator>
              <TxCount>{block.transactions?.length || 0}</TxCount>
            </BlockRow>
          ))}
        </BlockTable>
      )}
    </Container>
  );
};

export default BlockList;
