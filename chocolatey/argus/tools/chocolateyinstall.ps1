$ErrorActionPreference = 'Stop'

$packageName = 'argus'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$binPath = Join-Path $toolsDir 'argus.exe'

# Download Rust installer
$rustInstallerUrl = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
$rustInstallerPath = Join-Path $toolsDir 'rustup-init.exe'

Write-Host "Downloading Rust installer..."
Get-WebFile -Url $rustInstallerUrl -FileName $rustInstallerPath

# Install Rust silently
Write-Host "Installing Rust..."
$process = Start-Process -FilePath $rustInstallerPath -ArgumentList '-y', '--quiet', '--default-toolchain', 'stable' -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "Rust installation failed with exit code $($process.ExitCode)"
}

# Add Rust to PATH for this session
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Install argus-crawler using cargo
Write-Host "Installing argus-crawler..."
$process = Start-Process -FilePath 'cargo' -ArgumentList 'install', 'argus-crawler' -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "Failed to install argus-crawler"
}

# Copy the installed binary to tools directory
$cargoBinPath = Join-Path $env:USERPROFILE '.cargo\bin\argus.exe'
if (Test-Path $cargoBinPath) {
    Copy-Item $cargoBinPath $binPath -Force
    Write-Host "Argus installed successfully!"
} else {
    throw "argus.exe not found after installation"
}

# Clean up Rust installer
Remove-Item $rustInstallerPath -Force

# Create shim
Write-Host "Creating command-line shim..."
Install-BinFile -Name $packageName -Path $binPath
