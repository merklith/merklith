import React from 'react';
import { useParams, Link } from 'react-router-dom';
import styled from 'styled-components';
import { useBlock } from '../hooks/useMerklith';

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

const BlockNumber = styled.span`
  color: #00d4ff;
`;

const Card = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 2rem;
  margin-bottom: 2rem;
`;

const SectionTitle = styled.h2`
  font-size: 1.3rem;
  margin-bottom: 1.5rem;
  color: #ffffff;
`;

const DetailRow = styled.div`
  display: flex;
  padding: 0.75rem 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);

  &:last-child {
    border-bottom: none;
  }
`;

const Label = styled.div`
  width: 200px;
  color: rgba(255, 255, 255, 0.5);
  font-weight: 500;
`;

const Value = styled.div`
  flex: 1;
  color: #ffffff;
  font-family: ${props => props.monospace ? 'monospace' : 'inherit'};
  word-break: break-all;
`;

const CopyButton = styled.button`
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: rgba(255, 255, 255, 0.7);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 0.5rem;
  font-size: 0.8rem;

  &:hover {
    background: rgba(255, 255, 255, 0.2);
  }
`;

const TxList = styled.div`
  margin-top: 1rem;
`;

const TxItem = styled(Link)`
  display: block;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
  margin-bottom: 0.5rem;
  text-decoration: none;
  color: inherit;
  transition: background 0.2s;

  &:hover {
    background: rgba(255, 255, 255, 0.06);
  }
`;

const TxHash = styled.div`
  font-family: monospace;
  color: #00d4ff;
  margin-bottom: 0.5rem;
`;

const TxDetails = styled.div`
  display: flex;
  gap: 2rem;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.6);
`;

const LoadingText = styled.div`
  text-align: center;
  padding: 3rem;
  color: rgba(255, 255, 255, 0.5);
`;

const ErrorText = styled.div`
  text-align: center;
  padding: 3rem;
  color: #ff6b6b;
`;

const NavButtons = styled.div`
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
`;

const NavButton = styled(Link)`
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: #00d4ff;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  text-decoration: none;
  font-weight: 500;
  transition: all 0.2s;

  &:hover {
    background: rgba(0, 212, 255, 0.2);
  }
`;

const BlockDetail = () => {
  const { number } = useParams();
  const blockNum = parseInt(number);
  const { block, loading } = useBlock(blockNum);

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
  };

  const formatValue = (val) => {
    if (!val) return '';
    if (typeof val === 'string' && val.startsWith('0x')) {
      return val;
    }
    return val;
  };

  if (loading) return <LoadingText>Loading block...</LoadingText>;
  if (!block) return <ErrorText>Block not found</ErrorText>;

  return (
    <Container>
      <Header>
        <Title>
          Block <BlockNumber>#{blockNum}</BlockNumber>
        </Title>
      </Header>

      <NavButtons>
        <NavButton to={`/block/${blockNum - 1}`}>← Previous Block</NavButton>
        <NavButton to={`/block/${blockNum + 1}`}>Next Block →</NavButton>
      </NavButtons>

      <Card>
        <SectionTitle>Overview</SectionTitle>
        
        <DetailRow>
          <Label>Block Height:</Label>
          <Value>{blockNum}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Timestamp:</Label>
          <Value>
            {block.timestamp && new Date(parseInt(block.timestamp, 16) * 1000).toLocaleString()}
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Transactions:</Label>
          <Value>{block.transactions?.length || 0}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Validator:</Label>
          <Value monospace>
            {block.miner || 'N/A'}
            <CopyButton onClick={() => copyToClipboard(block.miner)}>Copy</CopyButton>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Block Hash:</Label>
          <Value monospace>
            {block.hash}
            <CopyButton onClick={() => copyToClipboard(block.hash)}>Copy</CopyButton>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Parent Hash:</Label>
          <Value monospace>
            {block.parentHash}
            <CopyButton onClick={() => copyToClipboard(block.parentHash)}>Copy</CopyButton>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>State Root:</Label>
          <Value monospace>{block.stateRoot || 'N/A'}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Gas Used:</Label>
          <Value>{block.gasUsed || '0'}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Gas Limit:</Label>
          <Value>{block.gasLimit || '0'}</Value>
        </DetailRow>
      </Card>

      {block.transactions && block.transactions.length > 0 && (
        <Card>
          <SectionTitle>
            Transactions ({block.transactions.length})
          </SectionTitle>
          
          <TxList>
            {block.transactions.map((tx, idx) => (
              <TxItem key={idx} to={`/tx/${typeof tx === 'string' ? tx : tx.hash}`}>
                <TxHash>
                  {typeof tx === 'string' ? tx : tx.hash}
                </TxHash>
                {typeof tx !== 'string' && (
                  <TxDetails>
                    <span>From: {tx.from?.slice(0, 20)}...</span>
                    <span>To: {tx.to?.slice(0, 20)}...</span>
                    <span>Value: {tx.value} wei</span>
                  </TxDetails>
                )}
              </TxItem>
            ))}
          </TxList>
        </Card>
      )}
    </Container>
  );
};

export default BlockDetail;
