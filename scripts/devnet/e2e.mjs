import fs from "node:fs";
import { BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  getAccount,
} from "@solana/spl-token";
import { accountExists, loadJson, loadProgram, pda, units } from "./lib.mjs";

const deploymentPath = process.env.MILADY_DEPLOYMENT || "deployments/devnet.local.json";
const walletFile = process.env.ANCHOR_WALLET;
if (!walletFile || !fs.existsSync(deploymentPath)) {
  throw new Error("Set ANCHOR_WALLET and bootstrap Devnet first");
}
const deployment = loadJson(deploymentPath);
const config = loadJson(process.env.MILADY_DEVNET_CONFIG || "config/devnet.local.json");
const loaded = loadProgram({
  rpcUrl: process.env.SOLANA_RPC_URL || deployment.rpcUrl,
  walletFile,
});
const payer = loaded.payer;
const connection = loaded.connection;
const program = loaded.program;
const protocolConfig = new PublicKey(deployment.protocolConfig);
const usdgMint = new PublicKey(deployment.mints.USDG);
const lendingPool = new PublicKey(deployment.lendingPool);
const liquidityVault = new PublicKey(deployment.liquidityVault);

async function claim(symbol) {
  const asset = config.assets.find((x) => x.symbol === symbol);
  const mint = new PublicKey(deployment.mints[symbol]);
  const ata = await createAssociatedTokenAccountIdempotent(connection, payer, mint, payer.publicKey);
  const faucetConfig = pda(program.programId, "faucet", mint);
  const faucetClaim = pda(program.programId, "claim", payer.publicKey, mint);
  await program.methods.claimFaucet().accounts({
    claimant: payer.publicKey,
    protocolConfig,
    faucetConfig,
    mint,
    destination: ata,
    faucetClaim,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  }).rpc();
  return { asset, mint, ata };
}

const usdg = await claim("USDG");
const supplierPosition = pda(program.programId, "supplier", lendingPool, payer.publicKey);
const supplyAmount = units("1000", 6);
await program.methods.supplyUsdg(new BN(supplyAmount.toString())).accounts({
  supplier: payer.publicKey,
  protocolConfig,
  lendingPool,
  usdgMint,
  supplierUsdgAccount: usdg.ata,
  liquidityVault,
  supplierPosition,
  tokenProgram: TOKEN_PROGRAM_ID,
  systemProgram: SystemProgram.programId,
}).rpc();

const collateralSymbol = process.env.E2E_COLLATERAL || "SPYx";
const collateral = await claim(collateralSymbol);
const marketConfig = new PublicKey(deployment.markets[collateralSymbol]);
const creditAccount = pda(program.programId, "credit", payer.publicKey);
if (!await accountExists(connection, creditAccount)) {
  await program.methods.initializeCreditAccount().accounts({
    owner: payer.publicKey,
    protocolConfig,
    creditAccount,
    systemProgram: SystemProgram.programId,
  }).rpc();
}
const collateralVault = pda(program.programId, "vault", creditAccount, marketConfig);
const deposit = units("1", collateral.asset.decimals);
await program.methods.depositCollateral(new BN(deposit.toString())).accounts({
  owner: payer.publicKey,
  protocolConfig,
  creditAccount,
  marketConfig,
  collateralMint: collateral.mint,
  ownerTokenAccount: collateral.ata,
  collateralVault,
  tokenProgram: TOKEN_PROGRAM_ID,
  systemProgram: SystemProgram.programId,
}).rpc();

const priceAddress = deployment.pythPriceAccounts && deployment.pythPriceAccounts[collateralSymbol];
if (!priceAddress) throw new Error("Missing Pyth priceAccount for " + collateralSymbol);
const riskAccounts = [
  { pubkey: marketConfig, isSigner: false, isWritable: false },
  { pubkey: new PublicKey(priceAddress), isSigner: false, isWritable: false },
];

await program.methods.refreshHealth().accounts({
  caller: payer.publicKey,
  protocolConfig,
  creditAccount,
  lendingPool,
}).remainingAccounts(riskAccounts).rpc();

await program.methods.updateLendingPool({
  borrowCapUsdg: new BN(units(config.pool.borrowCapUsdg, 6).toString()),
  reserveFactorBps: config.pool.reserveFactorBps,
  baseRateBps: config.pool.baseRateBps,
  slope1Bps: config.pool.slope1Bps,
  slope2Bps: config.pool.slope2Bps,
  kinkUtilizationBps: config.pool.kinkUtilizationBps,
  borrowEnabled: true,
}).accounts({
  authority: payer.publicKey,
  protocolConfig,
  lendingPool,
}).rpc();

const position = await program.account.creditAccount.fetch(creditAccount);
const room = BigInt(position.lastBorrowLimitUsdMicro.toString());
if (room <= 0n) throw new Error("No borrow capacity after collateral valuation");
const borrow = room > units("10", 6) ? units("10", 6) : room / 2n;

await program.methods.borrowUsdg(new BN(borrow.toString())).accounts({
  borrower: payer.publicKey,
  protocolConfig,
  creditAccount,
  lendingPool,
  usdgMint,
  borrowerUsdgAccount: usdg.ata,
  liquidityVault,
  tokenProgram: TOKEN_PROGRAM_ID,
}).remainingAccounts(riskAccounts).rpc();

await program.methods.repayUsdgMax().accounts({
  payer: payer.publicKey,
  protocolConfig,
  creditAccount,
  lendingPool,
  usdgMint,
  payerUsdgAccount: usdg.ata,
  liquidityVault,
  tokenProgram: TOKEN_PROGRAM_ID,
}).rpc();

const finalPosition = await program.account.creditAccount.fetch(creditAccount);
const vaultAccount = await getAccount(connection, liquidityVault);
if (BigInt(finalPosition.debtUsdg.toString()) !== 0n) {
  throw new Error("E2E debt did not return to zero");
}

console.log("44 Milady Devnet E2E: PASS");
console.log({
  collateral: collateralSymbol,
  suppliedUsdg: supplyAmount.toString(),
  borrowedAndRepaidUsdg: borrow.toString(),
  vaultRawUsdg: vaultAccount.amount.toString(),
  finalDebtUsdg: finalPosition.debtUsdg.toString(),
});
