import assert from "node:assert/strict";

const BPS = 10_000n;
const MICRO = 1_000_000n;
const YEAR = 31_536_000n;

function pow10(n) { return 10n ** BigInt(n); }

function tokenValueMicro(amount, decimals, price, exponent) {
  assert(price > 0n);
  const microPrice = exponent >= 0
    ? price * pow10(exponent) * MICRO
    : (price * MICRO) / pow10(-exponent);
  return (amount * microPrice) / pow10(decimals);
}

function weighted(value, bps) {
  return (value * BigInt(bps)) / BPS;
}

function health(liquidationCapacity, debt) {
  return debt === 0n ? null : (liquidationCapacity * BPS) / debt;
}

function assets(supplied, reserves = 0n, insurance = 0n, badDebt = 0n) {
  const gross = supplied + reserves + insurance;
  assert(badDebt <= gross, "pool insolvent");
  return gross - badDebt;
}

function availableLiquidity(
  supplied,
  borrowed,
  reserves = 0n,
  insurance = 0n,
  badDebt = 0n,
) {
  const totalAssets = assets(supplied, reserves, insurance, badDebt);
  assert(borrowed <= totalAssets, "pool accounting invariant");
  return totalAssets - borrowed;
}

function utilizationBps(
  supplied,
  borrowed,
  reserves = 0n,
  insurance = 0n,
  badDebt = 0n,
) {
  const totalAssets = assets(supplied, reserves, insurance, badDebt);
  if (totalAssets === 0n) return 0n;
  const util = (borrowed * BPS) / totalAssets;
  return util > BPS ? BPS : util;
}

function borrowAprBps({ supplied, borrowed, reserves = 0n, base, slope1, slope2, kink }) {
  const util = utilizationBps(supplied, borrowed, reserves);
  const k = BigInt(kink);
  if (util <= k) return BigInt(base) + (BigInt(slope1) * util) / k;
  return (
    BigInt(base) +
    BigInt(slope1) +
    (BigInt(slope2) * (util - k)) / (BPS - k)
  );
}

function supplyAprBps(pool) {
  if (pool.supplied === 0n || pool.borrowed === 0n) return 0n;
  const br = borrowAprBps(pool);
  return (
    br *
    pool.borrowed *
    (BPS - BigInt(pool.reserveFactor))
  ) / (pool.supplied * BPS);
}

function accrueOneYear(pool) {
  const rate = borrowAprBps(pool);
  const gross = (pool.borrowed * rate * YEAR) / (YEAR * BPS);
  const reserve = (gross * BigInt(pool.reserveFactor)) / BPS;
  const supplier = gross - reserve;
  return {
    ...pool,
    borrowed: pool.borrowed + gross,
    supplied: pool.supplied + supplier,
    reserves: (pool.reserves ?? 0n) + reserve,
    gross,
    reserve,
    supplier,
  };
}

function repay(pool, debt, requested = null) {
  assert(debt > 0n, "no debt");
  const amount = requested === null ? debt : requested;
  assert(amount > 0n && amount <= debt, "invalid repay amount");
  assert(amount <= pool.borrowed, "pool debt invariant");
  return {
    pool: { ...pool, borrowed: pool.borrowed - amount },
    debt: debt - amount,
    amount,
  };
}

function liquidationCap(debt, closeFactorBps, collateralUsd, bonusBps) {
  const closeCap = (debt * BigInt(closeFactorBps)) / BPS || 1n;
  const collateralCap =
    (collateralUsd * BPS) / (BPS + BigInt(bonusBps));
  return [debt, closeCap, collateralCap].reduce((a, b) => (a < b ? a : b));
}

function absorbBadDebt(pool, debt) {
  assert(debt > 0n && debt <= pool.borrowed);
  const reserveCover = debt < pool.reserves ? debt : pool.reserves;
  const afterReserve = debt - reserveCover;
  const insuranceCover = afterReserve < pool.insurance ? afterReserve : pool.insurance;
  const uncovered = afterReserve - insuranceCover;
  return {
    ...pool,
    borrowed: pool.borrowed - debt,
    reserves: pool.reserves - reserveCover,
    insurance: pool.insurance - insuranceCover,
    badDebt: pool.badDebt + uncovered,
    reserveCover,
    insuranceCover,
    uncovered,
  };
}

// M2-M5 collateral/risk model.
const collateral = tokenValueMicro(10_000_000n, 6, 19_025_000_000n, -8);
assert.equal(collateral, 1_902_500_000n);
assert.equal(weighted(collateral, 6_000), 1_141_500_000n);
assert.equal(weighted(collateral, 7_500), 1_426_875_000n);

const spy = tokenValueMicro(5_000_000n, 6, 60_000_000_000n, -8);
const total = collateral + spy;
const borrowLimit = weighted(collateral, 6_000) + weighted(spy, 7_000);
const liquidationCapacity = weighted(collateral, 7_500) + weighted(spy, 8_000);
assert.equal(total, 4_902_500_000n);
assert.equal(borrowLimit, 3_241_500_000n);
assert.equal(liquidationCapacity, 3_826_875_000n);

const initialDebt = 2_000_000_000n;
assert.equal(health(liquidationCapacity, initialDebt), 19_134n);

// M6 borrowing model.
const poolSupply = 10_000_000_000n;
const poolBorrowed = 2_000_000_000n;
assert.equal(availableLiquidity(poolSupply, poolBorrowed), 8_000_000_000n);

