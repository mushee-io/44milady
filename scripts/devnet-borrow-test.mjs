import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { Wallet } from "@coral-xyz/anchor";
import { HermesClient } from "@pythnetwork/hermes-client";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import {
  Connection, Keypair, PublicKey, Transaction, TransactionInstruction, sendAndConfirmTransaction,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

const RPC="https://api.devnet.solana.com";
const PROGRAM_ID=new PublicKey("BS3vTdhrkK5zHchx92PFGeodckt1dLzf7i9uJyEsmZst");
const connection=new Connection(RPC,"confirmed");
const payer=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(
  fs.readFileSync(path.join(os.homedir(),".config","solana","id.json"),"utf8")
)));
const wallet=new Wallet(payer);

const statePath=path.resolve("target/devnet/state.json");
if(!fs.existsSync(statePath)) throw new Error("Missing target/devnet/state.json");
const state=JSON.parse(fs.readFileSync(statePath,"utf8"));

function disc(name){ return crypto.createHash("sha256").update("global:"+name).digest().subarray(0,8); }
function u16(n){ const b=Buffer.alloc(2); b.writeUInt16LE(n); return b; }
function u32(n){ const b=Buffer.alloc(4); b.writeUInt32LE(n); return b; }
function u64(n){ const b=Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; }
function bool(v){ return Buffer.from([v?1:0]); }
function feed32(hex){ return Buffer.from(hex.replace(/^0x/,""),"hex"); }

const protocol=new PublicKey(state.protocol);
const pool=new PublicKey(state.lendingPool);
const liquidityVault=new PublicKey(state.liquidityVault);
const usdg=new PublicKey(state.mints.usdg);
const nvdaxMarket=new PublicKey(state.markets.nvdax.market);
let nvdaxFeed="0x"+state.markets.nvdax.feedId;
const usdgAta=new PublicKey("DVwL3qr226SQJTwTYVgrhWTKSoqj77XULbiwZcJg4td4");
const credit=new PublicKey("4rwvt4XXsrZEBMWLjRGuzQRnZ28Ts6rMVU9XVLsy33Yp");

const creditInfo=await connection.getAccountInfo(credit,"confirmed");
if(!creditInfo) throw new Error("Credit account missing");
const currentDebt=creditInfo.data.readBigUInt64LE(40);
if(currentDebt>0n){
  console.log("Borrow test already completed. Current debt:",Number(currentDebt)/1e6,"USDG");
  console.log("No transaction sent.");
  process.exit(0);
}

const apiKey=process.env.PYTH_API_KEY || process.env.HERMES_ACCESS_TOKEN || "";
if(!apiKey){
  console.error("PYTH_API_KEY is not set. No Solana transaction sent.");
  process.exit(2);
}

const feedRows=await fetch("https://hermes.pyth.network/v2/price_feeds?query=NVDA&asset_type=equity").then(r=>{
  if(!r.ok) throw new Error("Pyth feed discovery failed: "+r.status);
  return r.json();
});
const equity=feedRows.find(x=>x?.attributes?.symbol==="Equity.US.NVDA/USD")
  ?? feedRows.find(x=>String(x?.attributes?.symbol||"").includes("NVDA"));
if(!equity?.id) throw new Error("Pyth NVDA equity feed not found");
nvdaxFeed="0x"+equity.id;
console.log("Using Pyth equity feed:",equity.attributes?.symbol||"NVDA/USD",nvdaxFeed);

const hermes=new HermesClient(
  "https://pyth.dourolabs.app/hermes",
  {accessToken:apiKey},
);

console.log("Fetching fresh Pyth NVDA/USD update...");
let response;
try{
  response=await hermes.getLatestPriceUpdates([nvdaxFeed],{encoding:"base64"});
}catch(err){
  console.error("PYTH FETCH FAILED BEFORE ANY SOLANA TRANSACTION.");
  console.error(String(err));
  console.error("If this is an authorization error, set PYTH_API_KEY in this shell and rerun.");
  process.exit(2);
}
if(!response?.binary?.data?.length) throw new Error("Hermes returned no update data");

