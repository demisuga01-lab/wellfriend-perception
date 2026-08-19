# Android binding

Android uses the C ABI through a minimal NDK JNI shim. The shim contains no detector, homography, or restoration code; it converts Java byte arrays and strings to `wf_*` calls and returns their JSON result. The source seam is in `wellfriend-scan/android/scanner-perception/src/main/jni`.

MP10 builds the Rust C ABI on the host but does not package Android `.so` files. Until an NDK build produces reviewed ABI-specific artifacts, release scanners fail closed and debug scanners display their dev-mock status instead of presenting mock output as native perception.
