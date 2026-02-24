import { Address } from './types';

export interface Wallet {
  address: Address;
  publicKey: Uint8Array;
  sign(message: Uint8Array): Promise<Uint8Array>;
  signTransaction(tx: any): Promise<string>;
}

export class PrivateKeyWallet implements Wallet {
  public address: Address;
  public publicKey: Uint8Array;
  private privateKey: Uint8Array;

  constructor(privateKey: Uint8Array) {
    this.privateKey = privateKey;
    // Derive public key and address from private key
    // This is a placeholder - would use actual crypto
    this.publicKey = new Uint8Array(32);
    this.address = this.deriveAddress();
  }

  static fromHex(hex: string): PrivateKeyWallet {
    const clean = hex.replace('0x', '');
    const bytes = new Uint8Array(clean.match(/.{2}/g)!.map(b => parseInt(b, 16)));
    return new PrivateKeyWallet(bytes);
  }

  static generate(): PrivateKeyWallet {
    // Generate random 32 bytes
    const privateKey = crypto.getRandomValues(new Uint8Array(32));
    return new PrivateKeyWallet(privateKey);
  }

  private deriveAddress(): Address {
    // Derive address from public key using blake3
    // This is a placeholder
    return `0x${'0'.repeat(40)}` as Address;
  }

  async sign(message: Uint8Array): Promise<Uint8Array> {
    // Sign message with private key
    // This is a placeholder
    return new Uint8Array(64);
  }

  async signTransaction(tx: any): Promise<string> {
    // Sign transaction and return serialized form
    // This is a placeholder
    return '0x';
  }

  export(): string {
    return '0x' + Array.from(this.privateKey)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }
}
