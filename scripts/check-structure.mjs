import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const required = [
  "Anchor.toml",
  "Cargo.toml",
  "programs/forty_four_milady/Cargo.toml",
  "programs/forty_four_milady/src/lib.rs",
  "programs/forty_four_milady/src/parts/risk.rs",
  "app/package.json",
  "sdk/src/index.ts",
  "scripts/model-check.mjs",
  "docs/M2-M5.md",
];
for (const path of required) {
  if (!existsSync(path)) throw new Error(`Missing required file: ${path}`);
}

function rustSources(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return rustSources(path);
    return entry.name.endsWith(".rs") ? [readFileSync(path, "utf8")] : [];
  });
}
const source = rustSources("programs/forty_four_milady/src").join("\n");
for (const symbol of [
  "register_market",
  "register_faucet_asset",
  "claim_faucet",
  "initialize_credit_account",
  "deposit_collateral",
  "withdraw_collateral",
  "refresh_health",
  "PriceUpdateV2",
  "compute_portfolio_risk",
]) {
  if (!source.includes(symbol)) throw new Error(`Program missing ${symbol}`);
}
console.log("44 Milady structure check: PASS");
