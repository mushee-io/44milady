import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { AnchorProvider, BN, Program, Wallet } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  closeFactorCap,
  oracleFeedHex,
  rankCandidates,
} from "./policy.mjs";

const CREDIT_SEED = Buffer.from("credit");
const PROTOCOL_SEED = Buffer.from("protocol");
const LENDING_POOL_SEED = Buffer.from("lending_pool");
const VAULT_SEED = Buffer.from("vault");

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`Missing required environment variable ${name}`);
  return value;
}

function expandHome(value) {
  if (!value.startsWith("~/")) return value;
  return path.join(os.homedir(), value.slice(2));
}

function loadKeypair(file) {
  const bytes = JSON.parse(fs.readFileSync(expandHome(file), "utf8"));
  return Keypair.fromSecretKey(Uint8Array.from(bytes));
}

function loadJson(file) {
  return JSON.parse(fs.readFileSync(expandHome(file), "utf8"));
}

function bnToBigInt(value) {
  return BigInt(value.toString());
}

async function main() {
  const rpcUrl = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
  const idlPath = required("MILADY_IDL_PATH");
  const walletPath = required("LIQUIDATOR_KEYPAIR");
  const usdgMint = new PublicKey(required("USDG_MINT"));
  const oracleMap = loadJson(required("PYTH_PRICE_ACCOUNT_MAP"));
  const dryRun = (process.env.DRY_RUN ?? "true").toLowerCase() !== "false";
  const intervalMs = Number(process.env.KEEPER_INTERVAL_MS ?? "15000");

  const idl = loadJson(idlPath);
  const payer = loadKeypair(walletPath);
  const wallet = new Wallet(payer);
  const connection = new Connection(rpcUrl, "confirmed");
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  const program = new Program(idl, provider);
  const programId = program.programId;

  const [protocolConfig] = PublicKey.findProgramAddressSync(
    [PROTOCOL_SEED],
    programId,
  );
  const [lendingPool] = PublicKey.findProgramAddressSync(
    [LENDING_POOL_SEED, usdgMint.toBuffer()],
    programId,
  );

  const tokenProgram = process.env.TOKEN_PROGRAM_ID
    ? new PublicKey(process.env.TOKEN_PROGRAM_ID)
    : TOKEN_PROGRAM_ID;

  async function runOnce() {
    const pool = await program.account.lendingPool.fetch(lendingPool);
    if (!pool.liquidationEnabled) {
      console.log("44 Milady keeper: liquidations disabled");
      return;
    }

    const accounts = await program.account.creditAccount.all();
    const candidates = rankCandidates(
      accounts.map(({ publicKey, account }) => ({
        publicKey,
        owner: account.owner,
        healthFactorBps: bnToBigInt(account.lastHealthFactorBps),
        debtUsdg: bnToBigInt(account.debtUsdg),
        collaterals: account.collaterals,
      })),
    );

    if (candidates.length === 0) {
      console.log("44 Milady keeper: no cached liquidatable positions");
      return;
    }

    for (const candidate of candidates) {
      if (candidate.collaterals.length === 0) {
        console.log(
          `bad-debt candidate ${candidate.publicKey.toBase58()} has no collateral`,
        );
        continue;
      }

      const marketAccounts = [];
      for (const collateral of candidate.collaterals) {
        const marketPk = new PublicKey(collateral.market);
        const market = await program.account.marketConfig.fetch(marketPk);
        const feedHex = oracleFeedHex(market.feedId);
        const pricePkText = oracleMap[feedHex];
        if (!pricePkText) {
          throw new Error(
            `No fresh Pyth PriceUpdate account configured for feed ${feedHex}`,
          );
        }
        marketAccounts.push({
          publicKey: marketPk,
          market,
          priceUpdate: new PublicKey(pricePkText),
        });
      }

      // Start with the first collateral. The on-chain program rechecks the entire
      // portfolio and caps the actual repayment by collateral value and close factor.
      const selected = marketAccounts[0];
      const collateralMint = new PublicKey(selected.market.mint);
      const [collateralVault] = PublicKey.findProgramAddressSync(
        [VAULT_SEED, candidate.publicKey.toBuffer(), selected.publicKey.toBuffer()],
        programId,
      );

      const liquidatorUsdgAccount = getAssociatedTokenAddressSync(
        usdgMint,
        wallet.publicKey,
        false,
        tokenProgram,
      );
      const liquidatorCollateralAccount = getAssociatedTokenAddressSync(
        collateralMint,
        wallet.publicKey,
        false,
        tokenProgram,
      );

      const maxRepay = closeFactorCap(
        candidate.debtUsdg,
        Number(pool.liquidationCloseFactorBps),
      );

      const remainingAccounts = marketAccounts.flatMap((item) => [
        { pubkey: item.publicKey, isSigner: false, isWritable: false },
        { pubkey: item.priceUpdate, isSigner: false, isWritable: false },
      ]);

      const label =
        `${candidate.publicKey.toBase58()} health=${candidate.healthFactorBps} debt=${candidate.debtUsdg} maxRepay=${maxRepay}`;

      if (dryRun) {
        console.log(`DRY RUN liquidation candidate: ${label}`);
        continue;
      }

      const signature = await program.methods
        .liquidate(new BN(maxRepay.toString()))
        .accounts({
          liquidator: wallet.publicKey,
          borrower: candidate.owner,
          protocolConfig,
          creditAccount: candidate.publicKey,
          lendingPool,
          usdgMint,
          liquidatorUsdgAccount,
          liquidityVault: pool.liquidityVault,
          marketConfig: selected.publicKey,
          collateralMint,
          collateralVault,
          liquidatorCollateralAccount,
          priceUpdate: selected.priceUpdate,
          tokenProgram,
        })
        .remainingAccounts(remainingAccounts)
        .rpc();

      console.log(`liquidated ${label} tx=${signature}`);
    }
  }

  console.log(
    `44 Milady keeper started on ${rpcUrl}; dryRun=${dryRun}; interval=${intervalMs}ms`,
  );

  await runOnce();
  setInterval(() => {
    runOnce().catch((error) => console.error("keeper iteration failed", error));
  }, intervalMs);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
