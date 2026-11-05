# Run on Windows from an extracted portable Ramo source candidate.
# No prerequisite installs, credential access, signing or publishing.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:OS -ne 'Windows_NT') { throw 'A Windows host is required.' }
Set-Location (Split-Path $PSScriptRoot -Parent)
foreach ($command in @('node', 'npm.cmd', 'cargo', 'git')) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Missing $command. See docs/WINDOWS-BUILD.md for prerequisites."
    }
}
function Invoke-Checked([string] $Program, [string[]] $Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed (exit $LASTEXITCODE). No installer is ready to share." }
}
$version = (Get-Content package.json -Raw | ConvertFrom-Json).version
$output = Join-Path (Get-Location) "releases/local/$version/windows"
if (Test-Path $output) { throw "Output already exists: $output. Preserve previous packages." }
Invoke-Checked 'npm.cmd' @('ci')
Invoke-Checked 'npm.cmd' @('run', 'check:release')
Invoke-Checked 'npm.cmd' @('test')
Invoke-Checked 'npm.cmd' @('run', 'test:git')
Invoke-Checked 'npm.cmd' @('run', 'bundle', '--', '--bundles', 'nsis')
$installers = @(Get-ChildItem 'src-tauri/target/release/bundle/nsis' -Filter "*${version}*-setup.exe" -File)
if ($installers.Count -ne 1) { throw 'Expected exactly one version-matching NSIS installer.' }
New-Item -ItemType Directory -Path $output | Out-Null
$installer = Copy-Item $installers[0].FullName $output -PassThru
$hash = (Get-FileHash $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText((Join-Path $output 'SHA256SUMS'), "$hash  $($installer.Name)`n", [Text.UTF8Encoding]::new($false))
Copy-Item -Path 'LICENSE', 'NOTICE', 'THIRD_PARTY_NOTICES.txt', 'docs/WINDOWS-BUILD.md' -Destination $output
Write-Host "Unsigned Windows candidate: $output"
Write-Host 'Build success is not install/runtime acceptance. Complete the documented smoke checks before sharing.'
