import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const required = [
  "Anchor.toml",
  "Cargo.toml",
  "programs/forty_four_milady/Cargo.toml",
  "programs/forty_four_milady/src/lib.rs",
  "programs/forty_four_milady/src/parts/risk.rs",
  "programs/forty_four_milady/src/parts/accounts_lending.rs",
  "programs/forty_four_milady/src/parts/handlers_lending.rs",
  "app/package.json",
  "sdk/src/index.ts",
  "scripts/model-check.mjs",
  "docs/M2-M5.md",
  "docs/M6.md",
  "docs/M7.md",
  "docs/M8.md",
  "programs/forty_four_milady/src/parts/interest.rs",
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
  "initialize_lending_pool",
  "supply_usdg",
  "withdraw_supplied_usdg",
  "borrow_usdg",
  "LendingPool",
  "SupplierPosition",
  "accrue_interest",
  "sync_borrower_interest",
  "sync_supplier_interest",
  "borrow_apr_bps",
  "supply_apr_bps",
  "protocol_reserves_usdg",
  "repay_usdg",
  "repay_usdg_max",
  "repay_usdg_on_behalf",
  "repay_usdg_on_behalf_max",
  "close_credit_account",
  "close_supplier_position",
  "close_collateral_vault",
  "UsdgRepaid",
]) {
  if (!source.includes(symbol)) throw new Error(`Program missing ${symbol}`);
}
console.log("44 Milady structure check: PASS");
