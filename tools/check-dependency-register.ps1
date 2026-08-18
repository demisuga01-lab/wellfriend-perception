$register = Join-Path $PSScriptRoot '..\third_party\dependency-register.toml'
if (-not (Test-Path $register)) { throw 'dependency register is missing' }
$content = Get-Content -Raw -LiteralPath $register
if ($content -notmatch '\[\[dependency\]\]') { throw 'dependency register has no entries' }
Write-Host 'dependency register placeholder check passed'

