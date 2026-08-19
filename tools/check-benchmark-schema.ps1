$schema = Join-Path $PSScriptRoot '..\benchmarks\schema\scanbench-record.schema.json'
$example = Join-Path $PSScriptRoot '..\benchmarks\schema\mp2-scalar-baseline.example.json'
foreach ($path in @($schema, $example)) {
    if (-not (Test-Path $path)) { throw "benchmark artifact is missing: $path" }
}
$schemaObject = Get-Content -Raw -LiteralPath $schema | ConvertFrom-Json
$exampleObject = Get-Content -Raw -LiteralPath $example | ConvertFrom-Json
foreach ($field in $schemaObject.required) {
    if ($null -eq $exampleObject.$field) { throw "benchmark example is missing required field: $field" }
}
if ($exampleObject.schema_version -ne 1) { throw 'benchmark example has an unsupported schema version' }
if ($null -eq $exampleObject.metrics.operation -or $exampleObject.metrics.elapsed_nanoseconds -lt 0) {
    throw 'benchmark example metrics are incomplete'
}
Write-Host 'benchmark schema validation passed'
