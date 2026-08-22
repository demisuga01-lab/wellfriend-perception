#!/usr/bin/env bash
set -euo pipefail
ARTIFACT_ROOT="${1:?artifact root required}"
NDK_ROOT="${2:-${ANDROID_NDK_ROOT:-${ANDROID_NDK_HOME:-}}}"
[[ -n "$NDK_ROOT" && -d "$NDK_ROOT" ]] || { echo "Set ANDROID_NDK_ROOT to an Android NDK." >&2; exit 1; }
READELF="$NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"
[[ -x "$READELF" ]] || { echo "NDK llvm-readelf missing: $READELF" >&2; exit 1; }

mapfile -t libraries < <(find "$ARTIFACT_ROOT" -type f -name 'libwellfriend_perception*.so' | sort)
(( ${#libraries[@]} > 0 )) || { echo "No Wellfriend Android shared libraries found under $ARTIFACT_ROOT" >&2; exit 1; }
for library in "${libraries[@]}"; do
  mapfile -t loads < <("$READELF" -lW "$library" | awk '$1 == "LOAD" { print $NF }')
  (( ${#loads[@]} > 0 )) || { echo "No PT_LOAD segments found in $library" >&2; exit 1; }
  for alignment in "${loads[@]}"; do
    value=$((alignment))
    (( value >= 16384 )) || { echo "$library has PT_LOAD alignment $alignment; 0x4000 (16 KiB) is required" >&2; exit 1; }
  done
  echo "16 KiB PT_LOAD alignment verified: $library"
done
