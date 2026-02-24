import { Address, Hash, Transaction } from './types';

export interface RpcRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

export interface RpcResponse<T> {
  result?: T;
  error?: {
    code: number;
    message: string;
  };
}

export class MerklithClient {
  private url: string;
  private chainId?: number;

  constructor(url: string) {
    this.url = url;
  }

  static async connect(url: string): Promise<MerklithClient> {
    const client = new MerklithClient(url);
    client.chainId = await client.getChainId();
    return client;
  }

  async request<T>(method: string, params: any[]): Promise<T> {
    const request: RpcRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: 1,
    };

    const response = await fetch(this.url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    const data: RpcResponse<T> = await response.json();

    if (data.error) {
      throw new Error(`RPC error ${data.error.code}: ${data.error.message}`);
    }

    if (data.result === undefined) {
      throw new Error('Empty result');
    }

    return data.result;
  }

  async getChainId(): Promise<number> {
    const hex = await this.request<string>('eth_chainId', []);
    return parseInt(hex, 16);
  }

  async getBlockNumber(): Promise<number> {
    const hex = await this.request<string>('eth_blockNumber', []);
    return parseInt(hex, 16);
  }

  async getBalance(address: Address): Promise<bigint> {
    const hex = await this.request<string>('eth_getBalance', [address, 'latest']);
    return BigInt(hex);
  }

  async getTransactionCount(address: Address): Promise<number> {
    const hex = await this.request<string>('eth_getTransactionCount', [address, 'latest']);
    return parseInt(hex, 16);
  }

  async getGasPrice(): Promise<bigint> {
    const hex = await this.request<string>('eth_gasPrice', []);
    return BigInt(hex);
  }

  async getCode(address: Address): Promise<string> {
    return this.request<string>('eth_getCode', [address, 'latest']);
  }

  async sendRawTransaction(signedTx: string): Promise<Hash> {
    return this.request<string>('eth_sendRawTransaction', [signedTx]);
  }

  async getTransactionReceipt(hash: Hash): Promise<any | null> {
    return this.request<any | null>('eth_getTransactionReceipt', [hash]);
  }

  async waitForTransaction(
    hash: Hash,
    timeoutMs: number = 60000,
    intervalMs: number = 1000
  ): Promise<any> {
    const start = Date.now();
    
    while (Date.now() - start < timeoutMs) {
      const receipt = await this.getTransactionReceipt(hash);
      if (receipt) {
        return receipt;
      }
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
    
    throw new Error('Transaction timeout');
  }

  async estimateGas(tx: Partial<Transaction>): Promise<bigint> {
    const hex = await this.request<string>('eth_estimateGas', [tx]);
    return BigInt(hex);
  }

  async call(tx: Partial<Transaction>, block: string = 'latest'): Promise<string> {
    return this.request<string>('eth_call', [tx, block]);
  }
}

export { MerklithClient as Client };
