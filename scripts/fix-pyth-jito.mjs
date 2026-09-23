import fs from "node:fs";
import path from "node:path";

const root=path.resolve("node_modules/@pythnetwork/solana-utils/dist/esm");
if(!fs.existsSync(root)){
  console.log("Pyth solana-utils not installed; nothing to patch.");
  process.exit(0);
}

let changed=0;
for(const name of fs.readdirSync(root)){
  if(!name.endsWith(".mjs")) continue;
  const file=path.join(root,name);
  let text=fs.readFileSync(file,"utf8");
  const before=text;
  text=text
    .replaceAll('jito-ts/dist/sdk/block-engine/types"', 'jito-ts/dist/sdk/block-engine/types.js"')
    .replaceAll("jito-ts/dist/sdk/block-engine/types'", "jito-ts/dist/sdk/block-engine/types.js'")
    .replaceAll('jito-ts/dist/sdk/block-engine/searcher"', 'jito-ts/dist/sdk/block-engine/searcher.js"')
    .replaceAll("jito-ts/dist/sdk/block-engine/searcher'", "jito-ts/dist/sdk/block-engine/searcher.js'");
  if(text!==before){
    fs.writeFileSync(file,text);
    changed++;
  }
}
console.log(`Patched Pyth/Jito ESM imports in ${changed} file(s).`);
