import React, { useState, useEffect } from 'react';
import { useSearchParams, Link } from 'react-router-dom';
import styled from 'styled-components';
import { useMerklithRPC } from '../hooks/useMerklith';

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

const SearchQuery = styled.code`
  color: #00d4ff;
  background: rgba(0, 212, 255, 0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
`;

const ResultCard = styled(Link)`
  display: block;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1rem;
  text-decoration: none;
  color: inherit;
  transition: all 0.2s;

  &:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(0, 212, 255, 0.3);
  }
`;

const ResultType = styled.div`
  display: inline-block;
  background: rgba(0, 212, 255, 0.1);
  color: #00d4ff;
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.85rem;
  font-weight: 500;
  margin-bottom: 0.75rem;
`;

const ResultTitle = styled.h3`
  font-size: 1.1rem;
  margin-bottom: 0.5rem;
  color: #ffffff;
  font-family: monospace;
`;

const ResultDetails = styled.div`
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.9rem;
`;

const LoadingText = styled.div`
  text-align: center;
  padding: 3rem;
  color: rgba(255, 255, 255, 0.5);
`;

const NoResults = styled.div`
  text-align: center;
  padding: 4rem;
  color: rgba(255, 255, 255, 0.5);
`;

const ErrorText = styled.div`
  text-align: center;
  padding: 3rem;
  color: #ff6b6b;
`;

const SearchResults = () => {
  const [searchParams] = useSearchParams();
  const query = searchParams.get('q') || '';
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const { call } = useMerklithRPC();

  useEffect(() => {
    if (!query) return;

    const search = async () => {
      setLoading(true);
      setError(null);
      setResults([]);

      const searchResults = [];

      // Try to parse as block number
      const blockNum = parseInt(query);
      if (!isNaN(blockNum)) {
        try {
          const block = await call('merklith_getBlockByNumber', [
            `0x${blockNum.toString(16)}`,
            false
          ]);
          if (block) {
            searchResults.push({
              type: 'Block',
              title: `Block #${blockNum}`,
              link: `/block/${blockNum}`,
              details: `Hash: ${block.hash?.slice(0, 30)}... | ${block.transactions?.length || 0} transactions`
            });
          }
        } catch (e) {
          // Block not found
        }
      }

      // Try as transaction hash
      if (query.startsWith('0x') && query.length === 66) {
        try {
          const tx = await call('merklith_getTransactionByHash', [query]);
          if (tx) {
            searchResults.push({
              type: 'Transaction',
              title: tx.hash,
              link: `/tx/${tx.hash}`,
              details: `From: ${tx.from?.slice(0, 20)}... | To: ${tx.to?.slice(0, 20)}...`
            });
          }
        } catch (e) {
          // Tx not found
        }
      }

      // Try as address
      if (query.startsWith('0x') && query.length === 42) {
        try {
          const balance = await call('merklith_getBalance', [query]);
          searchResults.push({
            type: 'Account',
            title: query,
            link: `/address/${query}`,
            details: `Balance: ${parseInt(balance, 16) / 1e18} ANV`
          });
        } catch (e) {
          // Address not found
        }
      }

      // Try as block hash
      if (query.startsWith('0x') && query.length === 66) {
        // Block hash search would go here
      }

      setResults(searchResults);
      setLoading(false);
    };

    search();
  }, [query, call]);

  return (
    <Container>
      <Header>
        <Title>Search Results</Title>
        {query && (
          <p style={{ color: 'rgba(255,255,255,0.6)' }}>
            Searching for: <SearchQuery>{query}</SearchQuery>
          </p>
        )}
      </Header>

      {loading && <LoadingText>Searching...</LoadingText>}
      
      {error && <ErrorText>{error}</ErrorText>}

      {!loading && !error && results.length === 0 && (
        <NoResults>
          🔍 No results found for "{query}"
          <br />
          <span style={{ fontSize: '0.9rem', marginTop: '1rem', display: 'block' }}>
            Try searching for a block number, transaction hash, or address
          </span>
        </NoResults>
      )}

      {results.map((result, idx) => (
        <ResultCard key={idx} to={result.link}>
          <ResultType>{result.type}</ResultType>
          <ResultTitle>{result.title}</ResultTitle>
          <ResultDetails>{result.details}</ResultDetails>
        </ResultCard>
      ))}
    </Container>
  );
};

export default SearchResults;
