import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { Wallet } from "@coral-xyz/anchor";
import { HermesClient } from "@pythnetwork/hermes-client";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import {
  Connection, Keypair, PublicKey, SystemProgram, Transaction,
  TransactionInstruction, sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

const RPC="https://api.devnet.solana.com";
const PROGRAM_ID=new PublicKey("BS3vTdhrkK5zHchx92PFGeodckt1dLzf7i9uJyEsmZst");
const SOL_FEED="0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
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
function symbol8(s){ const b=Buffer.alloc(8); Buffer.from(s).copy(b); return b; }
function feed32(hex){ return Buffer.from(hex.replace(/^0x/,""),"hex"); }
async function exists(pk){ return !!(await connection.getAccountInfo(pk,"confirmed")); }
async function tokenAmount(pk){
  try{return BigInt((await connection.getTokenAccountBalance(pk,"confirmed")).value.amount);}
  catch{return 0n;}
}
async function send(ix,label){
  const sig=await sendAndConfirmTransaction(connection,new Transaction().add(ix),[payer],{commitment:"confirmed"});
  console.log(label+": PASS",sig);
  return sig;
}

const protocol=new PublicKey(state.protocol);
const pool=new PublicKey(state.lendingPool);
const liquidityVault=new PublicKey(state.liquidityVault);
const usdg=new PublicKey(state.mints.usdg);
const credit=PublicKey.findProgramAddressSync(
  [Buffer.from("credit"),payer.publicKey.toBuffer()],PROGRAM_ID
)[0];

const creditInfo=await connection.getAccountInfo(credit,"confirmed");
if(!creditInfo) throw new Error("Credit account missing");
const currentDebt=creditInfo.data.readBigUInt64LE(40);
if(currentDebt>0n){
  console.log("44 MILADY SOLx BORROW TEST: ALREADY COMPLETE");
  console.log("Current debt:",Number(currentDebt)/1e6,"USDG");
  process.exit(0);
}

const apiKey=process.env.PYTH_API_KEY || process.env.HERMES_ACCESS_TOKEN || "";
if(!apiKey){
  console.error("PYTH_API_KEY is not set. NO SOLANA TRANSACTION SENT.");
  process.exit(2);
}

console.log("PRECHECK: fetching entitled Pyth SOL/USD feed before any Solana transaction...");
const hermes=new HermesClient("https://pyth.dourolabs.app/hermes",{accessToken:apiKey});
let response;
try{
  response=await hermes.getLatestPriceUpdates([SOL_FEED],{encoding:"base64"});
}catch(err){
  console.error("PYTH SOL/USD PRECHECK FAILED. NO SOLANA TRANSACTION SENT.");
  console.error(String(err));
  process.exit(2);
}
if(!response?.binary?.data?.length){
  console.error("Pyth returned no SOL/USD update. NO SOLANA TRANSACTION SENT.");
  process.exit(2);
}
const parsed=response.parsed?.find(x=>("0x"+x.id.replace(/^0x/,"")).toLowerCase()===SOL_FEED.toLowerCase())
  ?? response.parsed?.[0];
if(!parsed?.price) throw new Error("Pyth SOL/USD response missing parsed price");
const solPrice=Number(parsed.price.price)*10**Number(parsed.price.expo);
console.log("PYTH SOL/USD PRECHECK: PASS");
console.log("Fresh SOL/USD:",solPrice,"USD");

// Remove the old NVDAx demo collateral so the credit account has only an entitled oracle-backed asset.
if(state.mints?.nvdax && state.markets?.nvdax){
  const nvdax=new PublicKey(state.mints.nvdax);
  const nvdaxMarket=new PublicKey(state.markets.nvdax.market);
  const nvdaxAta=(await getOrCreateAssociatedTokenAccount(connection,payer,nvdax,payer.publicKey)).address;
  const nvdaxVault=PublicKey.findProgramAddressSync(
    [Buffer.from("vault"),credit.toBuffer(),nvdaxMarket.toBuffer()],PROGRAM_ID
  )[0];
  const nvdaAmount=await tokenAmount(nvdaxVault);
  if(nvdaAmount>0n){
    const ix=new TransactionInstruction({
      programId:PROGRAM_ID,
      keys:[
        {pubkey:payer.publicKey,isSigner:true,isWritable:true},
        {pubkey:protocol,isSigner:false,isWritable:false},
        {pubkey:credit,isSigner:false,isWritable:true},
        {pubkey:pool,isSigner:false,isWritable:true},
        {pubkey:nvdaxMarket,isSigner:false,isWritable:true},
        {pubkey:nvdax,isSigner:false,isWritable:false},
        {pubkey:nvdaxAta,isSigner:false,isWritable:true},
        {pubkey:nvdaxVault,isSigner:false,isWritable:true},
        {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      ],
      data:Buffer.concat([disc("withdraw_collateral"),u64(nvdaAmount)]),
    });
    await send(ix,"withdraw old NVDAx collateral");
  } else {
    console.log("old NVDAx collateral: already clear");
  }
}

// Create/reuse mock SOLx mint controlled by the protocol PDA.
const mintFile=path.resolve("target/devnet/solx-mint.json");
let solxKp;
if(fs.existsSync(mintFile)){
  solxKp=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(mintFile,"utf8"))));
}else{
  solxKp=Keypair.generate();
  fs.mkdirSync(path.dirname(mintFile),{recursive:true});
  fs.writeFileSync(mintFile,JSON.stringify(Array.from(solxKp.secretKey)));
}
const solx=solxKp.publicKey;
if(!(await exists(solx))){
  await createMint(connection,payer,protocol,null,6,solxKp,undefined,TOKEN_PROGRAM_ID);
  console.log("SOLx mint: CREATED",solx.toBase58());
}else{
  console.log("SOLx mint: EXISTS",solx.toBase58());
}

