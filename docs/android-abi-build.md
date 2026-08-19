# Android ABI artifacts

MP10B packages the scalar runtime as `libwellfriend_perception.so` plus a thin
`libwellfriend_perception_jni.so` transport.  The JNI library owns no perception
algorithm; it calls the reviewed C ABI only.

Run PowerShell on Windows:

```powershell
$env:ANDROID_NDK_ROOT = "C:\\Android\\Sdk\\ndk\\27.1.12297006"
.\scripts\build-android-abi.ps1 -Profile release
.\scripts\validate-runtime-artifact.ps1 -ArtifactRoot target\wellfriend-android -Kind android
```

On Linux, set `ANDROID_NDK_ROOT` and run `./scripts/build-android-abi.sh release`.
The package includes arm64-v8a and x86_64 libraries, `manifest.json`, and
`checksums.json`. Its manifest records source SHA, Rust version, NDK version,
profile, ABI/target, sizes, and SHA-256 checksums. Build outputs are intentionally
ignored by Git; scan consumes them through its verified sync script or a CI artifact.

The manual **Android ABI artifacts** workflow creates the same package. It does
not assert device performance or document-detection quality.
