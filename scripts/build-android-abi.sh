#!/usr/bin/env bash
set -euo pipefail
PROFILE="${1:-release}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NDK_ROOT="${ANDROID_NDK_ROOT:-${ANDROID_NDK_HOME:-}}"
if [[ -z "$NDK_ROOT" || ! -d "$NDK_ROOT" ]]; then echo "Set ANDROID_NDK_ROOT to an Android NDK." >&2; exit 1; fi
HOST="$NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
[[ -d "$HOST" ]] || { echo "Linux NDK toolchain missing: $HOST" >&2; exit 1; }
OUT="$ROOT/target/wellfriend-android"; mkdir -p "$OUT"
records=()
for spec in "arm64-v8a:aarch64-linux-android:aarch64-linux-android21-clang:aarch64-linux-android" "x86_64:x86_64-linux-android:x86_64-linux-android21-clang:x86_64-linux-android"; do
  IFS=: read -r abi target clang arch <<< "$spec"
  rustup target add "$target"
  linker_key="CARGO_TARGET_${target^^}_LINKER"; linker_key="${linker_key//-/_}"
  export "$linker_key=$HOST/bin/$clang"
  cargo_args=(build --locked -p wellfriend-perception-ffi --target "$target"); [[ "$PROFILE" == release ]] && cargo_args+=(--release)
  (cd "$ROOT" && cargo "${cargo_args[@]}")
  build="debug"; [[ "$PROFILE" == release ]] && build="release"; mkdir -p "$OUT/$abi"
  cp "$ROOT/target/$target/$build/libwellfriend_perception.so" "$OUT/$abi/"
  "$HOST/bin/$clang" -shared -fPIC -I"$HOST/sysroot/usr/include" -I"$HOST/sysroot/usr/include/$arch" -I"$ROOT/bindings/ffi/include" "$ROOT/bindings/android-jni/wellfriend_perception_jni.c" -L"$OUT/$abi" -lwellfriend_perception -Wl,-soname,libwellfriend_perception_jni.so -o "$OUT/$abi/libwellfriend_perception_jni.so"
done
python3 - "$ROOT" "$OUT" "$PROFILE" "$NDK_ROOT" <<'PY'
import hashlib,json,os,subprocess,sys,datetime
root,out,profile,ndk=sys.argv[1:]
records=[]
targets={'arm64-v8a':'aarch64-linux-android','x86_64':'x86_64-linux-android'}
for abi in ('arm64-v8a','x86_64'):
 for name in ('libwellfriend_perception.so','libwellfriend_perception_jni.so'):
  p=os.path.join(out,abi,name)
  with open(p,'rb') as f: digest=hashlib.sha256(f.read()).hexdigest()
  records.append({'abi':abi,'target':targets[abi],'file':f'{abi}/{name}','sha256':digest,'bytes':os.path.getsize(p)})
manifest={'schema_version':1,'artifact_kind':'wellfriend-android-abi','source_sha':subprocess.check_output(['git','-C',root,'rev-parse','HEAD'],text=True).strip(),'rust_version':subprocess.check_output(['rustc','--version'],text=True).strip(),'ndk_root':ndk,'ndk_version':os.path.basename(ndk),'profile':profile,'libraries':records,'generated_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
with open(os.path.join(out,'manifest.json'),'w') as f: json.dump(manifest,f,indent=2)
with open(os.path.join(out,'checksums.json'),'w') as f: json.dump({'schema_version':1,'files':[{'path':x['file'],'sha256':x['sha256']} for x in records]},f,indent=2)
PY
