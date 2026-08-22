[CmdletBinding()]
param(
    [ValidateSet("debug", "release")][string]$Profile = "release",
    [string]$NdkRoot = $env:ANDROID_NDK_ROOT,
    [string[]]$Abi = @("arm64-v8a", "x86_64")
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if (-not $NdkRoot) { $NdkRoot = $env:ANDROID_NDK_HOME }
if (-not $NdkRoot -and $env:ANDROID_SDK_ROOT) {
    $NdkRoot = Get-ChildItem (Join-Path $env:ANDROID_SDK_ROOT "ndk") -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $NdkRoot -or -not (Test-Path $NdkRoot)) { throw "Android NDK not found. Set ANDROID_NDK_ROOT or pass -NdkRoot." }
$NdkHost = Join-Path $NdkRoot "toolchains\llvm\prebuilt\windows-x86_64"
if (-not (Test-Path $NdkHost)) { throw "Windows NDK LLVM toolchain not found at $NdkHost" }

$Targets = @{ "arm64-v8a" = @{ Rust = "aarch64-linux-android"; Clang = "aarch64-linux-android21-clang.cmd"; Include = "aarch64-linux-android" }; "x86_64" = @{ Rust = "x86_64-linux-android"; Clang = "x86_64-linux-android21-clang.cmd"; Include = "x86_64-linux-android" } }
$Output = Join-Path $RepoRoot "target\wellfriend-android"
New-Item -ItemType Directory -Force -Path $Output | Out-Null
$records = @()
foreach ($Name in $Abi) {
    if (-not $Targets.ContainsKey($Name)) { throw "Unsupported ABI: $Name" }
    $target = $Targets[$Name]
    & rustup target add $target.Rust
    if ($LASTEXITCODE -ne 0) { throw "Could not install Rust target $($target.Rust)" }
    $envKey = "CARGO_TARGET_$($target.Rust.ToUpper().Replace('-', '_'))_LINKER"
    Set-Item -Path "Env:$envKey" -Value (Join-Path $NdkHost "bin\$($target.Clang)")
    # Android 15+ devices may use 16 KiB pages. Keep every PT_LOAD segment compatible.
    $rustFlagsKey = "CARGO_TARGET_$($target.Rust.ToUpper().Replace('-', '_'))_RUSTFLAGS"
    Set-Item -Path "Env:$rustFlagsKey" -Value "-C link-arg=-Wl,-z,max-page-size=16384"
    $cargoArgs = @("build", "--locked", "-p", "wellfriend-perception-ffi", "--target", $target.Rust)
    if ($Profile -eq "release") { $cargoArgs += "--release" }
    Push-Location $RepoRoot
    try { & cargo @cargoArgs; if ($LASTEXITCODE -ne 0) { throw "Cargo Android build failed for $Name" } } finally { Pop-Location }
    $profileDirectory = if ($Profile -eq "release") { "release" } else { "debug" }
    $rustLibrary = Join-Path $RepoRoot "target\$($target.Rust)\$profileDirectory\libwellfriend_perception.so"
    if (-not (Test-Path $rustLibrary)) { throw "Missing Rust library: $rustLibrary" }
    $abiOutput = Join-Path $Output $Name
    New-Item -ItemType Directory -Force -Path $abiOutput | Out-Null
    Copy-Item -Force $rustLibrary (Join-Path $abiOutput "libwellfriend_perception.so")
    $clang = Join-Path $NdkHost "bin\$($target.Clang)"
    $jniSource = Join-Path $RepoRoot "bindings\android-jni\wellfriend_perception_jni.c"
    $include = Join-Path $NdkHost "sysroot\usr\include"
    $archInclude = Join-Path $include $target.Include
    & $clang -shared -fPIC "-I$include" "-I$archInclude" "-I$(Join-Path $RepoRoot 'bindings\ffi\include')" $jniSource "-L$abiOutput" -lwellfriend_perception "-Wl,-soname,libwellfriend_perception_jni.so" "-Wl,-z,max-page-size=16384" "-o$(Join-Path $abiOutput 'libwellfriend_perception_jni.so')"
    if ($LASTEXITCODE -ne 0) { throw "JNI shim build failed for $Name" }
    foreach ($library in @("libwellfriend_perception.so", "libwellfriend_perception_jni.so")) {
        $file = Join-Path $abiOutput $library
        $records += [ordered]@{ abi = $Name; target = $target.Rust; file = "$Name/$library"; sha256 = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = (Get-Item $file).Length }
    }
}
& (Join-Path $PSScriptRoot "check-android-page-size.ps1") -ArtifactRoot $Output -NdkRoot $NdkRoot
if ($LASTEXITCODE -ne 0) { throw "Android ABI page-size validation failed" }
$manifest = [ordered]@{ schema_version = 1; artifact_kind = "wellfriend-android-abi"; source_sha = (& git -C $RepoRoot rev-parse HEAD).Trim(); rust_version = (& rustc --version).Trim(); ndk_root = $NdkRoot; ndk_version = Split-Path $NdkRoot -Leaf; profile = $Profile; page_size_alignment_bytes = 16384; libraries = $records; generated_at_utc = (Get-Date).ToUniversalTime().ToString("o") }
$utf8NoBom = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText((Join-Path $Output "manifest.json"), ($manifest | ConvertTo-Json -Depth 6), $utf8NoBom)
[IO.File]::WriteAllText((Join-Path $Output "checksums.json"), ([ordered]@{ schema_version = 1; files = $records | ForEach-Object { [ordered]@{ path = $_.file; sha256 = $_.sha256 } } } | ConvertTo-Json -Depth 5), $utf8NoBom)
Write-Host "Built Android ABI artifacts in $Output"