const [faucet]=PublicKey.findProgramAddressSync([Buffer.from("faucet"),solx.toBuffer()],PROGRAM_ID);
if(!(await exists(faucet))){
  await send(new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:solx,isSigner:false,isWritable:false},
      {pubkey:faucet,isSigner:false,isWritable:true},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("register_faucet_asset"),u64(100n*1_000_000n),u32(86400)]),
  }),"register SOLx faucet");
}else console.log("SOLx faucet: EXISTS");

const [solxMarket]=PublicKey.findProgramAddressSync([Buffer.from("market"),solx.toBuffer()],PROGRAM_ID);
if(!(await exists(solxMarket))){
  await send(new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:solx,isSigner:false,isWritable:false},
      {pubkey:solxMarket,isSigner:false,isWritable:true},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([
      disc("register_market"),
      symbol8("SOLx"),
      feed32(SOL_FEED),
      u16(7000),u16(8000),u16(500),u16(1000),u32(300),
      u64(1_000_000n*1_000_000n),u64(0),
    ]),
  }),"register SOLx market");
}else console.log("SOLx market: EXISTS",solxMarket.toBase58());

state.mints.solx=solx.toBase58();
state.markets.solx={
  symbol:"SOLx",market:solxMarket.toBase58(),mint:solx.toBase58(),
  pythSymbol:"Crypto.SOL/USD",feedId:SOL_FEED.replace(/^0x/,""),
  ltvBps:7000,liquidationThresholdBps:8000,
};
fs.writeFileSync(statePath,JSON.stringify(state,null,2));

