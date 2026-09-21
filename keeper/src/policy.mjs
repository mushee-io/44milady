export const BPS = 10_000n;

export function isLiquidatable(healthFactorBps, debtUsdg) {
  return debtUsdg > 0n && healthFactorBps <= BPS;
}

export function closeFactorCap(debtUsdg, closeFactorBps) {
  if (debtUsdg <= 0n) return 0n;
  if (!Number.isInteger(closeFactorBps) || closeFactorBps <= 0 || closeFactorBps > 10_000) {
    throw new Error("invalid close factor");
  }
  const cap = (debtUsdg * BigInt(closeFactorBps)) / BPS;
  return cap === 0n ? 1n : cap;
}

export function rankCandidates(positions) {
  return positions
    .filter((p) => isLiquidatable(p.healthFactorBps, p.debtUsdg))
    .sort((a, b) => {
      if (a.healthFactorBps !== b.healthFactorBps) {
        return a.healthFactorBps < b.healthFactorBps ? -1 : 1;
      }
      if (a.debtUsdg !== b.debtUsdg) {
        return a.debtUsdg > b.debtUsdg ? -1 : 1;
      }
      return String(a.publicKey).localeCompare(String(b.publicKey));
    });
}

export function oracleFeedHex(feedId) {
  return Buffer.from(feedId).toString("hex");
}
