import { PublicKey } from "@solana/web3.js";

export const BRAND = "44 Milady" as const;
export const CLUSTER = "devnet" as const;
export const PROTOCOL_SEED = "protocol" as const;
export const MARKET_SEED = "market" as const;
export const CREDIT_SEED = "credit" as const;
export const VAULT_SEED = "vault" as const;
export const FAUCET_SEED = "faucet" as const;
export const CLAIM_SEED = "claim" as const;
export const LENDING_POOL_SEED = "lending_pool" as const;
export const LIQUIDITY_VAULT_SEED = "liquidity_vault" as const;
export const SUPPLIER_SEED = "supplier" as const;
export const BPS_DENOMINATOR = 10_000;
export const USD_MICRO = 1_000_000;
export const INDEX_SCALE_E18 = 1_000_000_000_000_000_000n;
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

export type LendingPoolSnapshot = {
  totalSuppliedUsdg: bigint;
  totalBorrowedUsdg: bigint;
  protocolReservesUsdg: bigint;
  insuranceReserveUsdg: bigint;
  badDebtUsdg: bigint;
  borrowCapUsdg: bigint;
  borrowEnabled: boolean;
  reserveFactorBps: number;
  baseRateBps: number;
  slope1Bps: number;
  slope2Bps: number;
  kinkUtilizationBps: number;
  lastBorrowAprBps: number;
  lastSupplyAprBps: number;
  liquidationCloseFactorBps: number;
  liquidationEnabled: boolean;
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

export function lendingPoolPda(programId: PublicKey, usdgMint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(LENDING_POOL_SEED), usdgMint.toBuffer()],
    programId,
  )[0];
}

export function liquidityVaultPda(programId: PublicKey, lendingPool: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(LIQUIDITY_VAULT_SEED), lendingPool.toBuffer()],
    programId,
  )[0];
}

export function supplierPositionPda(
  programId: PublicKey,
  lendingPool: PublicKey,
  supplier: PublicKey,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SUPPLIER_SEED), lendingPool.toBuffer(), supplier.toBuffer()],
    programId,
  )[0];
}

export function netPoolAssets(pool: LendingPoolSnapshot): bigint {
  const gross =
    pool.totalSuppliedUsdg +
    pool.protocolReservesUsdg +
    pool.insuranceReserveUsdg;
  if (pool.badDebtUsdg > gross) throw new Error("pool insolvent");
  return gross - pool.badDebtUsdg;
}

export function availableLiquidity(pool: LendingPoolSnapshot): bigint {
  const assets = netPoolAssets(pool);
  if (pool.totalBorrowedUsdg > assets) {
    throw new Error("pool accounting invariant violated");
  }
  return assets - pool.totalBorrowedUsdg;
}

export function availableBorrow(snapshot: RiskSnapshot): bigint {
  return snapshot.borrowLimitUsdMicro > snapshot.debtUsdg
    ? snapshot.borrowLimitUsdMicro - snapshot.debtUsdg
    : 0n;
}

export function utilizationBps(pool: LendingPoolSnapshot): bigint {
  const assets = netPoolAssets(pool);
  if (assets === 0n) return 0n;
  const value = (pool.totalBorrowedUsdg * 10_000n) / assets;
  return value > 10_000n ? 10_000n : value;
}

export function borrowAprBps(pool: LendingPoolSnapshot): number {
  const utilization = Number(utilizationBps(pool));
  const kink = pool.kinkUtilizationBps;
  if (kink <= 0 || kink >= BPS_DENOMINATOR) throw new Error("invalid utilization kink");

  if (utilization <= kink) {
    return pool.baseRateBps + Math.floor((pool.slope1Bps * utilization) / kink);
  }

  return (
    pool.baseRateBps +
    pool.slope1Bps +
    Math.floor((pool.slope2Bps * (utilization - kink)) / (BPS_DENOMINATOR - kink))
  );
}

export function supplyAprBps(pool: LendingPoolSnapshot): number {
  if (pool.totalSuppliedUsdg === 0n || pool.totalBorrowedUsdg === 0n) return 0;
  const borrowRate = BigInt(borrowAprBps(pool));
  const supplierShare = BigInt(BPS_DENOMINATOR - pool.reserveFactorBps);
  return Number(
    (borrowRate * pool.totalBorrowedUsdg * supplierShare) /
      (pool.totalSuppliedUsdg * BigInt(BPS_DENOMINATOR)),
  );
}

export function aprBpsToApyPercent(aprBps: number, compoundsPerYear = 365): number {
  const apr = aprBps / 10_000;
  return (Math.pow(1 + apr / compoundsPerYear, compoundsPerYear) - 1) * 100;
}

export function resolveRepaymentAmount(
  debtUsdg: bigint,
  requestedUsdg?: bigint,
): bigint {
  if (debtUsdg <= 0n) throw new Error("no debt to repay");
  if (requestedUsdg === undefined) return debtUsdg;
  if (requestedUsdg <= 0n || requestedUsdg > debtUsdg) {
    throw new Error("invalid repayment amount");
  }
  return requestedUsdg;
}

export function debtAfterRepayment(debtUsdg: bigint, repaymentUsdg: bigint): bigint {
  if (repaymentUsdg < 0n || repaymentUsdg > debtUsdg) {
    throw new Error("repayment exceeds debt");
  }
  return debtUsdg - repaymentUsdg;
}

export function canCloseCreditAccount(debtUsdg: bigint, collateralEntries: number): boolean {
  return debtUsdg === 0n && collateralEntries === 0;
}

export function liquidationRepayCap(
  debtUsdg: bigint,
  closeFactorBps: number,
  selectedCollateralValueUsdMicro: bigint,
  liquidationBonusBps: number,
): bigint {
  if (debtUsdg <= 0n) return 0n;
  if (closeFactorBps <= 0 || closeFactorBps > BPS_DENOMINATOR) {
    throw new Error("invalid close factor");
  }
  const closeCap = (debtUsdg * BigInt(closeFactorBps)) / BigInt(BPS_DENOMINATOR);
  const collateralCap =
    (selectedCollateralValueUsdMicro * BigInt(BPS_DENOMINATOR)) /
    BigInt(BPS_DENOMINATOR + liquidationBonusBps);
  const nonZeroCloseCap = closeCap === 0n ? 1n : closeCap;
  return [debtUsdg, nonZeroCloseCap, collateralCap].reduce((a, b) => (a < b ? a : b));
}

export function isLiquidatable(healthFactorBps: bigint | null): boolean {
  return healthFactorBps !== null && healthFactorBps <= BigInt(BPS_DENOMINATOR);
}

export function healthLabel(healthFactorBps: bigint | null): "NO DEBT" | "SAFE" | "CAUTION" | "DANGER" | "LIQUIDATABLE" {
  if (healthFactorBps === null) return "NO DEBT";
  if (healthFactorBps > 15_000n) return "SAFE";
  if (healthFactorBps > 12_000n) return "CAUTION";
  if (healthFactorBps > 10_000n) return "DANGER";
  return "LIQUIDATABLE";
}
