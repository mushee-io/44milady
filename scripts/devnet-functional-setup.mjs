import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import {
  Connection, Keypair, PublicKey, SystemProgram, Transaction,
  TransactionInstruction, sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

const RPC="https://api.devnet.solana.com";
const PROGRAM_ID=new PublicKey("BS3vTdhrkK5zHchx92PFGeodckt1dLzf7i9uJyEsmZst");
const connection=new Connection(RPC,"confirmed");
const payer=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(
  fs.readFileSync(path.join(os.homedir(),".config","solana","id.json"),"utf8")
)));
const state=JSON.parse(fs.readFileSync(path.resolve("target/devnet/state.json"),"utf8"));

function disc(name){ return crypto.createHash("sha256").update("global:"+name).digest().subarray(0,8); }
function u16(n){ const b=Buffer.alloc(2); b.writeUInt16LE(n); return b; }
function u32(n){ const b=Buffer.alloc(4); b.writeUInt32LE(n); return b; }
function u64(n){ const b=Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; }
function bool(n){ return Buffer.from([n?1:0]); }
async function exists(pk){ return !!(await connection.getAccountInfo(pk,"confirmed")); }
async function send(ix,label){
  const sig=await sendAndConfirmTransaction(connection,new Transaction().add(ix),[payer],{commitment:"confirmed"});
  console.log(label+": PASS",sig);
  return sig;
}
async function tokenAmount(ata){
  try { return BigInt((await connection.getTokenAccountBalance(ata,"confirmed")).value.amount); }
  catch { return 0n; }
}

const protocol=new PublicKey(state.protocol);
const pool=new PublicKey(state.lendingPool);
const liquidityVault=new PublicKey(state.liquidityVault);
const usdg=new PublicKey(state.mints.usdg);
const nvdax=new PublicKey(state.mints.nvdax);
const nvdaxMarket=new PublicKey(state.markets.nvdax.market);

const usdgAta=(await getOrCreateAssociatedTokenAccount(connection,payer,usdg,payer.publicKey)).address;
const nvdaxAta=(await getOrCreateAssociatedTokenAccount(connection,payer,nvdax,payer.publicKey)).address;

async function claimIfNeeded(label,mint,ata,minRaw){
  const current=await tokenAmount(ata);
  if(current>=minRaw){
    console.log(`claim ${label}: SKIP (wallet already has ${current} raw units)`);
    return;
  }
  const [faucet]=PublicKey.findProgramAddressSync([Buffer.from("faucet"),mint.toBuffer()],PROGRAM_ID);
  const [claim]=PublicKey.findProgramAddressSync(
    [Buffer.from("claim"),payer.publicKey.toBuffer(),mint.toBuffer()],PROGRAM_ID
  );
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:faucet,isSigner:false,isWritable:false},
      {pubkey:mint,isSigner:false,isWritable:true},
      {pubkey:ata,isSigner:false,isWritable:true},
      {pubkey:claim,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:disc("claim_faucet"),
  });
  await send(ix,"claim "+label);
}

await claimIfNeeded("USDG",usdg,usdgAta,5_000n*1_000_000n);
await claimIfNeeded("NVDAx",nvdax,nvdaxAta,10n*1_000_000n);

const [credit]=PublicKey.findProgramAddressSync([Buffer.from("credit"),payer.publicKey.toBuffer()],PROGRAM_ID);
if(!(await exists(credit))){
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:credit,isSigner:false,isWritable:true},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:disc("initialize_credit_account"),
  });
  await send(ix,"initialize_credit_account");
}else console.log("initialize_credit_account: ALREADY EXISTS");

const [supplier]=PublicKey.findProgramAddressSync(
  [Buffer.from("supplier"),pool.toBuffer(),payer.publicKey.toBuffer()],PROGRAM_ID
);
if(!(await exists(supplier))){
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:pool,isSigner:false,isWritable:true},
      {pubkey:usdg,isSigner:false,isWritable:false},
      {pubkey:usdgAta,isSigner:false,isWritable:true},
      {pubkey:liquidityVault,isSigner:false,isWritable:true},
      {pubkey:supplier,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("supply_usdg"),u64(5_000n*1_000_000n)]),
  });
  await send(ix,"supply 5000 USDG");
}else console.log("supply 5000 USDG: SKIP (supplier position already exists)");

const [collateralVault]=PublicKey.findProgramAddressSync(
  [Buffer.from("vault"),credit.toBuffer(),nvdaxMarket.toBuffer()],PROGRAM_ID
);
let vaultAmt=await tokenAmount(collateralVault);
if(vaultAmt===0n){
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:credit,isSigner:false,isWritable:true},
      {pubkey:nvdaxMarket,isSigner:false,isWritable:true},
      {pubkey:nvdax,isSigner:false,isWritable:false},
      {pubkey:nvdaxAta,isSigner:false,isWritable:true},
      {pubkey:collateralVault,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("deposit_collateral"),u64(10n*1_000_000n)]),
  });
  await send(ix,"deposit 10 NVDAx");
  vaultAmt=await tokenAmount(collateralVault);
}else console.log("deposit 10 NVDAx: SKIP (collateral vault already funded)");

const updateArgs=Buffer.concat([
  u64(1_000_000n*1_000_000n),
  u16(1000), u32(200), u32(800), u32(5000), u16(8000), bool(true),
]);
await send(new TransactionInstruction({
  programId:PROGRAM_ID,
  keys:[
    {pubkey:payer.publicKey,isSigner:true,isWritable:false},
    {pubkey:protocol,isSigner:false,isWritable:false},
    {pubkey:pool,isSigner:false,isWritable:true},
  ],
  data:Buffer.concat([disc("update_lending_pool"),updateArgs]),
}),"enable borrowing");

console.log("\n44 MILADY FUNCTIONAL SETUP: PASS");
console.log("USDG ATA:",usdgAta.toBase58());
console.log("NVDAx ATA:",nvdaxAta.toBase58());
console.log("Credit PDA:",credit.toBase58());
console.log("Supplier PDA:",supplier.toBase58());
console.log("NVDAx collateral vault:",collateralVault.toBase58());
console.log("Wallet USDG:",Number(await tokenAmount(usdgAta))/1e6);
console.log("Pool USDG:",Number(await tokenAmount(liquidityVault))/1e6);
console.log("Wallet NVDAx:",Number(await tokenAmount(nvdaxAta))/1e6);
console.log("Vault NVDAx:",Number(await tokenAmount(collateralVault))/1e6);
console.log("SOL balance:",(await connection.getBalance(payer.publicKey))/1e9);
