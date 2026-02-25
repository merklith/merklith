import { Block as IBlock } from './types';
import { randomHex } from './utils';

export class Block implements IBlock {
    public number: number;
    public hash: string;
    public parentHash: string;
    public timestamp: number;
    public transactions: string[];
    public gasLimit: string;
    public gasUsed: string;
    public size: number;

    constructor(
        number: number,
        hash: string,
        parentHash: string,
        timestamp: number,
        transactions: string[],
        gasLimit: string,
        gasUsed: string,
        size: number
    ) {
        this.number = number;
        this.hash = hash;
        this.parentHash = parentHash;
        this.timestamp = timestamp;
        this.transactions = transactions;
        this.gasLimit = gasLimit;
        this.gasUsed = gasUsed;
        this.size = size;
    }

    public static fromJSON(json: any): Block {
        return new Block(
            parseInt(json.number, 16),
            json.hash,
            json.parentHash,
            parseInt(json.timestamp, 16),
            json.transactions || [],
            json.gasLimit,
            json.gasUsed,
            parseInt(json.size, 16)
        );
    }

    public static empty(): Block {
        return new Block(
            0,
            randomHex(32),
            randomHex(32),
            Date.now(),
            [],
            "0x0",
            "0x0",
            0
        );
    }
}
