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
  "programs/forty_four_milady/src/parts/accounts_liquidation.rs",
  "programs/forty_four_milady/src/parts/handlers_liquidation.rs",
  "programs/forty_four_milady/src/parts/interest.rs",
  "app/package.json",
  "app/src/components/ProtocolDashboard.tsx",
  "sdk/src/index.ts",
  "keeper/package.json",
  "keeper/src/index.mjs",
  "keeper/src/policy.mjs",
  "scripts/model-check.mjs",
  "scripts/adversarial-check.mjs",
  "scripts/security-check.mjs",
  "scripts/release-check.mjs",
  "scripts/devnet/prepare-program.sh",
  "scripts/devnet/deploy.sh",
  "scripts/devnet/bootstrap.mjs",
  "scripts/devnet/verify.mjs",
  "scripts/devnet/e2e.mjs",
  "scripts/devnet/monitor.mjs",
  "config/devnet.example.json",
  ".github/workflows/devnet-deploy.yml",
  "SECURITY.md",
  "docs/THREAT_MODEL.md",
  "docs/RELEASE_CHECKLIST.md",
  "docs/M2-M5.md",
  "docs/M6.md",
  "docs/M7.md",
  "docs/M8.md",
  "docs/M9.md",
  "docs/M10.md",
];
for (const path of required) {
  if (!existsSync(path)) throw new Error("Missing required file: " + path);
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
  "claim_faucet",
  "deposit_collateral",
  "withdraw_collateral",
  "compute_portfolio_risk",
  "borrow_usdg",
  "accrue_interest",
  "repay_usdg_max",
  "liquidate",
  "fund_insurance_reserve",
  "absorb_bad_debt",
  "recapitalize_bad_debt",
  "pending_authority",
  "pending_emergency_authority",
  "propose_authority",
  "accept_authority",
  "cancel_authority_transfer",
  "propose_emergency_authority",
  "accept_emergency_authority",
  "set_treasury",
  "bad_debt_usdg",
]) {
  if (!source.includes(symbol)) throw new Error("Program missing " + symbol);
}
console.log("44 Milady M1-M10 structure check: PASS");
