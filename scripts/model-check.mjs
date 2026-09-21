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

const hf = health(liquidationCapacity, 2_000_000_000n);
assert.equal(hf, 19_134n); // 1.9134 health factor represented in bps.
assert(hf > BPS);

console.log("44 Milady M2-M5 model checks: PASS");
console.log({
  collateralUsd: Number(total) / 1e6,
  maxBorrowUsd: Number(borrowLimit) / 1e6,
  liquidationCapacityUsd: Number(liquidationCapacity) / 1e6,
  healthFactor: Number(hf) / 1e4,
});
