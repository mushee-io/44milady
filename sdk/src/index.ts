import { PublicKey } from "@solana/web3.js";

export const BRAND = "44 Milady" as const;
export const CLUSTER = "devnet" as const;
export const PROTOCOL_SEED = "protocol" as const;
export const MARKET_SEED = "market" as const;
export const CREDIT_SEED = "credit" as const;
export const VAULT_SEED = "vault" as const;
export const FAUCET_SEED = "faucet" as const;
export const CLAIM_SEED = "claim" as const;
export const BPS_DENOMINATOR = 10_000;
export const USD_MICRO = 1_000_000;
export const MAX_COLLATERAL_ASSETS = 8;

export type ProtocolStatus = {
  initialized: boolean;
  paused: boolean;
  version: number;
};

export type MarketRisk = {
  symbol: string;
  ltvBps: number;
  liquidationThresholdBps: number;
  liquidationBonusBps: number;
  maxConfidenceBps: number;
  maxPriceAgeSecs: number;
};

export type RiskSnapshot = {
  collateralValueUsdMicro: bigint;
  borrowLimitUsdMicro: bigint;
  liquidationCapacityUsdMicro: bigint;
  debtUsdg: bigint;
  healthFactorBps: bigint | null;
};

export function encodeSymbol(symbol: string): number[] {
  const bytes = new TextEncoder().encode(symbol);
  if (bytes.length === 0 || bytes.length > 8) throw new Error("symbol must be 1-8 bytes");
  return [...bytes, ...new Array(8 - bytes.length).fill(0)];
}

export function decodeSymbol(bytes: number[]): string {
  return new TextDecoder().decode(Uint8Array.from(bytes.filter((b) => b !== 0)));
}

export function protocolPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(PROTOCOL_SEED)], programId)[0];
}

export function marketPda(programId: PublicKey, mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(MARKET_SEED), mint.toBuffer()],
    programId,
  )[0];
}

export function creditPda(programId: PublicKey, owner: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(CREDIT_SEED), owner.toBuffer()],
    programId,
  )[0];
}

export function collateralVaultPda(
  programId: PublicKey,
  creditAccount: PublicKey,
  market: PublicKey,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), creditAccount.toBuffer(), market.toBuffer()],
    programId,
  )[0];
}

export function healthLabel(healthFactorBps: bigint | null): "NO DEBT" | "SAFE" | "CAUTION" | "DANGER" | "LIQUIDATABLE" {
  if (healthFactorBps === null) return "NO DEBT";
  if (healthFactorBps > 15_000n) return "SAFE";
  if (healthFactorBps > 12_000n) return "CAUTION";
  if (healthFactorBps > 10_000n) return "DANGER";
  return "LIQUIDATABLE";
}
