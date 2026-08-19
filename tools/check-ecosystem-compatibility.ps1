$ErrorActionPreference = 'Stop'
$manifest = Get-Content -Raw -LiteralPath "$PSScriptRoot/../docs/ecosystem-compatibility.json" | ConvertFrom-Json
if ($manifest.schema_version -ne 1 -or $manifest.ecosystem_version -ne '0.1.0-alpha.1') { throw 'invalid ecosystem compatibility schema/version' }
foreach ($field in 'device_classes','document_tasks','filter_presets','processor_ids','guidance_codes','export_formats') { if (@($manifest.shared_contracts.$field).Count -eq 0) { throw "compatibility field $field is empty" } }
if (@($manifest.known_blockers).Count -eq 0 -or @($manifest.mock_boundaries).Count -eq 0) { throw 'release blockers and mock boundaries must be disclosed' }
Write-Host 'Ecosystem compatibility manifest passed'
