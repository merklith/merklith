import { Address, Hash, Hex } from './types';

export function formatEther(wei: bigint): string {
  const ether = Number(wei) / 1e18;
  return ether.toFixed(6);
}

export function parseEther(ether: string): bigint {
  return BigInt(Math.floor(parseFloat(ether) * 1e18));
}

export function formatUnits(value: bigint, decimals: number): string {
  const divisor = BigInt(10) ** BigInt(decimals);
  const integer = value / divisor;
  const fraction = value % divisor;
  
  const fractionStr = fraction.toString().padStart(decimals, '0');
  const trimmedFraction = fractionStr.replace(/0+$/, '');
  
  return trimmedFraction.length > 0
    ? `${integer}.${trimmedFraction}`
    : integer.toString();
}

export function parseUnits(value: string, decimals: number): bigint {
  const [integer, fraction = ''] = value.split('.');
  const paddedFraction = fraction.padEnd(decimals, '0');
  return BigInt(integer + paddedFraction);
}

export function isAddress(value: string): value is Address {
  return /^0x[a-fA-F0-9]{40}$/.test(value);
}

export function isHash(value: string): value is Hash {
  return /^0x[a-fA-F0-9]{64}$/.test(value);
}

export function getAddress(address: string): Address {
  if (!isAddress(address)) {
    throw new Error('Invalid address');
  }
  return address.toLowerCase() as Address;
}

export function hexToBytes(hex: Hex): Uint8Array {
  const clean = hex.replace('0x', '');
  return new Uint8Array(clean.match(/.{2}/g)!.map(b => parseInt(b, 16)));
}

export function bytesToHex(bytes: Uint8Array): Hex {
  return ('0x' + Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('')) as Hex;
}

export function keccak256(data: Uint8Array): Hash {
  // Would use actual blake3 in production
  return ('0x' + '0'.repeat(64)) as Hash;
}

export function formatAddress(address: Address): string {
  const addr = address.slice(2);
  return `${address.slice(0, 6)}...${addr.slice(-4)}`;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
