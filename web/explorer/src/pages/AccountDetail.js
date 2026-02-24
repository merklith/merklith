import React from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useAccount } from '../hooks/useMerklith';

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

const Address = styled.div`
  font-family: monospace;
  color: rgba(255, 255, 255, 0.7);
  font-size: 1.1rem;
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

const BalanceGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
`;

const BalanceCard = styled.div`
  background: rgba(0, 212, 255, 0.05);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 12px;
  padding: 1.5rem;
`;

const BalanceLabel = styled.div`
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.5);
  margin-bottom: 0.5rem;
`;

const BalanceValue = styled.div`
  font-size: 1.5rem;
  font-weight: 700;
  color: #00d4ff;
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

const AccountDetail = () => {
  const { address } = useParams();
  const { account, loading } = useAccount(address);

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
  };

  const formatBalance = (wei) => {
    if (!wei) return '0';
    // Convert wei to ANV (assuming 18 decimals)
    const anv = parseInt(wei, 16) / 1e18;
    return anv.toFixed(4);
  };

  if (loading) return <LoadingText>Loading account...</LoadingText>;

  return (
    <Container>
      <Header>
        <Title>Account</Title>
        <Address>
          {address}
          <CopyButton onClick={() => copyToClipboard(address)}>Copy</CopyButton>
        </Address>
      </Header>

      <Card>
        <SectionTitle>Balance</SectionTitle>
        <BalanceGrid>
          <BalanceCard>
            <BalanceLabel>ANV Balance</BalanceLabel>
            <BalanceValue>{formatBalance(account?.balance)} ANV</BalanceValue>
          </BalanceCard>
          
          <BalanceCard>
            <BalanceLabel>Nonce</BalanceLabel>
            <BalanceValue>{account?.nonce || 0}</BalanceValue>
          </BalanceCard>
        </BalanceGrid>
      </Card>

      <Card>
        <SectionTitle>Account Info</SectionTitle>
        
        <DetailRow>
          <Label>Address:</Label>
          <Value monospace>{address}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Raw Balance (wei):</Label>
          <Value monospace>{account?.balance || '0'}</Value>
        </DetailRow>

        <DetailRow>
          <Label>Transaction Count:</Label>
          <Value>{account?.nonce || 0}</Value>
        </DetailRow>
      </Card>
    </Container>
  );
};

export default AccountDetail;
