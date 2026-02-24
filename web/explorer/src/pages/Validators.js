import React from 'react';
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

const ValidatorsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
`;

const ValidatorCard = styled.div`
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
  transition: transform 0.2s, box-shadow 0.2s;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 10px 30px rgba(0, 212, 255, 0.1);
  }
`;

const ValidatorHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
`;

const ValidatorIcon = styled.div`
  width: 50px;
  height: 50px;
  background: linear-gradient(135deg, #00d4ff, #0099cc);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
`;

const ValidatorName = styled.h3`
  font-size: 1.1rem;
  color: #ffffff;
`;

const ValidatorStatus = styled.div`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(0, 255, 136, 0.1);
  color: #00ff88;
  padding: 0.25rem 0.75rem;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 500;
`;

const StatusDot = styled.div`
  width: 8px;
  height: 8px;
  background: #00ff88;
  border-radius: 50%;
`;

const ValidatorStats = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
`;

const Stat = styled.div`
  text-align: center;
`;

const StatLabel = styled.div`
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.5);
  margin-bottom: 0.25rem;
`;

const StatValue = styled.div`
  font-weight: 600;
  color: #ffffff;
`;

const Validators = () => {
  // Mock validator data - in real app, fetch from API
  const validators = [
    {
      id: 1,
      name: 'Validator Node 1',
      address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      status: 'active',
      stake: '100,000 ANV',
      blocks: '1,234',
      uptime: '99.9%'
    },
    {
      id: 2,
      name: 'Validator Node 2',
      address: '0x8ba1f109551bD432803012645Hac136c8',
      status: 'active',
      stake: '85,000 ANV',
      blocks: '1,156',
      uptime: '99.7%'
    },
    {
      id: 3,
      name: 'Validator Node 3',
      address: '0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC',
      status: 'active',
      stake: '92,500 ANV',
      blocks: '1,198',
      uptime: '99.8%'
    }
  ];

  const formatAddress = (addr) => {
    return `${addr.slice(0, 8)}...${addr.slice(-6)}`;
  };

  return (
    <Container>
      <Header>
        <Title>Validators</Title>
        <Subtitle>Active validators securing the MERKLITH network</Subtitle>
      </Header>

      <ValidatorsGrid>
        {validators.map((validator) => (
          <ValidatorCard key={validator.id}>
            <ValidatorHeader>
              <ValidatorIcon>⛏️</ValidatorIcon>
              <div>
                <ValidatorName>{validator.name}</ValidatorName>
                <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: '0.85rem', fontFamily: 'monospace' }}>
                  {formatAddress(validator.address)}
                </div>
              </div>
            </ValidatorHeader>

            <ValidatorStatus>
              <StatusDot />
              Active
            </ValidatorStatus>

            <ValidatorStats>
              <Stat>
                <StatLabel>Stake</StatLabel>
                <StatValue>{validator.stake}</StatValue>
              </Stat>
              <Stat>
                <StatLabel>Blocks</StatLabel>
                <StatValue>{validator.blocks}</StatValue>
              </Stat>
              <Stat>
                <StatLabel>Uptime</StatLabel>
                <StatValue>{validator.uptime}</StatValue>
              </Stat>
              <Stat>
                <StatLabel>Status</StatLabel>
                <StatValue style={{ color: '#00ff88' }}>Online</StatValue>
              </Stat>
            </ValidatorStats>
          </ValidatorCard>
        ))}
      </ValidatorsGrid>
    </Container>
  );
};

export default Validators;
