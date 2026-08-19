# Runtime artifact manifest contract

`schema_version: 1` artifacts require a 40-character source SHA, an expected
artifact kind, per-file SHA-256 entries in both `manifest.json` and
`checksums.json`, and safe relative paths. Android packages additionally require
both `libwellfriend_perception.so` and `libwellfriend_perception_jni.so` for
arm64-v8a and x86_64. WASM packages require the `.wasm`, loader, declarations,
and module marker.

`scripts/validate-runtime-artifact.ps1` rejects unsafe paths, absent files,
checksum mismatches, incorrect schema/kind, and incomplete required sets. A
manifest validates provenance and integrity; it is not a claim of real-device
performance, model provenance, or scanner accuracy.
