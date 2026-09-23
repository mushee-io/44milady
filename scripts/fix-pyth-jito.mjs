import fs from "node:fs";
import path from "node:path";

const roots=[
  path.resolve("node_modules/@pythnetwork"),
];

function walk(dir,out=[]){
  if(!fs.existsSync(dir)) return out;
  for(const ent of fs.readdirSync(dir,{withFileTypes:true})){
    const p=path.join(dir,ent.name);
    if(ent.isDirectory()) walk(p,out);
    else if(ent.isFile() && (p.endsWith(".mjs") || p.endsWith(".js"))) out.push(p);
  }
  return out;
}

let changed=0;
let scanned=0;

for(const root of roots){
  for(const file of walk(root)){
    scanned++;
    let text=fs.readFileSync(file,"utf8");
    const before=text;

    text=text
      .replaceAll('jito-ts/dist/sdk/block-engine/types"', 'jito-ts/dist/sdk/block-engine/types.js"')
      .replaceAll("jito-ts/dist/sdk/block-engine/types'", "jito-ts/dist/sdk/block-engine/types.js'")
      .replaceAll('jito-ts/dist/sdk/block-engine/searcher"', 'jito-ts/dist/sdk/block-engine/searcher.js"')
      .replaceAll("jito-ts/dist/sdk/block-engine/searcher'", "jito-ts/dist/sdk/block-engine/searcher.js'")
      .replaceAll('@coral-xyz/anchor/dist/cjs/utils/bytes"', '@coral-xyz/anchor/dist/cjs/utils/bytes/index.js"')
      .replaceAll("@coral-xyz/anchor/dist/cjs/utils/bytes'", "@coral-xyz/anchor/dist/cjs/utils/bytes/index.js'");

    if(text!==before){
      fs.writeFileSync(file,text);
      changed++;
    }
  }
}

console.log(`Patched Node 24 ESM compatibility in ${changed} file(s); scanned ${scanned} Pyth JS files.`);
