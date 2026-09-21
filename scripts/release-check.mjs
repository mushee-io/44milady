import fs from "node:fs";
import path from "node:path";
import { PublicKey } from "@solana/web3.js";

const lib = fs.readFileSync("programs/forty_four_milady/src/lib.rs", "utf8");
const anchor = fs.readFileSync("Anchor.toml", "utf8");
const idMatch = lib.match(/declare_id!\("([^"]+)"\)/);
const anchorMatch = anchor.match(/forty_four_milady\s*=\s*"([^"]+)"/);

if (!idMatch || !anchorMatch) throw new Error("Program ID missing from lib.rs or Anchor.toml");
if (idMatch[1] !== anchorMatch[1]) {
  throw new Error("Program ID mismatch: lib.rs=" + idMatch[1] + " Anchor.toml=" + anchorMatch[1]);
}
new PublicKey(idMatch[1]);

const placeholder = "7ahY74GVSGRf9sDXFPtX6EnynoxWz2myNijQd7MPH5vF";
if (process.env.REQUIRE_NON_PLACEHOLDER_ID === "1" && idMatch[1] === placeholder) {
  throw new Error("Deployment blocked: generate the program keypair and run anchor keys sync first.");
}

const ignored = new Set([".git", "node_modules", "target", ".next", ".anchor"]);
function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (ignored.has(entry.name)) return [];
    const p = path.join(dir, entry.name);
    return entry.isDirectory() ? walk(p) : [p];
  });
}
for (const file of walk(".")) {
  if (/keypair\.json$/i.test(file)) throw new Error("Private key file must not be committed: " + file);
}

console.log("44 Milady release preflight: PASS");
console.log({ programId: idMatch[1], nonPlaceholder: idMatch[1] !== placeholder });
