#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:?artifact root required}"
KIND="${2:?android or wasm required}"
python3 - "$ROOT" "$KIND" <<'PY'
import hashlib,json,os,re,sys
root,kind=sys.argv[1:]
with open(os.path.join(root,'manifest.json')) as f: manifest=json.load(f)
with open(os.path.join(root,'checksums.json')) as f: checksums=json.load(f)
expected='wellfriend-android-abi' if kind == 'android' else 'wellfriend-wasm-package'
if manifest.get('schema_version') != 1 or checksums.get('schema_version') != 1 or manifest.get('artifact_kind') != expected or not re.fullmatch(r'[0-9a-f]{40}',manifest.get('source_sha','')): raise SystemExit('invalid runtime artifact manifest')
records=manifest.get('libraries' if kind == 'android' else 'files',[]); sums={x['path']:x['sha256'] for x in checksums.get('files',[])}
for record in records:
 path=record.get('file',record.get('path',''))
 if not path or path.startswith(('/', '\\')) or '..' in path.split('/'): raise SystemExit('unsafe artifact path')
 actual=hashlib.sha256(open(os.path.join(root,path),'rb').read()).hexdigest()
 if actual != record.get('sha256') or sums.get(path) != actual: raise SystemExit('checksum mismatch: '+path)
required=(('arm64-v8a/libwellfriend_perception.so','arm64-v8a/libwellfriend_perception_jni.so','x86_64/libwellfriend_perception.so','x86_64/libwellfriend_perception_jni.so') if kind == 'android' else ('wellfriend_perception_bg.wasm','wellfriend_perception.js','wellfriend_perception.d.ts','package.json'))
present={r.get('file',r.get('path')) for r in records}
if not set(required).issubset(present): raise SystemExit('required runtime files missing')
print('Validated '+expected+' artifact at '+root)
PY
