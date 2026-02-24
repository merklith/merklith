import { useState, useEffect, useCallback } from 'react';

const RPC_URL = process.env.REACT_APP_RPC_URL || 'http://localhost:8545';

export const useMerklithRPC = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const call = useCallback(async (method, params = []) => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch(RPC_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method,
          params,
          id: Math.floor(Math.random() * 1000000)
        })
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.error) {
        throw new Error(data.error.message || 'RPC Error');
      }

      return data.result;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { call, loading, error };
};

export const useBlockNumber = () => {
  const [blockNumber, setBlockNumber] = useState(null);
  const { call } = useMerklithRPC();

  useEffect(() => {
    const fetchBlockNumber = async () => {
      try {
        const result = await call('merklith_blockNumber');
        setBlockNumber(parseInt(result, 16));
      } catch (err) {
        console.error('Failed to fetch block number:', err);
      }
    };

    fetchBlockNumber();
    const interval = setInterval(fetchBlockNumber, 3000);
    return () => clearInterval(interval);
  }, [call]);

  return blockNumber;
};

export const useChainStats = () => {
  const [stats, setStats] = useState(null);
  const { call } = useMerklithRPC();

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const result = await call('merklith_getChainStats');
        setStats({
          blockNumber: parseInt(result.blockNumber, 16),
          accounts: result.accounts,
          chainId: parseInt(result.chainId, 16),
          blockHash: result.blockHash
        });
      } catch (err) {
        console.error('Failed to fetch chain stats:', err);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, [call]);

  return stats;
};

export const useBlock = (blockNumber) => {
  const [block, setBlock] = useState(null);
  const [loading, setLoading] = useState(false);
  const { call } = useMerklithRPC();

  useEffect(() => {
    if (!blockNumber && blockNumber !== 0) return;

    const fetchBlock = async () => {
      setLoading(true);
      try {
        const hexNumber = typeof blockNumber === 'number' 
          ? `0x${blockNumber.toString(16)}` 
          : blockNumber;
        const result = await call('merklith_getBlockByNumber', [hexNumber, true]);
        setBlock(result);
      } catch (err) {
        console.error('Failed to fetch block:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchBlock();
  }, [blockNumber, call]);

  return { block, loading };
};

export const useBlocks = (count = 10) => {
  const [blocks, setBlocks] = useState([]);
  const [loading, setLoading] = useState(false);
  const { call } = useMerklithRPC();
  const currentBlock = useBlockNumber();

  useEffect(() => {
    if (!currentBlock) return;

    const fetchBlocks = async () => {
      setLoading(true);
      try {
        const blockPromises = [];
        for (let i = 0; i < count && currentBlock - i >= 0; i++) {
          const blockNum = currentBlock - i;
          blockPromises.push(
            call('merklith_getBlockByNumber', [`0x${blockNum.toString(16)}`, false])
          );
        }
        
        const results = await Promise.all(blockPromises);
        setBlocks(results.filter(Boolean));
      } catch (err) {
        console.error('Failed to fetch blocks:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchBlocks();
  }, [currentBlock, count, call]);

  return { blocks, loading };
};

export const useTransaction = (txHash) => {
  const [tx, setTx] = useState(null);
  const [loading, setLoading] = useState(false);
  const { call } = useMerklithRPC();

  useEffect(() => {
    if (!txHash) return;

    const fetchTx = async () => {
      setLoading(true);
      try {
        const result = await call('merklith_getTransactionByHash', [txHash]);
        setTx(result);
      } catch (err) {
        console.error('Failed to fetch transaction:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchTx();
  }, [txHash, call]);

  return { tx, loading };
};

export const useAccount = (address) => {
  const [account, setAccount] = useState(null);
  const [loading, setLoading] = useState(false);
  const { call } = useMerklithRPC();

  useEffect(() => {
    if (!address) return;

    const fetchAccount = async () => {
      setLoading(true);
      try {
        const [balance, nonce] = await Promise.all([
          call('merklith_getBalance', [address]),
          call('merklith_getNonce', [address])
        ]);
        
        setAccount({
          address,
          balance,
          nonce: parseInt(nonce, 16)
        });
      } catch (err) {
        console.error('Failed to fetch account:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchAccount();
  }, [address, call]);

  return { account, loading };
};
