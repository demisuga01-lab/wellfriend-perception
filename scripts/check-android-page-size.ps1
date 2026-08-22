[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ArtifactRoot,
    [string]$NdkRoot = $env:ANDROID_NDK_ROOT
)

$ErrorActionPreference = "Stop"
if (-not $NdkRoot) { $NdkRoot = $env:ANDROID_NDK_HOME }
if (-not $NdkRoot -and $env:ANDROID_SDK_ROOT) {
    $NdkRoot = Get-ChildItem (Join-Path $env:ANDROID_SDK_ROOT "ndk") -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $NdkRoot -or -not (Test-Path $NdkRoot)) { throw "Android NDK not found. Set ANDROID_NDK_ROOT or pass -NdkRoot." }
$readElf = Join-Path $NdkRoot "toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-readelf.exe"
if (-not (Test-Path $readElf)) { throw "NDK llvm-readelf not found at $readElf" }

$libraries = Get-ChildItem -Path $ArtifactRoot -Recurse -Filter "libwellfriend_perception*.so" -File
if ($libraries.Count -eq 0) { throw "No Wellfriend Android shared libraries found under $ArtifactRoot" }
foreach ($library in $libraries) {
    $loads = @(& $readElf -lW $library.FullName | Where-Object { $_ -match '^\s*LOAD\s+' })
    if ($loads.Count -eq 0) { throw "No PT_LOAD segments found in $($library.FullName)" }
    foreach ($load in $loads) {
        $match = [regex]::Match($load, '(0x[0-9A-Fa-f]+)\s*$')
        if (-not $match.Success) { throw "Could not parse PT_LOAD alignment for $($library.FullName): $load" }
        $alignment = [Convert]::ToInt64($match.Groups[1].Value.Substring(2), 16)
        if ($alignment -lt 16384) { throw "$($library.FullName) has PT_LOAD alignment $($match.Groups[1].Value); 0x4000 (16 KiB) is required" }
    }
    Write-Host "16 KiB PT_LOAD alignment verified: $($library.FullName)"
}
