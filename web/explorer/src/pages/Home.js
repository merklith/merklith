import React from 'react';
import styled from 'styled-components';
import { Link } from 'react-router-dom';
import { useChainStats, useBlocks } from '../hooks/useMerklith';

const Container = styled.div`
  max-width: 1400px;
  margin: 0 auto;
`;

const Hero = styled.div`
  text-align: center;
  padding: 3rem 0;
  margin-bottom: 2rem;
`;

const Title = styled.h1`
  font-size: 3rem;
  font-weight: 800;
  margin-bottom: 1rem;
  background: linear-gradient(135deg, #00d4ff, #0099cc);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
`;

const Subtitle = styled.p`
  font-size: 1.2rem;
  color: rgba(255, 255, 255, 0.7);
  max-width: 600px;
  margin: 0 auto;
`;

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 3rem;
`;

const StatCard = styled.div`
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 1.5rem;
  backdrop-filter: blur(10px);
  transition: transform 0.2s, box-shadow 0.2s;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 10px 30px rgba(0, 212, 255, 0.1);
  }
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

const Section = styled.div`
  margin-bottom: 3rem;
`;

const SectionHeader = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
`;

const SectionTitle = styled.h2`
  font-size: 1.5rem;
  font-weight: 600;
  color: #ffffff;
`;

const ViewAllLink = styled(Link)`
  color: #00d4ff;
  text-decoration: none;
  font-weight: 500;
  
  &:hover {
    text-decoration: underline;
  }
`;

const BlockList = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  overflow: hidden;
`;

const BlockItem = styled(Link)`
  display: grid;
  grid-template-columns: 80px 1fr 1fr 120px;
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
  font-family: 'Monaco', 'Consolas', monospace;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.6);
  overflow: hidden;
  text-overflow: ellipsis;
`;

const BlockTime = styled.div`
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.9rem;
`;

const TxCount = styled.div`
  text-align: right;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
`;

const LoadingText = styled.div`
  text-align: center;
  padding: 3rem;
  color: rgba(255, 255, 255, 0.5);
`;

const FeaturesGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  margin-top: 3rem;
`;

const FeatureCard = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 2rem;
`;

const FeatureIcon = styled.div`
  font-size: 2.5rem;
  margin-bottom: 1rem;
`;

const FeatureTitle = styled.h3`
  font-size: 1.2rem;
  margin-bottom: 0.5rem;
  color: #ffffff;
`;

const FeatureDescription = styled.p`
  color: rgba(255, 255, 255, 0.6);
  line-height: 1.6;
`;

const Home = () => {
  const stats = useChainStats();
  const { blocks, loading } = useBlocks(10);

  const formatHash = (hash) => {
    if (!hash) return '';
    return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
  };

  const formatTime = (timestamp) => {
    if (!timestamp) return '';
    const date = new Date(parseInt(timestamp) * 1000);
    return date.toLocaleTimeString();
  };

  return (
    <Container>
      <Hero>
        <Title>MERKLITH Blockchain</Title>
        <Subtitle>
          Proof of Contribution based Layer 1 blockchain. 
          Fast, secure, and decentralized.
        </Subtitle>
      </Hero>

      {stats && (
        <StatsGrid>
          <StatCard>
            <StatLabel>Block Height</StatLabel>
            <StatValue>{stats.blockNumber?.toLocaleString() || '0'}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Chain ID</StatLabel>
            <StatValue>{stats.chainId}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Total Accounts</StatLabel>
            <StatValue>{stats.accounts || '0'}</StatValue>
          </StatCard>
          <StatCard>
            <StatLabel>Network Status</StatLabel>
            <StatValue style={{ color: '#00ff88', fontSize: '1.2rem' }}>
              ● Online
            </StatValue>
          </StatCard>
        </StatsGrid>
      )}

      <Section>
        <SectionHeader>
          <SectionTitle>Latest Blocks</SectionTitle>
          <ViewAllLink to="/blocks">View All →</ViewAllLink>
        </SectionHeader>

        {loading ? (
          <LoadingText>Loading blocks...</LoadingText>
        ) : (
          <BlockList>
            {blocks.map((block) => (
              <BlockItem key={block.number} to={`/block/${parseInt(block.number, 16)}`}>
                <BlockNumber>#{parseInt(block.number, 16)}</BlockNumber>
                <BlockHash>{formatHash(block.hash)}</BlockHash>
                <BlockTime>{formatTime(block.timestamp)}</BlockTime>
                <TxCount>{block.transactions?.length || 0} txs</TxCount>
              </BlockItem>
            ))}
          </BlockList>
        )}
      </Section>

      <FeaturesGrid>
        <FeatureCard>
          <FeatureIcon>⚡</FeatureIcon>
          <FeatureTitle>Fast Consensus</FeatureTitle>
          <FeatureDescription>
            Proof of Contribution consensus with 2-second block times 
            and instant finality.
          </FeatureDescription>
        </FeatureCard>
        <FeatureCard>
          <FeatureIcon>🔒</FeatureIcon>
          <FeatureTitle>Secure by Design</FeatureTitle>
          <FeatureDescription>
            Ed25519 signatures, BLS attestations, and multi-layered 
            security mechanisms.
          </FeatureDescription>
        </FeatureCard>
        <FeatureCard>
          <FeatureIcon>🌐</FeatureIcon>
          <FeatureTitle>Web3 Compatible</FeatureTitle>
          <FeatureDescription>
            Full Ethereum JSON-RPC compatibility with additional 
            MERKLITH-native methods.
          </FeatureDescription>
        </FeatureCard>
      </FeaturesGrid>
    </Container>
  );
};

export default Home;
