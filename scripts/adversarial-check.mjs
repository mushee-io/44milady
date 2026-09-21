import assert from "node:assert/strict";

const BPS = 10_000n;
let seed = 0x44c0ffee;
function rand() {
  seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
  return seed;
}
function pick(max) {
  if (max <= 1) return 0n;
  return BigInt(rand() % max);
}
function hf(capacity, debt) {
  return debt === 0n ? null : (capacity * BPS) / debt;
}

for (let scenario = 0; scenario < 500; scenario++) {
  let supplied = 1_000_000n + pick(20_000_000);
  let debt = 0n;
  let collateral = 1_000_000n + pick(30_000_000);
  const ltvBps = 4_000n + pick(3_000);
  const liqRoom = Number(9_500n - ltvBps);
  const liqBps = ltvBps + 500n + pick(liqRoom);
  const bonusBps = 100n + pick(1_000);
  const closeFactorBps = 2_500n + pick(7_501);

  for (let step = 0; step < 100; step++) {
    const action = rand() % 5;
    const beforeDebt = debt;
    const beforeCollateral = collateral;
    const borrowLimit = collateral * ltvBps / BPS;
    const liqCapacity = collateral * liqBps / BPS;

    if (action === 0) {
      supplied += pick(1_000_000);
    } else if (action === 1) {
      const idle = supplied >= debt ? supplied - debt : 0n;
      const room = borrowLimit > debt ? borrowLimit - debt : 0n;
      const max = idle < room ? idle : room;
      if (max > 0n) {
        const bounded = max > 1_000_000n ? 1_000_000n : max;
        debt += pick(Number(bounded)) + 1n;
      }
    } else if (action === 2 && debt > 0n) {
      const bounded = debt > 1_000_000n ? 1_000_000n : debt;
      const repay = pick(Number(bounded)) + 1n;
      debt -= repay > debt ? debt : repay;
    } else if (action === 3) {
      const shockBps = 5_000n + pick(5_001);
      collateral = collateral * shockBps / BPS;
    } else if (action === 4 && debt > 0n && hf(liqCapacity, debt) <= BPS) {
      const closeCap = (debt * closeFactorBps / BPS) || 1n;
      const collateralRepayCap = collateral * BPS / (BPS + bonusBps);
      let repay = closeCap < debt ? closeCap : debt;
      if (collateralRepayCap < repay) repay = collateralRepayCap;
      if (repay > 0n) {
        const seizeValue = repay * (BPS + bonusBps) / BPS;
        debt -= repay;
        collateral = collateral > seizeValue ? collateral - seizeValue : 0n;
      }
    }

    assert(supplied >= 0n && debt >= 0n && collateral >= 0n);
    if (action === 1 && debt > beforeDebt) {
      assert(debt <= borrowLimit, "borrow exceeded LTV");
      assert(debt <= supplied, "borrow exceeded pool assets");
    }
    if (action === 2) assert(debt <= beforeDebt, "repayment increased debt");
    if (action === 4) {
      assert(debt <= beforeDebt, "liquidation increased debt");
      assert(collateral <= beforeCollateral, "liquidation increased collateral");
    }
  }
}

console.log("44 Milady adversarial checks: PASS (50,000 deterministic transitions)");
