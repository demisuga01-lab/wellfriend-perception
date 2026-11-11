# Android 16 KiB native page compatibility

Wellfriend Android ABI artifacts target 16 KiB-compatible `PT_LOAD` alignment for both
`libwellfriend_perception.so` and `libwellfriend_perception_jni.so`. This avoids Android
page-size compatibility warnings on devices that use 16 KiB memory pages.

The ABI builders pass `-Wl,-z,max-page-size=16384` to the Rust link step and the JNI shim
link step. They run the matching page-size validator before writing the artifact manifest.

On Windows:

```powershell
$env:ANDROID_NDK_ROOT = "$env:LOCALAPPDATA\Android\Sdk\ndk\27.1.12297006"
.\scripts\build-android-abi.ps1 -Profile release
.\scripts\check-android-page-size.ps1 -ArtifactRoot .\target\wellfriend-android
```

On Linux:

```bash
ANDROID_NDK_ROOT=/path/to/ndk ./scripts/build-android-abi.sh release
./scripts/check-android-page-size.sh target/wellfriend-android "$ANDROID_NDK_ROOT"
```

The validator inspects every `PT_LOAD` segment with the NDK `llvm-readelf` and rejects
alignment below `0x4000`. It validates ELF libraries, not physical-device camera behavior.
