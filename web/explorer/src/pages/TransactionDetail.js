import React from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useTransaction } from '../hooks/useMerklith';

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

const StatusBadge = styled.span`
  background: ${props => props.success ? 'rgba(0, 255, 136, 0.2)' : 'rgba(255, 107, 107, 0.2)'};
  color: ${props => props.success ? '#00ff88' : '#ff6b6b'};
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.85rem;
  font-weight: 500;
`;

const TransactionDetail = () => {
  const { hash } = useParams();
  const { tx, loading } = useTransaction(hash);

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
  };

  if (loading) return <LoadingText>Loading transaction...</LoadingText>;
  if (!tx) return <ErrorText>Transaction not found</ErrorText>;

  return (
    <Container>
      <Header>
        <Title>Transaction Details</Title>
      </Header>

      <Card>
        <SectionTitle>Overview</SectionTitle>

        <DetailRow>
          <Label>Transaction Hash:</Label>
          <Value monospace>
            {tx.hash}
            <CopyButton onClick={() => copyToClipboard(tx.hash)}>Copy</CopyButton>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Status:</Label>
          <Value>
            <StatusBadge success>✓ Success</StatusBadge>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Block:</Label>
          <Value>
            #{tx.blockNumber && parseInt(tx.blockNumber, 16)}
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>From:</Label>
          <Value monospace>
            {tx.from}
            <CopyButton onClick={() => copyToClipboard(tx.from)}>Copy</CopyButton>
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>To:</Label>
          <Value monospace>
            {tx.to || 'Contract Creation'}
            {tx.to && (
              <CopyButton onClick={() => copyToClipboard(tx.to)}>Copy</CopyButton>
            )}
          </Value>
        </DetailRow>

        <DetailRow>
          <Label>Value:</Label>
          <Value>{tx.value} wei</Value>
        </DetailRow>

        <DetailRow>
          <Label>Gas Price:</Label>
          <Value>{tx.gasPrice || '1 gwei'}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Gas Limit:</Label>
          <Value>{tx.gas || '21000'}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Nonce:</Label>
          <Value>{tx.nonce && parseInt(tx.nonce, 16)}</Value>
        </DetailRow>
      </Card>
    </Container>
  );
};

export default TransactionDetail;
