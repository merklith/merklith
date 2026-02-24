import { SDKConfig, RPCRequest, RPCResponse, Block, Transaction, Account, TransactionReceipt } from './types';

export class MerklithSDK {
  private config: SDKConfig;
  private requestId = 0;

  constructor(config: SDKConfig) {
    this.config = {
      timeout: 30000,
      ...config
    };
  }

  /**
   * Make RPC call
   */
  async call(method: string, params: any[] = []): Promise<any> {
    const request: RPCRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: ++this.requestId
    };

    try {
      const response = await fetch(this.config.rpcUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(request)
      });

      const data: RPCResponse = await response.json();

      if (data.error) {
        throw new Error(`RPC Error: ${data.error.message}`);
      }

      return data.result;
    } catch (error) {
      throw new Error(`Failed to call ${method}: ${error}`);
    }
  }

  /**
   * Get chain ID
   */
  async getChainId(): Promise<number> {
    const result = await this.call('eth_chainId');
    return parseInt(result, 16);
  }

  /**
   * Get block number
   */
  async getBlockNumber(): Promise<number> {
    const result = await this.call('eth_blockNumber');
    return parseInt(result, 16);
  }

  /**
   * Get block by number
   */
  async getBlock(blockNumber: number | string): Promise<Block> {
    const block = await this.call('eth_getBlockByNumber', [
      typeof blockNumber === 'number' ? `0x${blockNumber.toString(16)}` : blockNumber,
      false
    ]);
    return this.formatBlock(block);
  }

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<Transaction> {
    const tx = await this.call('eth_getTransactionByHash', [hash]);
    return this.formatTransaction(tx);
  }

  /**
   * Get transaction receipt
   */
  async getTransactionReceipt(hash: string): Promise<TransactionReceipt> {
    return await this.call('eth_getTransactionReceipt', [hash]);
  }

  /**
   * Get account balance
   */
  async getBalance(address: string): Promise<string> {
    return await this.call('eth_getBalance', [address, 'latest']);
  }

  /**
   * Get account nonce
   */
  async getNonce(address: string): Promise<number> {
    const result = await this.call('eth_getTransactionCount', [address, 'latest']);
    return parseInt(result, 16);
  }

  /**
   * Send raw transaction
   */
  async sendRawTransaction(signedTx: string): Promise<string> {
    return await this.call('eth_sendRawTransaction', [signedTx]);
  }

  /**
   * Estimate gas
   */
  async estimateGas(tx: any): Promise<string> {
    return await this.call('eth_estimateGas', [tx]);
  }

  /**
   * Get gas price
   */
  async getGasPrice(): Promise<string> {
    return await this.call('eth_gasPrice');
  }

  /**
   * Format block
   */
  private formatBlock(block: any): Block {
    return {
      number: parseInt(block.number, 16),
      hash: block.hash,
      parentHash: block.parentHash,
      timestamp: parseInt(block.timestamp, 16),
      transactions: block.transactions,
      gasLimit: block.gasLimit,
      gasUsed: block.gasUsed,
      size: parseInt(block.size, 16)
    };
  }

  /**
   * Format transaction
   */
  private formatTransaction(tx: any): Transaction {
    return {
      hash: tx.hash,
      from: tx.from,
      to: tx.to,
      value: tx.value,
      gasLimit: parseInt(tx.gas, 16),
      gasPrice: tx.gasPrice,
      nonce: parseInt(tx.nonce, 16),
      data: tx.input,
      chainId: parseInt(tx.chainId, 16)
    };
  }
}
