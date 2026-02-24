export type Hex = `0x${string}`;
export type Address = Hex;
export type Hash = Hex;

export interface Transaction {
  hash: string;
  from: string;
  to: string | null;
  value: string;
  gasLimit: number;
  gasPrice: string;
  nonce: number;
  data: string;
  chainId: number;
  signature?: string;
}

export interface Block {
  number: number;
  hash: string;
  parentHash: string;
  timestamp: number;
  transactions: string[];
  gasLimit: string;
  gasUsed: string;
  size: number;
}

export interface Account {
  address: string;
  balance: string;
  nonce: number;
  code?: string;
}

export interface Log {
  address: string;
  topics: string[];
  data: string;
  blockNumber: number;
  transactionHash: string;
  logIndex: number;
}

export interface TransactionReceipt {
  transactionHash: string;
  blockNumber: number;
  blockHash: string;
  gasUsed: string;
  status: boolean;
  logs: Log[];
}

export interface RPCRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

export interface RPCResponse {
  jsonrpc: string;
  result?: any;
  error?: {
    code: number;
    message: string;
  };
  id: number;
}

export interface SDKConfig {
  rpcUrl: string;
  chainId: number;
  timeout?: number;
}

export interface SendTransactionOptions {
  to?: string;
  value?: string;
  data?: string;
  gasLimit?: number;
  gasPrice?: string;
}
