[CmdletBinding()]
param([ValidateSet("debug", "release")][string]$Profile = "release")
$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Output = Join-Path $RepoRoot "target\wellfriend-wasm"
New-Item -ItemType Directory -Force -Path $Output | Out-Null
& rustup target add wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { throw "Unable to install wasm32-unknown-unknown" }
$cargoArgs = @("build", "--locked", "-p", "wellfriend-perception-wasm", "--target", "wasm32-unknown-unknown")
if ($Profile -eq "release") { $cargoArgs += "--release" }
Push-Location $RepoRoot
try { & cargo @cargoArgs; if ($LASTEXITCODE -ne 0) { throw "WASM cargo build failed" } } finally { Pop-Location }
$profileDirectory = if ($Profile -eq "release") { "release" } else { "debug" }
$raw = Join-Path $RepoRoot "target\wasm32-unknown-unknown\$profileDirectory\wellfriend_perception_wasm.wasm"
if (-not (Test-Path $raw)) { throw "Raw WASM artifact missing: $raw" }
& wasm-bindgen $raw --target web --out-name wellfriend_perception --out-dir $Output --typescript
if ($LASTEXITCODE -ne 0) { throw "wasm-bindgen generation failed. Match the CLI version to Cargo.lock." }
'{"type":"module","private":true}' | Set-Content -NoNewline -Encoding utf8 (Join-Path $Output "package.json")
$files = @("wellfriend_perception_bg.wasm", "wellfriend_perception.js", "wellfriend_perception.d.ts", "package.json") | ForEach-Object {
    $file = Join-Path $Output $_
    if (-not (Test-Path $file)) { throw "Generated WASM package file missing: $file" }
    [ordered]@{ path = $_; sha256 = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = (Get-Item $file).Length }
}
$manifest = [ordered]@{ schema_version = 1; artifact_kind = "wellfriend-wasm-package"; source_sha = (& git -C $RepoRoot rev-parse HEAD).Trim(); rust_version = (& rustc --version).Trim(); wasm_bindgen_version = (& wasm-bindgen --version).Trim(); target = "wasm32-unknown-unknown"; profile = $Profile; exports = @("createEngine", "destroyEngine", "version", "EngineHandle.analyzeFrame", "EngineHandle.reconstructPage", "EngineHandle.applyFilter"); runtime_schema_version = 1; files = $files; generated_at_utc = (Get-Date).ToUniversalTime().ToString("o") }
$utf8NoBom = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText((Join-Path $Output "manifest.json"), ($manifest | ConvertTo-Json -Depth 6), $utf8NoBom)
[IO.File]::WriteAllText((Join-Path $Output "checksums.json"), ([ordered]@{ schema_version = 1; files = $files | ForEach-Object { [ordered]@{ path = $_.path; sha256 = $_.sha256 } } } | ConvertTo-Json -Depth 5), $utf8NoBom)
@"
# Wellfriend Perception WASM package

Generated scalar-runtime package. Validate manifest and checksums before local consumption. No model weights are included.
"@ | Set-Content -NoNewline -Encoding utf8 (Join-Path $Output "README.md")
Write-Host "Built browser WASM package in $Output"
