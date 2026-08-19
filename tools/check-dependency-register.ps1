$register = Join-Path $PSScriptRoot '..\third_party\dependency-register.toml'
if (-not (Test-Path $register)) { throw 'dependency register is missing' }
$entries = (Get-Content -Raw -LiteralPath $register) -split '(?m)^\[\[dependency\]\]' | Select-Object -Skip 1
if ($entries.Count -eq 0) { throw 'dependency register has no entries' }
$required = @('name', 'version', 'license', 'source_url', 'purpose', 'risk_level', 'status', 'used_by')
foreach ($entry in $entries) {
    foreach ($field in $required) {
        if ($entry -notmatch "(?m)^$field\s*=") {
            throw "dependency entry is missing required field: $field"
        }
    }
    if ($entry -match '(?im)license\s*=\s*"[^"]*(GPL|AGPL|LGPL|non-commercial|research-only)[^"]*"') {
        throw 'dependency register declares a prohibited license category'
    }
}
Write-Host "dependency register validation passed for $($entries.Count) entries"
