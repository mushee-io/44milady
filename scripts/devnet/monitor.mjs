import fs from "node:fs";
import { PublicKey } from "@solana/web3.js";
import { loadJson, loadProgram } from "./lib.mjs";

const deploymentPath = process.env.MILADY_DEPLOYMENT || "deployments/devnet.local.json";
const walletFile = process.env.ANCHOR_WALLET;
if (!walletFile || !fs.existsSync(deploymentPath)) {
  throw new Error("Set ANCHOR_WALLET and create deployments/devnet.local.json");
}
const deployment = loadJson(deploymentPath);
const loaded = loadProgram({
  rpcUrl: process.env.SOLANA_RPC_URL || deployment.rpcUrl,
  walletFile,
});
const connection = loaded.connection;
const program = loaded.program;

async function sample() {
  const protocol = await program.account.protocolConfig.fetch(new PublicKey(deployment.protocolConfig));
  const pool = await program.account.lendingPool.fetch(new PublicKey(deployment.lendingPool));
  const vault = await connection.getTokenAccountBalance(new PublicKey(deployment.liquidityVault), "confirmed");
  const badDebt = BigInt(pool.badDebtUsdg.toString());
  const supplied = BigInt(pool.totalSuppliedUsdg.toString());
  const borrowed = BigInt(pool.totalBorrowedUsdg.toString());
  const utilization = supplied === 0n ? 0 : Number(borrowed * 10_000n / supplied) / 100;

  const status = {
    at: new Date().toISOString(),
    paused: protocol.paused,
    borrowEnabled: pool.borrowEnabled,
    liquidationEnabled: pool.liquidationEnabled,
    vaultUsdg: vault.value.uiAmountString,
    totalSuppliedUsdg: supplied.toString(),
    totalBorrowedUsdg: borrowed.toString(),
    protocolReservesUsdg: pool.protocolReservesUsdg.toString(),
    insuranceReserveUsdg: pool.insuranceReserveUsdg.toString(),
    badDebtUsdg: badDebt.toString(),
    utilizationPct: utilization,
    borrowAprPct: Number(pool.lastBorrowAprBps) / 100,
    supplyAprPct: Number(pool.lastSupplyAprBps) / 100,
  };
  console.log(JSON.stringify(status));
  if (badDebt > 0n) console.error("ALERT: 44 Milady has uncovered bad debt");
  if (protocol.paused) console.error("ALERT: 44 Milady protocol is paused");
}

await sample();
setInterval(() => sample().catch(console.error), Number(process.env.MONITOR_INTERVAL_MS || 30000));
