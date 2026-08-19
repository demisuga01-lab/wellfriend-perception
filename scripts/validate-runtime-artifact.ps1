[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ArtifactRoot,
    [ValidateSet("android", "wasm")][string]$Kind
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path $ArtifactRoot).Path
$manifestPath = Join-Path $root "manifest.json"
$checksumsPath = Join-Path $root "checksums.json"
if (-not (Test-Path $manifestPath) -or -not (Test-Path $checksumsPath)) { throw "Runtime artifact requires manifest.json and checksums.json" }
$manifest = Get-Content -Raw $manifestPath | ConvertFrom-Json
$checksums = Get-Content -Raw $checksumsPath | ConvertFrom-Json
if ($manifest.schema_version -ne 1 -or $checksums.schema_version -ne 1) { throw "Unsupported runtime artifact schema version" }
if ($manifest.source_sha -notmatch '^[0-9a-f]{40}$') { throw "Runtime artifact source_sha must be a Git SHA" }
$expectedKind = if ($Kind -eq "android") { "wellfriend-android-abi" } else { "wellfriend-wasm-package" }
if ($manifest.artifact_kind -ne $expectedKind) { throw "Expected $expectedKind artifact" }
$records = if ($Kind -eq "android") { $manifest.libraries } else { $manifest.files }
if (-not $records -or $records.Count -eq 0) { throw "Runtime artifact records are empty" }
foreach ($record in $records) {
    $relative = if ($record.file) { $record.file } else { $record.path }
    if (-not $relative -or [IO.Path]::IsPathRooted($relative) -or $relative.Contains("..")) { throw "Unsafe artifact path" }
    $file = Join-Path $root $relative
    if (-not (Test-Path $file)) { throw "Artifact file missing: $relative" }
    $actual = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $record.sha256) { throw "Checksum mismatch: $relative" }
    $checksum = $checksums.files | Where-Object { $_.path -eq $relative } | Select-Object -First 1
    if (-not $checksum -or $checksum.sha256 -ne $actual) { throw "Checksum manifest mismatch: $relative" }
}
if ($Kind -eq "android") {
    $expected = @("arm64-v8a/libwellfriend_perception.so", "arm64-v8a/libwellfriend_perception_jni.so", "x86_64/libwellfriend_perception.so", "x86_64/libwellfriend_perception_jni.so")
    foreach ($item in $expected) { if (-not ($records | Where-Object { $_.file -eq $item })) { throw "Required ABI library missing: $item" } }
} else {
    foreach ($item in @("wellfriend_perception_bg.wasm", "wellfriend_perception.js", "wellfriend_perception.d.ts", "package.json")) { if (-not ($records | Where-Object { $_.path -eq $item })) { throw "Required WASM package file missing: $item" } }
}
Write-Host "Validated $expectedKind artifact at $root"
