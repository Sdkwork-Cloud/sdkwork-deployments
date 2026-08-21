#Requires -RunAsAdministrator
<#
.SYNOPSIS
  Bind SDKWork API gateway domains to 127.0.0.1 in the Windows hosts file.

.DESCRIPTION
  WSL2 forwards localhost to Windows 127.0.0.1, so nginx on WSL port 80
  is reachable from Windows browsers via these hostnames. Supports all
  brand domains (sdkwork.com, birdcoder.com, dtupay.com) and all environments.

.PARAMETER Environments
  Comma-separated environments to bind (default: development,test,staging,production)

.PARAMETER ModuleName
  Module name for comment identification

.PARAMETER Verify
  Verify DNS resolution after binding

.EXAMPLE
  ../sdkwork-deployments/scripts/bind-windows-hosts.ps1
  ../sdkwork-deployments/scripts/bind-windows-hosts.ps1 -Environments development,production
#>
[CmdletBinding()]
param(
  [string]$Environments = 'development,test,staging,production',
  [string]$ModuleName = 'sdkwork-api-cloud-gateway',
  [switch]$Verify
)

$ErrorActionPreference = 'Stop'

# ============================================================================
# Configuration
# ============================================================================
$BrandDomains = @('sdkwork.com', 'birdcoder.com', 'dtupay.com')

$PortMap = @{
  development = 3910
  test        = 3911
  staging     = 3912
  production  = 3913
}

function Get-EnvironmentSubdomain {
  param([string]$Environment)
  switch ($Environment) {
    'development' { 'api-dev' }
    'test' { 'api-test' }
    'staging' { 'api-staging' }
    'production' { 'api' }
    default { throw "unsupported environment: $Environment" }
  }
}

# ============================================================================
# Build hosts entries
# ============================================================================
$hostsPath = "$env:SystemRoot\System32\drivers\etc\hosts"
$markerBegin = '# === SDKWORK DOMAINS BEGIN ==='
$markerEnd = '# === SDKWORK DOMAINS END ==='

$lines = @()
$lines += $markerBegin
$lines += "# SDKWork $ModuleName (WSL nginx :80 -> container ports)"
$lines += "# Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"

$envList = $Environments -split ',' | ForEach-Object { $_.Trim() }
foreach ($env in $envList) {
  $subdomain = Get-EnvironmentSubdomain $env
  $entries = @()
  foreach ($domain in $BrandDomains) {
    $entries += "$subdomain.$domain"
  }
  $lines += "127.0.0.1 $($entries -join ' ')"
}

$lines += $markerEnd
$domainBlock = $lines -join "`r`n"

# ============================================================================
# Update hosts file
# ============================================================================
$content = Get-Content -Path $hostsPath -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
if ($content -match [regex]::Escape($markerBegin)) {
  $pattern = "(?s)$([regex]::Escape($markerBegin)).*?$([regex]::Escape($markerEnd))"
  $content = [regex]::Replace($content, $pattern, $domainBlock)
} else {
  $content = $content.TrimEnd() + "`r`n`r`n" + $domainBlock + "`r`n"
}

Set-Content -Path $hostsPath -Value $content -Encoding UTF8 -NoNewline
Write-Host "Updated Windows hosts file: $hostsPath" -ForegroundColor Green

# ============================================================================
# Verify (optional)
# ============================================================================
if ($Verify) {
  Write-Host ""
  Write-Host "Verifying DNS resolution..." -ForegroundColor Cyan
  foreach ($env in $envList) {
    $subdomain = Get-EnvironmentSubdomain $env
    $domain = "$subdomain.$($BrandDomains[0])"
    try {
      $result = Resolve-DnsName -Name $domain -Type A -ErrorAction Stop | Select-Object -First 1
      Write-Host "  [OK] $domain -> $($result.IPAddress)" -ForegroundColor Green
    } catch {
      Write-Warning "  [FAIL] $domain - resolution failed"
    }
  }
}

Write-Host ""
Write-Host "Hosts binding complete for environments: $Environments" -ForegroundColor Green
