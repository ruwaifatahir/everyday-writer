# Release check: fails unless a draft release carries both Builds (x64 and ARM64), each installer with
# its update signature, and a latest.json whose entries point each Build at its own installer.
# Reads files only, so it runs the same against fixtures: see release-check.tests.ps1.
#   -Version         app version without the `v`, e.g. 1.4.0
#   -ReleasePath     the release as one item of the GitHub REST API's release list (GET /repos/{owner}/{repo}/releases)
#   -LatestJsonPath  the release's latest.json; a path that doesn't exist means it couldn't be downloaded
param(
  [Parameter(Mandatory)] [string]$Version,
  [Parameter(Mandatory)] [string]$ReleasePath,
  [Parameter(Mandatory)] [string]$LatestJsonPath
)
$ErrorActionPreference = 'Stop'

$release = Get-Content -Raw $ReleasePath | ConvertFrom-Json
$errors = [Collections.Generic.List[string]]::new()

# One entry per Build: its installer (Tauri's default name) and its platform key in latest.json.
$builds = @(
  @{ Installer = "Everyday.Writer_${Version}_x64-setup.exe"; Platform = 'windows-x86_64' }
  @{ Installer = "Everyday.Writer_${Version}_arm64-setup.exe"; Platform = 'windows-aarch64' }
)

function Find-Asset([string]$name) {
  @($release.assets | Where-Object { $_.name -eq $name -and $_.state -eq 'uploaded' -and $_.size -gt 0 })[0]
}

# Publishing is the maintainer's call (ADR 0002); the workflow must only ever make drafts.
if ($release.draft -ne $true) { $errors.Add('The release is not a draft') }

foreach ($build in $builds) {
  if (-not (Find-Asset $build.Installer)) { $errors.Add("Missing installer $($build.Installer)") }
  if (-not (Find-Asset "$($build.Installer).sig")) { $errors.Add("Missing update signature $($build.Installer).sig") }
}

if (-not (Find-Asset 'latest.json') -or -not (Test-Path $LatestJsonPath)) {
  $errors.Add('Missing latest.json')
} else {
  $latest = Get-Content -Raw $LatestJsonPath | ConvertFrom-Json
  if ($latest.version -ne $Version) { $errors.Add("latest.json is for version '$($latest.version)', not $Version") }

  foreach ($build in $builds) {
    # tauri-action writes `<platform>` and `<platform>-nsis`; the updater may read either, so check both.
    foreach ($key in $build.Platform, "$($build.Platform)-nsis") {
      $entry = $latest.platforms.$key
      if (-not $entry) { $errors.Add("latest.json has no '$key' entry"); continue }
      # tauri-action v1 writes the asset's API URL; accept the browser download URL too.
      $asset = @($release.assets | Where-Object { $_.url -eq $entry.url -or $_.browser_download_url -eq $entry.url })[0]
      if (-not $asset) {
        $errors.Add("latest.json '$key' points at $($entry.url), which is not an asset on this release")
      } elseif ($asset.name -ne $build.Installer) {
        $errors.Add("latest.json '$key' points at $($asset.name), not $($build.Installer)")
      }
      if ([string]::IsNullOrWhiteSpace($entry.signature)) { $errors.Add("latest.json '$key' has an empty signature") }
    }
  }
}

foreach ($e in $errors) { Write-Output "::error::$e" }
if ($errors.Count -gt 0) { exit 1 }
Write-Output 'Release check passed: both Builds, their signatures and latest.json are on the draft.'
