# Tests for release-check.ps1, run against the fixtures in fixtures/release-check.
# Each case starts from the good fixtures, breaks one thing, and expects the check to fail.
# Run: pwsh .github/scripts/release-check.tests.ps1 (or Windows PowerShell 5.1). Exits 1 if any case fails.
$ErrorActionPreference = 'Stop'

$check = Join-Path $PSScriptRoot 'release-check.ps1'
$fixtures = Join-Path $PSScriptRoot 'fixtures/release-check'
$shell = (Get-Process -Id $PID).Path
$work = Join-Path ([IO.Path]::GetTempPath()) "release-check-tests-$PID"
New-Item -ItemType Directory -Force $work | Out-Null

function Read-Fixture([string]$name) {
  Get-Content -Raw (Join-Path $fixtures $name) | ConvertFrom-Json
}

# Runs the check on (possibly edited) copies of the fixtures; returns its exit code and output.
function Invoke-Check([scriptblock]$edit = {}, [string]$version = '1.4.0') {
  $release = Read-Fixture 'release.json'
  $latest = Read-Fixture 'latest.json'
  & $edit $release $latest
  $releasePath = Join-Path $work 'release.json'
  $latestPath = Join-Path $work 'latest.json'
  $release | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 $releasePath
  # Like the workflow: latest.json is only downloaded when the release has it.
  Remove-Item -Force -ErrorAction SilentlyContinue $latestPath
  if ($release.assets | Where-Object name -EQ 'latest.json') {
    $latest | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 $latestPath
  }
  # Windows PowerShell turns a native command's stderr into errors; don't let that stop the run.
  $ErrorActionPreference = 'Continue'
  $output = & $shell -NoProfile -ExecutionPolicy Bypass -File $check -Version $version -ReleasePath $releasePath -LatestJsonPath $latestPath 2>&1
  [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = ($output | Out-String) }
}

function Remove-Asset($release, [string]$name) {
  $release.assets = @($release.assets | Where-Object { $_.name -ne $name })
}

$cases = @(
  @{ Name = 'passes on a good draft'; Pass = $true; Edit = {} }
  @{ Name = 'fails when the ARM64 installer is missing'; Pass = $false; Edit = {
      param($release) Remove-Asset $release 'Everyday.Writer_1.4.0_arm64-setup.exe' } }
  @{ Name = 'fails when the x64 installer is missing'; Pass = $false; Edit = {
      param($release) Remove-Asset $release 'Everyday.Writer_1.4.0_x64-setup.exe' } }
  @{ Name = 'fails when the ARM64 .sig is missing'; Pass = $false; Edit = {
      param($release) Remove-Asset $release 'Everyday.Writer_1.4.0_arm64-setup.exe.sig' } }
  @{ Name = 'fails when the x64 .sig is empty'; Pass = $false; Edit = {
      param($release) ($release.assets | Where-Object name -EQ 'Everyday.Writer_1.4.0_x64-setup.exe.sig').size = 0 } }
  @{ Name = 'fails when latest.json is not on the release'; Pass = $false; Edit = {
      param($release) Remove-Asset $release 'latest.json' } }
  # One matrix job overwriting the other's latest.json leaves only its own Build's keys.
  @{ Name = 'fails when latest.json has no windows-aarch64 entry'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.PSObject.Properties.Remove('windows-aarch64') } }
  @{ Name = 'fails when latest.json has no windows-x86_64-nsis entry'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.PSObject.Properties.Remove('windows-x86_64-nsis') } }
  @{ Name = 'fails when latest.json is for another version'; Pass = $false; Edit = {
      param($release, $latest) $latest.version = '1.3.1' } }
  @{ Name = 'fails when windows-aarch64 points at the x64 installer'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.'windows-aarch64'.url = $latest.platforms.'windows-x86_64'.url } }
  @{ Name = 'fails when windows-x86_64-nsis points at the ARM64 installer'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.'windows-x86_64-nsis'.url = $latest.platforms.'windows-aarch64'.url } }
  @{ Name = 'fails when an entry points at an asset not on the release'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.'windows-x86_64'.url = 'https://api.github.com/repos/ruwaifatahir/everyday-writer/releases/assets/999' } }
  @{ Name = 'fails when the windows-aarch64 signature is empty'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.'windows-aarch64'.signature = '' } }
  @{ Name = 'fails when the windows-x86_64-nsis signature is whitespace'; Pass = $false; Edit = {
      param($release, $latest) $latest.platforms.'windows-x86_64-nsis'.signature = '  ' } }
  @{ Name = 'fails when the release is already published'; Pass = $false; Edit = {
      param($release) $release.draft = $false } }
)

$failed = 0
foreach ($case in $cases) {
  $result = Invoke-Check $case.Edit
  $passed = $result.ExitCode -eq 0
  if ($passed -eq $case.Pass) {
    Write-Output "ok    $($case.Name)"
  } else {
    $failed++
    Write-Output "FAIL  $($case.Name) (exit $($result.ExitCode))"
    Write-Output $result.Output
  }
}

Remove-Item -Recurse -Force $work
if ($failed -gt 0) {
  Write-Output "$failed of $($cases.Count) cases failed"
  exit 1
}
Write-Output "all $($cases.Count) cases passed"
