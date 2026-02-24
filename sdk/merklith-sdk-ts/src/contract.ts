import { MerklithClient } from './client';
import { Address } from './types';

export interface ContractOptions {
  address: Address;
  abi?: any[];
}

export interface CallOptions {
  from?: Address;
  gas?: bigint;
  gasPrice?: bigint;
  value?: bigint;
}

export interface SendOptions extends CallOptions {
  nonce?: number;
}

export class Contract {
  private client: MerklithClient;
  public address: Address;
  private abi?: any[];

  constructor(client: MerklithClient, options: ContractOptions) {
    this.client = client;
    this.address = options.address;
    this.abi = options.abi;
  }

  async call(data: string, options: CallOptions = {}): Promise<string> {
    return this.client.call({
      to: this.address,
      data,
      from: options.from,
      gas: options.gas ? `0x${options.gas.toString(16)}` : undefined,
      gasPrice: options.gasPrice ? `0x${options.gasPrice.toString(16)}` : undefined,
      value: options.value ? `0x${options.value.toString(16)}` : undefined,
    });
  }

  async sendTransaction(data: string, options: SendOptions = {}): Promise<string> {
    // This would need a wallet to sign the transaction
    // For now, just return a placeholder
    throw new Error('Send transaction requires wallet integration');
  }

  async getCode(): Promise<string> {
    return this.client.getCode(this.address);
  }

  async exists(): Promise<boolean> {
    const code = await this.getCode();
    return code !== '0x' && code.length > 2;
  }
}

export class ContractDeployer {
  private client: MerklithClient;
  private bytecode: string;
  private abi?: any[];

  constructor(client: MerklithClient, bytecode: string, abi?: any[]) {
    this.client = client;
    this.bytecode = bytecode;
    this.abi = abi;
  }

  async deploy(args: string = '0x', options: SendOptions = {}): Promise<Contract> {
    // Combine bytecode with constructor args
    const data = this.bytecode + args.replace('0x', '');

    // This would need wallet integration
    throw new Error('Contract deployment requires wallet integration');
  }
}
