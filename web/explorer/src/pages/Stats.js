import React from 'react';
import styled from 'styled-components';
import { useChainStats } from '../hooks/useMerklith';

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

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 3rem;
`;

const StatCard = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
`;

const StatLabel = styled.div`
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 0.5rem;
`;

const StatValue = styled.div`
  font-size: 2rem;
  font-weight: 700;
  color: #00d4ff;
`;

const Card = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 2rem;
  margin-bottom: 2rem;
`;

const CardTitle = styled.h2`
  font-size: 1.3rem;
  margin-bottom: 1.5rem;
  color: #ffffff;
`;

const InfoRow = styled.div`
  display: flex;
  justify-content: space-between;
  padding: 0.75rem 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);

  &:last-child {
    border-bottom: none;
  }
`;

const InfoLabel = styled.div`
  color: rgba(255, 255, 255, 0.6);
`;

const InfoValue = styled.div`
  color: #ffffff;
  font-weight: 500;
`;

const ChartPlaceholder = styled.div`
  background: rgba(255, 255, 255, 0.02);
  border: 2px dashed rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 4rem;
  text-align: center;
  color: rgba(255, 255, 255, 0.4);
`;

const Stats = () => {
  const stats = useChainStats();

  return (
    <Container>
      <Header>
        <Title>Network Statistics</Title>
        <Subtitle>Real-time metrics and analytics</Subtitle>
      </Header>

      {stats && (
        <StatsGrid>
          <StatCard>
            <StatLabel>Block Height</StatLabel>
            <StatValue>{stats.blockNumber?.toLocaleString()}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Total Accounts</StatLabel>
            <StatValue>{stats.accounts}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Chain ID</StatLabel>
            <StatValue>{stats.chainId}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Block Time</StatLabel>
            <StatValue>~2s</StatValue>
          </StatCard>
        </StatsGrid>
      )}

      <Card>
        <CardTitle>Network Configuration</CardTitle>
        <InfoRow>
          <InfoLabel>Consensus Algorithm</InfoLabel>
          <InfoValue>Proof of Contribution (PoC)</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Block Time</InfoLabel>
          <InfoValue>2 seconds</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Gas Price</InfoLabel>
          <InfoValue>1 gwei</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Gas Limit</InfoLabel>
          <InfoValue>30,000,000</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Currency Symbol</InfoLabel>
          <InfoValue>ANV</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Decimals</InfoLabel>
          <InfoValue>18</InfoValue>
        </InfoRow>
      </Card>

      <Card>
        <CardTitle>Transaction Activity</CardTitle>
        <ChartPlaceholder>
          📊 Transaction volume charts coming soon...
        </ChartPlaceholder>
      </Card>

      <Card>
        <CardTitle>Network Health</CardTitle>
        <InfoRow>
          <InfoLabel>Node Status</InfoLabel>
          <InfoValue style={{ color: '#00ff88' }}>● All Systems Operational</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Active Validators</InfoLabel>
          <InfoValue>3</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Network Latency</InfoLabel>
          <InfoValue>{'< 50ms'}</InfoValue>
        </InfoRow>
        <InfoRow>
          <InfoLabel>Last Block</InfoLabel>
          <InfoValue>{stats?.blockNumber?.toLocaleString() || 'Loading...'}</InfoValue>
        </InfoRow>
      </Card>
    </Container>
  );
};

export default Stats;
