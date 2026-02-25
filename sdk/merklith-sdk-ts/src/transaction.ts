import { Transaction as ITransaction, Hash, Address } from './types';
import { randomHex } from './utils';

export class Transaction implements ITransaction {
    public hash: string;
    public from: string;
    public to: string | null;
    public value: string;
    public gasLimit: number;
    public gasPrice: string;
    public nonce: number;
    public data: string;
    public chainId: number;
    public signature?: string;

    constructor(
        from: string,
        to: string | null,
        value: string,
        gasLimit: number,
        gasPrice: string,
        nonce: number,
        data: string = '0x',
        chainId: number = 17001,
        signature?: string,
        hash?: string
    ) {
        this.from = from;
        this.to = to;
        this.value = value;
        this.gasLimit = gasLimit;
        this.gasPrice = gasPrice;
        this.nonce = nonce;
        this.data = data;
        this.chainId = chainId;
        this.signature = signature;
        this.hash = hash || this.calculateHash();
    }

    private calculateHash(): string {
        // Placeholder for hash calculation
        // In a real implementation, this would use RLP encoding + Keccak/Blake3
        return randomHex(32);
    }

    public static fromJSON(json: any): Transaction {
        return new Transaction(
            json.from,
            json.to,
            json.value,
            parseInt(json.gasLimit || json.gas || '0', 16),
            json.gasPrice,
            parseInt(json.nonce || '0', 16),
            json.data || json.input,
            parseInt(json.chainId || '0x4269', 16),
            json.signature,
            json.hash
        );
    }
}