const requestedBorrow = 1_000_000_000n;
const newDebt = initialDebt + requestedBorrow;
assert(newDebt <= borrowLimit);
assert(requestedBorrow <= availableLiquidity(poolSupply, poolBorrowed));
assert.equal(health(liquidationCapacity, newDebt), 12_756n);

// M7 utilization curve: 2% base + 8% slope to 80% kink + 50% jump slope.
const ratePool = {
  supplied: 10_000_000_000n,
  borrowed: 8_000_000_000n,
  reserves: 0n,
  base: 200,
  slope1: 800,
  slope2: 5_000,
  kink: 8_000,
  reserveFactor: 1_000,
};

assert.equal(utilizationBps(ratePool.supplied, ratePool.borrowed), 8_000n);
assert.equal(borrowAprBps(ratePool), 1_000n); // 10% borrow APR at kink.
assert.equal(supplyAprBps(ratePool), 720n);   // 7.2% supplier APR after 10% reserve cut.

const highUtil = { ...ratePool, borrowed: 9_000_000_000n };
assert.equal(borrowAprBps(highUtil), 3_500n); // 35% at 90% utilization.

const accrued = accrueOneYear(ratePool);
assert.equal(accrued.gross, 800_000_000n);
assert.equal(accrued.reserve, 80_000_000n);
assert.equal(accrued.supplier, 720_000_000n);
assert.equal(accrued.borrowed, 8_800_000_000n);
assert.equal(accrued.supplied, 10_720_000_000n);
assert.equal(accrued.reserves, 80_000_000n);

// Balance-sheet identity: accrued interest does not invent physical cash.
assert.equal(
  availableLiquidity(accrued.supplied, accrued.borrowed, accrued.reserves),
  2_000_000_000n,
);

// M8 repayment restores pool liquidity exactly by the amount repaid.
const partial = repay(accrued, 3_000_000_000n, 1_000_000_000n);
assert.equal(partial.debt, 2_000_000_000n);
assert.equal(partial.pool.borrowed, accrued.borrowed - 1_000_000_000n);
assert.equal(
  availableLiquidity(partial.pool.supplied, partial.pool.borrowed, partial.pool.reserves),
  3_000_000_000n,
);

// MAX repayment clears the synchronized debt and makes the position debt-free.
const maxRepay = repay(partial.pool, partial.debt);
assert.equal(maxRepay.debt, 0n);
assert.equal(maxRepay.amount, 2_000_000_000n);
assert.equal(
  availableLiquidity(maxRepay.pool.supplied, maxRepay.pool.borrowed, maxRepay.pool.reserves),
  5_000_000_000n,
);

// M9 liquidation: 50% close factor and 5% bonus.
const unhealthyDebt = 3_000_000_000n;
const unhealthyCapacity = 2_700_000_000n;
assert.equal(health(unhealthyCapacity, unhealthyDebt), 9_000n);
const liqCap = liquidationCap(
  unhealthyDebt,
  5_000,
  2_000_000_000n,
  500,
);
assert.equal(liqCap, 1_500_000_000n);

// Bad debt uses protocol reserves first, then insurance, then records only
// the uncovered deficit.
const insolventPool = {
  supplied: 10_000_000_000n,
  borrowed: 3_000_000_000n,
  reserves: 100_000_000n,
  insurance: 200_000_000n,
  badDebt: 0n,
};
const beforeWriteoff = availableLiquidity(
  insolventPool.supplied,
  insolventPool.borrowed,
  insolventPool.reserves,
  insolventPool.insurance,
  insolventPool.badDebt,
);
const absorbed = absorbBadDebt(insolventPool, 500_000_000n);
assert.equal(absorbed.reserveCover, 100_000_000n);
assert.equal(absorbed.insuranceCover, 200_000_000n);
assert.equal(absorbed.uncovered, 200_000_000n);
assert.equal(absorbed.badDebt, 200_000_000n);
assert.equal(
  availableLiquidity(
    absorbed.supplied,
    absorbed.borrowed,
    absorbed.reserves,
    absorbed.insurance,
    absorbed.badDebt,
  ),
  beforeWriteoff,
);

// Real USDG recapitalization removes the deficit and increases physical liquidity.
absorbed.badDebt -= 200_000_000n;
assert.equal(
  availableLiquidity(
    absorbed.supplied,
    absorbed.borrowed,
    absorbed.reserves,
    absorbed.insurance,
    absorbed.badDebt,
  ),
  beforeWriteoff + 200_000_000n,
);

console.log("44 Milady M2-M9 model checks: PASS");
console.log({
  collateralUsd: Number(total) / 1e6,
  maxBorrowUsd: Number(borrowLimit) / 1e6,
  m7BorrowAprPctAtKink: Number(borrowAprBps(ratePool)) / 100,
  m7SupplyAprPctAtKink: Number(supplyAprBps(ratePool)) / 100,
  annualGrossInterestUsd: Number(accrued.gross) / 1e6,
  annualSupplierInterestUsd: Number(accrued.supplier) / 1e6,
  annualProtocolReserveUsd: Number(accrued.reserve) / 1e6,
  partialRepayUsd: Number(partial.amount) / 1e6,
  remainingDebtUsd: Number(partial.debt) / 1e6,
  maxRepayUsd: Number(maxRepay.amount) / 1e6,
  liquidationCapUsd: Number(liqCap) / 1e6,
  uncoveredBadDebtUsd: Number(absorbed.uncovered) / 1e6,
});
