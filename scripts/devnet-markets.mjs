import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import {
  Connection, Keypair, PublicKey, SystemProgram, Transaction,
  TransactionInstruction, sendAndConfirmTransaction,
} from "@solana/web3.js";

const RPC="https://api.devnet.solana.com";
const PROGRAM_ID=new PublicKey("BS3vTdhrkK5zHchx92PFGeodckt1dLzf7i9uJyEsmZst");
const connection=new Connection(RPC,"confirmed");
const payer=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(
  fs.readFileSync(path.join(os.homedir(),".config","solana","id.json"),"utf8")
)));
const statePath=path.resolve("target/devnet/state.json");
if(!fs.existsSync(statePath)) throw new Error("Missing target/devnet/state.json. Run devnet-init first.");
const state=JSON.parse(fs.readFileSync(statePath,"utf8"));

function disc(name){ return crypto.createHash("sha256").update("global:"+name).digest().subarray(0,8); }
function u16(n){ const b=Buffer.alloc(2); b.writeUInt16LE(n); return b; }
function u32(n){ const b=Buffer.alloc(4); b.writeUInt32LE(n); return b; }
function u64(n){ const b=Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; }
function symbol8(s){ const b=Buffer.alloc(8); Buffer.from(s).copy(b); return b; }
function feed32(hex){ return Buffer.from(hex.replace(/^0x/,""),"hex"); }
async function exists(pk){ return !!(await connection.getAccountInfo(pk,"confirmed")); }

async function resolveFeed(query, exactSymbol){
  const urls=[
    `https://hermes.pyth.network/v2/price_feeds?query=${encodeURIComponent(query)}`,
    `https://pyth.dourolabs.app/hermes/v2/price_feeds?query=${encodeURIComponent(query)}`,
  ];
  let last="";
  for(const url of urls){
    try{
      const r=await fetch(url,{headers:{accept:"application/json"}});
      last=`${r.status} ${r.statusText}`;
      if(!r.ok) continue;
      const rows=await r.json();
      const hit=rows.find(x=>x?.attributes?.symbol===exactSymbol)
        ?? rows.find(x=>x?.attributes?.display_symbol===query)
        ?? rows[0];
      if(hit?.id && /^[0-9a-fA-F]{64}$/.test(hit.id)) return hit.id;
    }catch(e){ last=String(e); }
  }
  throw new Error(`Could not resolve ${exactSymbol} from Pyth Hermes (${last})`);
}

async function send(ix,label){
  const sig=await sendAndConfirmTransaction(connection,new Transaction().add(ix),[payer],{commitment:"confirmed"});
  console.log(label+": PASS",sig);
}

const protocol=new PublicKey(state.protocol);
const configs=[
  {key:"nvdax", symbol:"NVDAx", query:"NVDAX", pyth:"Crypto.NVDAX/USD", ltv:6000, liq:7500},
  {key:"aaplx", symbol:"AAPLx", query:"AAPLX", pyth:"Crypto.AAPLX/USD", ltv:6000, liq:7500},
  {key:"spyx",  symbol:"SPYx",  query:"SPYX",  pyth:"Crypto.SPYX/USD",  ltv:7000, liq:8000},
  {key:"tslax", symbol:"TSLAx", query:"TSLAX", pyth:"Crypto.TSLAX/USD", ltv:5000, liq:6500},
];

state.markets ??= {};
for(const cfg of configs){
  const mint=new PublicKey(state.mints[cfg.key]);
  const [market]=PublicKey.findProgramAddressSync([Buffer.from("market"),mint.toBuffer()],PROGRAM_ID);
  const feedId=await resolveFeed(cfg.query,cfg.pyth);
  console.log(`${cfg.symbol} Pyth feed: ${feedId}`);

  if(!(await exists(market))){
    const data=Buffer.concat([
      disc("register_market"),
      symbol8(cfg.symbol),
      feed32(feedId),
      u16(cfg.ltv),
      u16(cfg.liq),
      u16(500),      // 5% liquidation bonus
      u16(1000),     // max 10% oracle confidence width on Devnet
      u32(300),      // max 5 minute price age
      u64(1_000_000n*1_000_000n), // 1m mock tokens supply cap
      u64(0),        // per-market debt ceiling currently unused
    ]);
    const ix=new TransactionInstruction({
      programId:PROGRAM_ID,
      keys:[
        {pubkey:payer.publicKey,isSigner:true,isWritable:true},
        {pubkey:protocol,isSigner:false,isWritable:false},
        {pubkey:mint,isSigner:false,isWritable:false},
        {pubkey:market,isSigner:false,isWritable:true},
        {pubkey:SystemProgram.programId,isSigner:false,isWritable:false},
      ],
      data,
    });
    await send(ix,"register_market "+cfg.symbol);
  } else {
    console.log("register_market "+cfg.symbol+": ALREADY EXISTS");
  }

  state.markets[cfg.key]={
    symbol:cfg.symbol,
    market:market.toBase58(),
    mint:mint.toBase58(),
    pythSymbol:cfg.pyth,
    feedId,
    ltvBps:cfg.ltv,
    liquidationThresholdBps:cfg.liq,
  };
}
fs.writeFileSync(statePath,JSON.stringify(state,null,2));
console.log("\n44 MILADY MARKET REGISTRY: PASS");
console.log(JSON.stringify(state.markets,null,2));
console.log("Ending balance:",(await connection.getBalance(payer.publicKey))/1e9,"SOL");
