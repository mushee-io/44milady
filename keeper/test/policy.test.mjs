import test from "node:test";
import assert from "node:assert/strict";
import {
  closeFactorCap,
  isLiquidatable,
  rankCandidates,
} from "../src/policy.mjs";

test("liquidation threshold is health <= 1.0", () => {
  assert.equal(isLiquidatable(10_000n, 1n), true);
  assert.equal(isLiquidatable(9_999n, 1n), true);
  assert.equal(isLiquidatable(10_001n, 1n), false);
  assert.equal(isLiquidatable(9_000n, 0n), false);
});

test("50% close factor caps each liquidation", () => {
  assert.equal(closeFactorCap(4_000_000_000n, 5_000), 2_000_000_000n);
});

test("keeper prioritizes worst health then largest debt", () => {
  const ranked = rankCandidates([
    { publicKey: "a", healthFactorBps: 9_500n, debtUsdg: 1_000n },
    { publicKey: "b", healthFactorBps: 8_000n, debtUsdg: 500n },
    { publicKey: "c", healthFactorBps: 8_000n, debtUsdg: 2_000n },
    { publicKey: "safe", healthFactorBps: 12_000n, debtUsdg: 9_000n },
  ]);
  assert.deepEqual(ranked.map((x) => x.publicKey), ["c", "b", "a"]);
});