const storedFeed=("0x"+state.markets.nvdax.feedId).toLowerCase();
if(storedFeed!==nvdaxFeed.toLowerCase()){
  console.log("Updating NVDAx market oracle to the entitled NVDA equity feed...");
  const updateIx=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:false},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:nvdaxMarket,isSigner:false,isWritable:true},
    ],
    data:Buffer.concat([
      disc("update_market"),
      feed32(nvdaxFeed),
      u16(6000),
      u16(7500),
      u16(500),
      u16(1000),
      u32(300),
      u64(1_000_000n*1_000_000n),
      u64(0),
      bool(true),
    ]),
  });
  const updateSig=await sendAndConfirmTransaction(
    connection,
    new Transaction().add(updateIx),
    [payer],
    {commitment:"confirmed"}
  );
  console.log("update_market NVDAx: PASS",updateSig);
  state.markets.nvdax.feedId=nvdaxFeed.replace(/^0x/,"");
  state.markets.nvdax.pythSymbol=equity.attributes?.symbol||"Equity.US.NVDA/USD";
  fs.writeFileSync(statePath,JSON.stringify(state,null,2));
}

const parsed=response.parsed?.find(x=>("0x"+x.id.replace(/^0x/,""))===nvdaxFeed) ?? response.parsed?.[0];
if(!parsed?.price) throw new Error("Hermes response missing parsed NVDAx price");
const px=Number(parsed.price.price)*10**Number(parsed.price.expo);
const collateralUsd=10*px;
const borrowUsd=Math.max(1,Math.min(500,Math.floor(collateralUsd*0.20)));
const borrowRaw=BigInt(borrowUsd)*1_000_000n;

console.log("Fresh NVDA/USD price:",px,"USD");
console.log("10 NVDAx collateral value:",collateralUsd.toFixed(2),"USD");
console.log("Planned test borrow:",borrowUsd,"USDG (20% of collateral value, capped at 500)");

const receiver=new PythSolanaReceiver({connection,wallet});
const builder=receiver.newTransactionBuilder({closeUpdateAccounts:true});
await builder.addPostPriceUpdates(response.binary.data);

const priceUpdate=builder.getPriceUpdateAccount(nvdaxFeed);
console.log("Ephemeral Pyth price account:",priceUpdate.toBase58());

await builder.addPriceConsumerInstructions(async (getPriceUpdateAccount)=>{
  const pythAccount=getPriceUpdateAccount(nvdaxFeed);
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:credit,isSigner:false,isWritable:true},
      {pubkey:pool,isSigner:false,isWritable:true},
      {pubkey:usdg,isSigner:false,isWritable:false},
      {pubkey:usdgAta,isSigner:false,isWritable:true},
      {pubkey:liquidityVault,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      // remaining_accounts pair expected by 44 Milady risk engine
      {pubkey:nvdaxMarket,isSigner:false,isWritable:false},
      {pubkey:pythAccount,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("borrow_usdg"),u64(borrowRaw)]),
  });
  return [{instruction:ix,signers:[]}];
});

const txs=await builder.buildVersionedTransactions({
  computeUnitPriceMicroLamports:1_000,
  tightComputeBudget:true,
});

console.log("Sending Pyth update + borrow sequence...");
const signatures=await receiver.provider.sendAll(txs,{preflightCommitment:"confirmed",commitment:"confirmed"});
console.log("Transactions:",signatures);

const afterCredit=await connection.getAccountInfo(credit,"confirmed");
if(!afterCredit) throw new Error("Credit account disappeared");
const debt=afterCredit.data.readBigUInt64LE(40);
const walletUsdg=await connection.getTokenAccountBalance(usdgAta,"confirmed");
const poolUsdg=await connection.getTokenAccountBalance(liquidityVault,"confirmed");

if(debt===0n) throw new Error("Borrow transaction did not create debt");

const result={
  borrowUsd,
  debtUsdg:Number(debt)/1e6,
  walletUsdg:Number(walletUsdg.value.amount)/1e6,
  poolUsdg:Number(poolUsdg.value.amount)/1e6,
  signatures,
};
fs.writeFileSync(path.resolve("target/devnet/borrow-test.json"),JSON.stringify(result,null,2));

console.log("\n44 MILADY PYTH BORROW TEST: PASS");
console.log(JSON.stringify(result,null,2));
