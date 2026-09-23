import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import {
  Connection, Keypair, PublicKey, SystemProgram, Transaction,
  TransactionInstruction, sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID, createMint,
} from "@solana/spl-token";

const RPC="https://api.devnet.solana.com";
const PROGRAM_ID=new PublicKey("BS3vTdhrkK5zHchx92PFGeodckt1dLzf7i9uJyEsmZst");
const connection=new Connection(RPC,"confirmed");
const walletPath=path.join(os.homedir(),".config","solana","id.json");
const payer=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(walletPath,"utf8"))));
const outDir=path.resolve("target/devnet");
fs.mkdirSync(outDir,{recursive:true});

function disc(name){ return crypto.createHash("sha256").update("global:"+name).digest().subarray(0,8); }
function u16(n){ const b=Buffer.alloc(2); b.writeUInt16LE(n); return b; }
function u32(n){ const b=Buffer.alloc(4); b.writeUInt32LE(n); return b; }
function u64(n){ const b=Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; }
async function exists(pk){ return !!(await connection.getAccountInfo(pk,"confirmed")); }
async function send(ix,label,signers=[]){
  const sig=await sendAndConfirmTransaction(connection,new Transaction().add(ix),[payer,...signers],{commitment:"confirmed"});
  console.log(label+": PASS",sig);
  return sig;
}
function kpFile(name){ return path.join(outDir,name+"-mint.json"); }
function loadOrNewKp(name){
  const f=kpFile(name);
  if(fs.existsSync(f)) return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(f,"utf8"))));
  const kp=Keypair.generate();
  fs.writeFileSync(f,JSON.stringify(Array.from(kp.secretKey)));
  return kp;
}
async function ensureMint(name,authority,decimals=6){
  const kp=loadOrNewKp(name);
  if(!(await exists(kp.publicKey))){
    await createMint(connection,payer,authority,null,decimals,kp,undefined,TOKEN_PROGRAM_ID);
    console.log(name+" mint: CREATED",kp.publicKey.toBase58());
  } else console.log(name+" mint: EXISTS",kp.publicKey.toBase58());
  return kp.publicKey;
}

const program=await connection.getAccountInfo(PROGRAM_ID,"confirmed");
if(!program?.executable) throw new Error("44 Milady program is not executable on Devnet");

const [protocol]=PublicKey.findProgramAddressSync([Buffer.from("protocol")],PROGRAM_ID);
console.log("Wallet:",payer.publicKey.toBase58());
console.log("Program:",PROGRAM_ID.toBase58());
console.log("Protocol PDA:",protocol.toBase58());
console.log("Starting balance:",(await connection.getBalance(payer.publicKey))/1e9,"SOL");

if(!(await exists(protocol))){
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:payer.publicKey,isSigner:false,isWritable:false},
      {pubkey:payer.publicKey,isSigner:false,isWritable:false},
      {pubkey:protocol,isSigner:false,isWritable:true},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:disc("initialize_protocol"),
  });
  await send(ix,"initialize_protocol");
}else console.log("initialize_protocol: ALREADY INITIALIZED");

const assets={};
for(const name of ["usdg","nvdax","aaplx","spyx","tslax"]) assets[name]=await ensureMint(name,protocol,6);

async function ensureFaucet(name,mint,amount,cooldown=86400){
  const [faucet]=PublicKey.findProgramAddressSync([Buffer.from("faucet"),mint.toBuffer()],PROGRAM_ID);
  if(await exists(faucet)){ console.log("faucet "+name+": EXISTS",faucet.toBase58()); return faucet; }
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:mint,isSigner:false,isWritable:false},
      {pubkey:faucet,isSigner:false,isWritable:true},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("register_faucet_asset"),u64(amount),u32(cooldown)]),
  });
  await send(ix,"register_faucet_asset "+name);
  return faucet;
}

await ensureFaucet("USDG",assets.usdg,10_000n*1_000_000n);
await ensureFaucet("NVDAx",assets.nvdax,100n*1_000_000n);
await ensureFaucet("AAPLx",assets.aaplx,100n*1_000_000n);
await ensureFaucet("SPYx",assets.spyx,100n*1_000_000n);
await ensureFaucet("TSLAx",assets.tslax,100n*1_000_000n);

const [pool]=PublicKey.findProgramAddressSync([Buffer.from("lending_pool"),assets.usdg.toBuffer()],PROGRAM_ID);
const [vault]=PublicKey.findProgramAddressSync([Buffer.from("liquidity_vault"),pool.toBuffer()],PROGRAM_ID);
if(!(await exists(pool))){
  const args=Buffer.concat([
    u64(1_000_000n*1_000_000n), // 1m USDG Devnet borrow cap
    u16(1000), // 10% reserve factor
    u32(200),  // 2% base APR
    u32(800),  // +8% to kink
    u32(5000), // +50% above kink
    u16(8000), // 80% kink
  ]);
  const ix=new TransactionInstruction({
    programId:PROGRAM_ID,
    keys:[
      {pubkey:payer.publicKey,isSigner:true,isWritable:true},
      {pubkey:protocol,isSigner:false,isWritable:false},
      {pubkey:assets.usdg,isSigner:false,isWritable:false},
      {pubkey:pool,isSigner:false,isWritable:true},
      {pubkey:vault,isSigner:false,isWritable:true},
      {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
      {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
    ],
    data:Buffer.concat([disc("initialize_lending_pool"),args]),
  });
  await send(ix,"initialize_lending_pool");
}else console.log("initialize_lending_pool: ALREADY INITIALIZED");

const state={
  network:"devnet", programId:PROGRAM_ID.toBase58(), authority:payer.publicKey.toBase58(),
  protocol:protocol.toBase58(), lendingPool:pool.toBase58(), liquidityVault:vault.toBase58(),
  mints:Object.fromEntries(Object.entries(assets).map(([k,v])=>[k,v.toBase58()])),
};
fs.writeFileSync(path.join(outDir,"state.json"),JSON.stringify(state,null,2));
console.log("\n44 MILADY CORE INITIALIZATION: PASS");
console.log(JSON.stringify(state,null,2));
console.log("Ending balance:",(await connection.getBalance(payer.publicKey))/1e9,"SOL");
