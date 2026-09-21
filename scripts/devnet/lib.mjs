import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

export function expandHome(value) {
  return value && value.startsWith("~/") ? path.join(os.homedir(), value.slice(2)) : value;
}
export function loadJson(file) {
  return JSON.parse(fs.readFileSync(expandHome(file), "utf8"));
}
export function saveJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
export function loadKeypair(file) {
  return Keypair.fromSecretKey(Uint8Array.from(loadJson(file)));
}
export function units(value, decimals = 6) {
  const parts = String(value).split(".");
  const whole = parts[0];
  const fraction = parts[1] || "";
  const padded = (fraction + "0".repeat(decimals)).slice(0, decimals);
  return BigInt(whole) * (10n ** BigInt(decimals)) + BigInt(padded || "0");
}
export function feedBytes(feedId) {
  const clean = String(feedId).replace(/^0x/, "");
  if (!/^[0-9a-fA-F]{64}$/.test(clean)) {
    throw new Error("Invalid 32-byte Pyth feed id: " + feedId);
  }
  return [...Buffer.from(clean, "hex")];
}
export function pda(programId, ...seeds) {
  return PublicKey.findProgramAddressSync(
    seeds.map((seed) => typeof seed === "string" ? Buffer.from(seed) : seed.toBuffer()),
    programId,
  )[0];
}
export function loadProgram({ rpcUrl, walletFile, idlFile = "target/idl/forty_four_milady.json" }) {
  const payer = loadKeypair(walletFile);
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  const idl = loadJson(idlFile);
  const program = new Program(idl, provider);
  return { payer, connection, wallet, provider, program };
}
export async function accountExists(connection, pubkey) {
  return (await connection.getAccountInfo(pubkey, "confirmed")) !== null;
}
