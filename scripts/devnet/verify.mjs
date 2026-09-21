import fs from "node:fs";
import { PublicKey } from "@solana/web3.js";
import { loadJson, loadProgram } from "./lib.mjs";

const deploymentPath = process.env.MILADY_DEPLOYMENT || "deployments/devnet.local.json";
const walletFile = process.env.ANCHOR_WALLET;
if (!walletFile) throw new Error("Set ANCHOR_WALLET");
if (!fs.existsSync(deploymentPath)) throw new Error("Missing " + deploymentPath);

const deployment = loadJson(deploymentPath);
const loaded = loadProgram({
  rpcUrl: process.env.SOLANA_RPC_URL || deployment.rpcUrl,
  walletFile,
});
const connection = loaded.connection;
const program = loaded.program;

if (program.programId.toBase58() !== deployment.programId) {
  throw new Error("IDL program id does not match deployment manifest");
}
const executable = await connection.getAccountInfo(program.programId, "confirmed");
if (!executable || !executable.executable) throw new Error("Program is not executable on Devnet");

const protocol = await program.account.protocolConfig.fetch(new PublicKey(deployment.protocolConfig));
const pool = await program.account.lendingPool.fetch(new PublicKey(deployment.lendingPool));
if (Number(protocol.version) < 7) throw new Error("Protocol version is below M10");
if (pool.usdgMint.toBase58() !== deployment.mints.USDG) throw new Error("USDG pool mint mismatch");

for (const [symbol, address] of Object.entries(deployment.mints)) {
  if (!await connection.getAccountInfo(new PublicKey(address), "confirmed")) {
    throw new Error("Missing mint " + symbol + ": " + address);
  }
}
for (const [symbol, address] of Object.entries(deployment.markets)) {
  const market = await program.account.marketConfig.fetch(new PublicKey(address));
  if (!market.enabled) throw new Error("Market disabled unexpectedly: " + symbol);
}

console.log("44 Milady public Devnet verification: PASS");
console.log({
  programId: program.programId.toBase58(),
  protocolVersion: Number(protocol.version),
  lendingPool: deployment.lendingPool,
  usdgMint: deployment.mints.USDG,
  markets: Object.keys(deployment.markets),
});
