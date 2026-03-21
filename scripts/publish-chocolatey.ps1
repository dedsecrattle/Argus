# Script to publish Argus to Chocolatey
# Must be run on Windows with Chocolatey installed

param(
    [switch]$Test
)

Write-Host "=== Publishing Argus to Chocolatey ===" -ForegroundColor Green

# Check if choco is installed
if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Chocolatey not installed. Install from https://chocolatey.org/install" -ForegroundColor Red
    exit 1
}

# Check if we're on Windows
if ($PSVersionTable.PSVersion.Major -lt 6 -and -not $IsWindows) {
    Write-Host "Error: Chocolatey publishing must be done on Windows" -ForegroundColor Red
    exit 1
}

# Build the package
Write-Host "Building Chocolatey package..."
Set-Location chocolatey
choco pack

# Check if package was built
$package = Get-ChildItem "*.nupkg" | Select-Object -First 1
if (-not $package) {
    Write-Host "Error: Package build failed" -ForegroundColor Red
    exit 1
}

if ($Test) {
    Write-Host "Testing package locally..." -ForegroundColor Yellow
    choco install argus -s . -y
    
    Write-Host "Testing installation..." -ForegroundColor Yellow
    argus --help
    
    Write-Host "Uninstalling test package..." -ForegroundColor Yellow
    choco uninstall argus -y
    
    Write-Host "✅ Package test successful!" -ForegroundColor Green
} else {
    Write-Host "Pushing to Chocolatey gallery..."
    Write-Host "Make sure you have:"
    Write-Host "  1. A Chocolatey account at https://chocolatey.org/"
    Write-Host "  2. Run 'choco apikey -k YOUR_API_KEY -source https://push.chocolatey.org/'"
    Write-Host ""
    
    $confirm = Read-Host "Continue pushing to Chocolatey? (y/N)"
    if ($confirm -eq 'y') {
        choco push $package.Name
        Write-Host "✅ Package published to Chocolatey!" -ForegroundColor Green
        Write-Host "Visit: https://chocolatey.org/packages/argus"
    } else {
        Write-Host "Publishing cancelled" -ForegroundColor Yellow
    }
}
