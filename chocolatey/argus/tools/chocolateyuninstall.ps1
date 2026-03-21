$ErrorActionPreference = 'Stop'

$packageName = 'argus'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

# Remove shim
Uninstall-BinFile -Name $packageName

# Uninstall argus-crawler using cargo
Write-Host "Uninstalling argus-crawler..."
$process = Start-Process -FilePath 'cargo' -ArgumentList 'uninstall', 'argus-crawler' -Wait -PassThru
# Don't fail if cargo uninstall fails (it might not be installed)

# Remove installed binary
$binPath = Join-Path $toolsDir 'argus.exe'
if (Test-Path $binPath) {
    Remove-Item $binPath -Force
    Write-Host "Argus uninstalled successfully!"
}
