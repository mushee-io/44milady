import fs from "node:fs";
import path from "node:path";

function rustFiles(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) return rustFiles(p);
    return entry.name.endsWith(".rs") ? [p] : [];
  });
}

const files = rustFiles("programs/forty_four_milady/src");
for (const file of files) {
  if (file.endsWith("tests.rs")) continue;
  const source = fs.readFileSync(file, "utf8");
  for (const forbidden of ["unsafe {", ".unwrap()", ".expect(", "panic!("]) {
    if (source.includes(forbidden)) {
      throw new Error("Forbidden release pattern " + JSON.stringify(forbidden) + " in " + file);
    }
  }
}

const combined = files.map((file) => fs.readFileSync(file, "utf8")).join("\n");
for (const required of [
  "InvalidOracleOwner",
  "OracleConfidenceTooWide",
  "ProtocolPaused",
  "BorrowLimitExceeded",
  "PositionNotLiquidatable",
  "PoolAccountingInvariant",
  "bad_debt_usdg",
  "pending_authority",
  "pending_emergency_authority",
]) {
  if (!combined.includes(required)) throw new Error("Missing security control: " + required);
}

console.log("44 Milady static security checks: PASS");