const solxAta=(await getOrCreateAssociatedTokenAccount(connection,payer,solx,payer.publicKey)).address;
let walletSolx=await tokenAmount(solxAta);
if(walletSolx<10n*1_000_000n){
  const [claim]=PublicKey.findProgramAddressSync(
    [Buffer.from("claim"),payer.publicKey.toBuffer(),solx.toBuffer()],PROGRAM_ID
  );
  await send(new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:faucet,isSigner:false,isWritable:false},
      {pubkey:solx,isSigner:false,isWritable:true},
      {pubkey:solxAta,isSigner:false,isWritable:true},
      {pubkey:claim,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:disc("claim_faucet"),
  }),"claim 100 SOLx");
  walletSolx=await tokenAmount(solxAta);
}else console.log("SOLx faucet claim: SKIP");

const solxVault=PublicKey.findProgramAddressSync(
  [Buffer.from("vault"),credit.toBuffer(),solxMarket.toBuffer()],PROGRAM_ID
)[0];
let vaultSolx=await tokenAmount(solxVault);
if(vaultSolx<10n*1_000_000n){
  const deposit=10n*1_000_000n-vaultSolx;
  await send(new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:credit,isSigner:false,isWritable:true},
      {pubkey:solxMarket,isSigner:false,isWritable:true},
      {pubkey:solx,isSigner:false,isWritable:false},
      {pubkey:solxAta,isSigner:false,isWritable:true},
      {pubkey:solxVault,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("deposit_collateral"),u64(deposit)]),
  }),"deposit SOLx collateral");
  vaultSolx=await tokenAmount(solxVault);
}else console.log("SOLx collateral: ALREADY DEPOSITED");

const collateralUsd=(Number(vaultSolx)/1e6)*solPrice;
const borrowUsd=Math.max(1,Math.min(500,Math.floor(collateralUsd*0.20)));
const borrowRaw=BigInt(borrowUsd)*1_000_000n;
console.log("SOLx collateral value:",collateralUsd.toFixed(2),"USD");
console.log("Planned borrow:",borrowUsd,"USDG");

const usdgAta=(await getOrCreateAssociatedTokenAccount(connection,payer,usdg,payer.publicKey)).address;
const receiver=new PythSolanaReceiver({connection,wallet});
const builder=receiver.newTransactionBuilder({closeUpdateAccounts:true});
await builder.addPostPriceUpdates(response.binary.data);

await builder.addPriceConsumerInstructions(async getPriceUpdateAccount=>{
  const pythAccount=getPriceUpdateAccount(SOL_FEED);
  return [{
    instruction:new TransactionInstruction({
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
        {pubkey:solxMarket,isSigner:false,isWritable:false},
        {pubkey:pythAccount,isSigner:false,isWritable:false},
      ],
      data:Buffer.concat([disc("borrow_usdg"),u64(borrowRaw)]),
    }),
    signers:[],
  }];
});

console.log("Sending Pyth SOL/USD update + real borrow...");
const txs=await builder.buildVersionedTransactions({
  computeUnitPriceMicroLamports:1_000,
  tightComputeBudget:true,
});
const signatures=await receiver.provider.sendAll(
  txs,{preflightCommitment:"confirmed",commitment:"confirmed"}
);

const afterCredit=await connection.getAccountInfo(credit,"confirmed");
const debt=afterCredit?.data.readBigUInt64LE(40) ?? 0n;
const walletUsdg=await tokenAmount(usdgAta);
const poolUsdg=await tokenAmount(liquidityVault);
if(debt===0n) throw new Error("Borrow did not create debt");

const result={
  oracle:"Pyth Crypto.SOL/USD",
  solPrice,
  solxCollateral:Number(vaultSolx)/1e6,
  collateralUsd,
  debtUsdg:Number(debt)/1e6,
  walletUsdg:Number(walletUsdg)/1e6,
  poolUsdg:Number(poolUsdg)/1e6,
  signatures,
};
fs.writeFileSync(path.resolve("target/devnet/solx-borrow-test.json"),JSON.stringify(result,null,2));

console.log("\n44 MILADY SOLx PYTH BORROW TEST: PASS");
console.log(JSON.stringify(result,null,2));
console.log("SOL balance:",(await connection.getBalance(payer.publicKey))/1e9);
