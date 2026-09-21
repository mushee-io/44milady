import fs from "node:fs";
import { BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint } from "@solana/spl-token";
import { accountExists, feedBytes, loadJson, loadProgram, pda, saveJson, units } from "./lib.mjs";

const configPath = process.env.MILADY_DEVNET_CONFIG || "config/devnet.local.json";
const walletFile = process.env.ANCHOR_WALLET;
if (!walletFile) throw new Error("Set ANCHOR_WALLET to a funded Devnet keypair");
if (!fs.existsSync(configPath)) {
  throw new Error("Missing " + configPath + ". Copy config/devnet.example.json and fill Pyth feed IDs/accounts.");
}
const config = loadJson(configPath);
const loaded = loadProgram({
  rpcUrl: process.env.SOLANA_RPC_URL || config.rpcUrl,
  walletFile,
});
const payer = loaded.payer;
const connection = loaded.connection;
const program = loaded.program;
const programId = program.programId;
const protocolConfig = pda(programId, "protocol");
const treasury = config.treasury ? new PublicKey(config.treasury) : payer.publicKey;
const emergencyAuthority = config.emergencyAuthority ? new PublicKey(config.emergencyAuthority) : payer.publicKey;

const deploymentPath = "deployments/devnet.local.json";
const deployment = fs.existsSync(deploymentPath)
  ? loadJson(deploymentPath)
  : { programId: programId.toBase58(), mints: {}, markets: {}, pythPriceAccounts: {} };

if (!await accountExists(connection, protocolConfig)) {
  await program.methods.initializeProtocol().accounts({
    authority: payer.publicKey,
    treasury,
    emergencyAuthority,
    protocolConfig,
    systemProgram: SystemProgram.programId,
  }).rpc();
}

for (const asset of config.assets) {
  if (!deployment.mints[asset.symbol]) {
    const mint = await createMint(
      connection,
      payer,
      protocolConfig,
      null,
      asset.decimals,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID,
    );
    deployment.mints[asset.symbol] = mint.toBase58();
    saveJson(deploymentPath, deployment);
  }

  const mint = new PublicKey(deployment.mints[asset.symbol]);
  const faucetConfig = pda(programId, "faucet", mint);
  if (!await accountExists(connection, faucetConfig)) {
    await program.methods.registerFaucetAsset(
      new BN(units(asset.faucetAmount, asset.decimals).toString()),
      config.faucetCooldownSeconds,
    ).accounts({
      authority: payer.publicKey,
      protocolConfig,
      mint,
      faucetConfig,
      systemProgram: SystemProgram.programId,
    }).rpc();
  }

  if (asset.symbol === "USDG") continue;
  if (!asset.feedId || String(asset.feedId).startsWith("REPLACE_")) {
    throw new Error("Missing Pyth feedId for " + asset.symbol);
  }
  const marketConfig = pda(programId, "market", mint);
  if (!await accountExists(connection, marketConfig)) {
    const symbol = [...Buffer.from(asset.symbol), ...new Array(8).fill(0)].slice(0, 8);
    await program.methods.registerMarket({
      symbol,
      feedId: feedBytes(asset.feedId),
      ltvBps: asset.ltvBps,
      liquidationThresholdBps: asset.liquidationThresholdBps,
      liquidationBonusBps: asset.liquidationBonusBps,
      maxConfidenceBps: asset.maxConfidenceBps,
      maxPriceAgeSecs: asset.maxPriceAgeSecs,
      supplyCap: new BN(units(asset.supplyCap, asset.decimals).toString()),
      debtCeilingUsdg: new BN(units(asset.debtCeilingUsdg || "0", 6).toString()),
    }).accounts({
      authority: payer.publicKey,
      protocolConfig,
      collateralMint: mint,
      marketConfig,
      systemProgram: SystemProgram.programId,
    }).rpc();
  }
  deployment.markets[asset.symbol] = marketConfig.toBase58();
  if (asset.priceAccount && !String(asset.priceAccount).startsWith("REPLACE_")) {
    deployment.pythPriceAccounts[asset.symbol] = asset.priceAccount;
  }
}

const usdgMint = new PublicKey(deployment.mints.USDG);
const lendingPool = pda(programId, "lending_pool", usdgMint);
const liquidityVault = pda(programId, "liquidity_vault", lendingPool);
if (!await accountExists(connection, lendingPool)) {
  await program.methods.initializeLendingPool({
    borrowCapUsdg: new BN(units(config.pool.borrowCapUsdg, 6).toString()),
    reserveFactorBps: config.pool.reserveFactorBps,
    baseRateBps: config.pool.baseRateBps,
    slope1Bps: config.pool.slope1Bps,
    slope2Bps: config.pool.slope2Bps,
    kinkUtilizationBps: config.pool.kinkUtilizationBps,
  }).accounts({
    authority: payer.publicKey,
    protocolConfig,
    usdgMint,
    lendingPool,
    liquidityVault,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  }).rpc();
}
await program.methods.updateLiquidationConfig(config.pool.liquidationCloseFactorBps, true).accounts({
  authority: payer.publicKey,
  protocolConfig,
  lendingPool,
}).rpc();

deployment.protocolConfig = protocolConfig.toBase58();
deployment.lendingPool = lendingPool.toBase58();
deployment.liquidityVault = liquidityVault.toBase58();
deployment.cluster = "devnet";
deployment.rpcUrl = process.env.SOLANA_RPC_URL || config.rpcUrl;
saveJson(deploymentPath, deployment);

console.log("44 Milady Devnet bootstrap complete");
console.log(deployment);
