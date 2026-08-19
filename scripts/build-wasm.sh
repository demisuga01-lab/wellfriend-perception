#!/usr/bin/env bash
set -euo pipefail
PROFILE="${1:-release}"; ROOT="$(cd "$(dirname "$0")/.." && pwd)"; OUT="$ROOT/target/wellfriend-wasm"
mkdir -p "$OUT"; rustup target add wasm32-unknown-unknown
args=(build --locked -p wellfriend-perception-wasm --target wasm32-unknown-unknown); [[ "$PROFILE" == release ]] && args+=(--release)
(cd "$ROOT" && cargo "${args[@]}")
build=debug; [[ "$PROFILE" == release ]] && build=release
wasm-bindgen "$ROOT/target/wasm32-unknown-unknown/$build/wellfriend_perception_wasm.wasm" --target web --out-name wellfriend_perception --out-dir "$OUT" --typescript
printf '{"type":"module","private":true}' > "$OUT/package.json"
python3 - "$ROOT" "$OUT" "$PROFILE" <<'PY'
import hashlib,json,os,subprocess,sys,datetime
root,out,profile=sys.argv[1:]; names=['wellfriend_perception_bg.wasm','wellfriend_perception.js','wellfriend_perception.d.ts','package.json']; files=[]
for name in names:
 p=os.path.join(out,name)
 with open(p,'rb') as f: digest=hashlib.sha256(f.read()).hexdigest()
 files.append({'path':name,'sha256':digest,'bytes':os.path.getsize(p)})
manifest={'schema_version':1,'artifact_kind':'wellfriend-wasm-package','source_sha':subprocess.check_output(['git','-C',root,'rev-parse','HEAD'],text=True).strip(),'rust_version':subprocess.check_output(['rustc','--version'],text=True).strip(),'wasm_bindgen_version':subprocess.check_output(['wasm-bindgen','--version'],text=True).strip(),'target':'wasm32-unknown-unknown','profile':profile,'exports':['createEngine','destroyEngine','version','EngineHandle.analyzeFrame','EngineHandle.reconstructPage','EngineHandle.applyFilter'],'runtime_schema_version':1,'files':files,'generated_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
with open(os.path.join(out,'manifest.json'),'w') as f: json.dump(manifest,f,indent=2)
with open(os.path.join(out,'checksums.json'),'w') as f: json.dump({'schema_version':1,'files':[{'path':x['path'],'sha256':x['sha256']} for x in files]},f,indent=2)
PY
printf '# Wellfriend Perception WASM package\n\nGenerated scalar-runtime package. Validate manifest and checksums before local consumption.\n' > "$OUT/README.md"
