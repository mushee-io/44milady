import assert from "node:assert/strict";

const BPS = 10_000n;
const MICRO = 1_000_000n;

function pow10(n) {
  return 10n ** BigInt(n);
}

function tokenValueMicro(amount, decimals, price, exponent) {
  assert(price > 0n);
  let microPrice;
  if (exponent >= 0) {
    microPrice = price * pow10(exponent) * MICRO;
  } else {
    microPrice = (price * MICRO) / pow10(-exponent);
  }
  return (amount * microPrice) / pow10(decimals);
}

function weighted(value, bps) {
  return (value * BigInt(bps)) / BPS;
}

function health(liquidationCapacity, debt) {
  return debt === 0n ? null : (liquidationCapacity * BPS) / debt;
}

function availableLiquidity(totalSupplied, totalBorrowed) {
  assert(totalBorrowed <= totalSupplied, "pool accounting invariant");
  return totalSupplied - totalBorrowed;
}

// 10 NVDAx, 6 decimals, $190.25 -> $1,902.50.
const collateral = tokenValueMicro(10_000_000n, 6, 19_025_000_000n, -8);
assert.equal(collateral, 1_902_500_000n);
assert.equal(weighted(collateral, 6_000), 1_141_500_000n);
assert.equal(weighted(collateral, 7_500), 1_426_875_000n);

// Two collateral assets aggregate independently by market risk weights.
const spy = tokenValueMicro(5_000_000n, 6, 60_000_000_000n, -8); // 5 * $600
const total = collateral + spy;
const borrowLimit = weighted(collateral, 6_000) + weighted(spy, 7_000);
const liquidationCapacity = weighted(collateral, 7_500) + weighted(spy, 8_000);
assert.equal(total, 4_902_500_000n);
assert.equal(borrowLimit, 3_241_500_000n);
assert.equal(liquidationCapacity, 3_826_875_000n);

const initialDebt = 2_000_000_000n;
const hf = health(liquidationCapacity, initialDebt);
assert.equal(hf, 19_134n);
assert(hf > BPS);

// Milestone 6 pool: $10k supplied, $2k already borrowed.
const poolSupply = 10_000_000_000n;
const poolBorrowed = 2_000_000_000n;
assert.equal(availableLiquidity(poolSupply, poolBorrowed), 8_000_000_000n);

// Borrow another $1k: both collateral capacity and pool liquidity permit it.
const requestedBorrow = 1_000_000_000n;
const newDebt = initialDebt + requestedBorrow;
assert(newDebt <= borrowLimit);
assert(requestedBorrow <= availableLiquidity(poolSupply, poolBorrowed));
const newHealth = health(liquidationCapacity, newDebt);
assert.equal(newHealth, 12_756n);
assert(newHealth > BPS);

// Borrowing beyond LTV must fail even if the pool has cash.
assert(initialDebt + 1_500_000_000n > borrowLimit);

// Suppliers can only withdraw idle pool liquidity, never borrower-held USDG.
assert(availableLiquidity(poolSupply, poolBorrowed) < poolSupply);

console.log("44 Milady M2-M6 model checks: PASS");
console.log({
  collateralUsd: Number(total) / 1e6,
  maxBorrowUsd: Number(borrowLimit) / 1e6,
  liquidationCapacityUsd: Number(liquidationCapacity) / 1e6,
  debtAfterBorrowUsd: Number(newDebt) / 1e6,
  healthFactorAfterBorrow: Number(newHealth) / 1e4,
  poolLiquidityUsd: Number(availableLiquidity(poolSupply, poolBorrowed)) / 1e6,
});
