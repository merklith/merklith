import React, { useState } from 'react';
import styled from 'styled-components';
import { Link } from 'react-router-dom';

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

const InfoBox = styled.div`
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 12px;
  padding: 2rem;
  text-align: center;
`;

const InfoTitle = styled.h3`
  color: #00d4ff;
  margin-bottom: 1rem;
`;

const InfoText = styled.p`
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.6;
`;

const TransactionList = () => {
  return (
    <Container>
      <Header>
        <Title>Transactions</Title>
        <Subtitle>View all transactions on the MERKLITH blockchain</Subtitle>
      </Header>

      <InfoBox>
        <InfoTitle>🚧 Coming Soon</InfoTitle>
        <InfoText>
          Transaction indexing is being implemented. <br/>
          For now, you can view transactions within each block.
        </InfoText>
        <Link 
          to="/blocks" 
          style={{ 
            display: 'inline-block', 
            marginTop: '1.5rem',
            color: '#00d4ff',
            textDecoration: 'none'
          }}
        >
          View Blocks →
        </Link>
      </InfoBox>
    </Container>
  );
};

export default TransactionList;
