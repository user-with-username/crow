$repo = "user-with-username/crow"
$installDir = "$HOME\.crow\bin"
$executableName = "crow.exe"

function Write-Info ($msg) { Write-Host "info: " -ForegroundColor Cyan -NoNewline; Write-Host $msg }
function Write-Success ($msg) { Write-Host "success: " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Action ($msg) { Write-Host "action: " -ForegroundColor Yellow -NoNewline; Write-Host $msg }

Write-Host "`n  Crow Installer" -ForegroundColor Blue

if (!(Test-Path $installDir)) { New-Item -ItemType Directory -Force -Path $installDir | Out-Null }

$arch = $env:PROCESSOR_ARCHITECTURE
$artifact = if ($arch -eq "AMD64") { "windows-x64.exe" } else { "windows-x86.exe" }

Write-Info "fetching latest release metadata..."
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    $latestTag = $release.tag_name
} catch {
    Write-Error "Failed to fetch releases."
    exit
}

Write-Info "detected target ($artifact)"
Write-Info "downloading version $latestTag..."

$downloadUrl = "https://github.com/$repo/releases/download/$latestTag/$artifact"
$outputPath = Join-Path $installDir $executableName

$oldProgressPreference = $ProgressPreference
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath
$ProgressPreference = $oldProgressPreference

Write-Success "crow has been installed to $installDir`n"

$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$installDir*") {
    $newPath = "$currentPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Action "added crow to your PATH."
    Write-Host "Please restart your terminal to start using 'crow'." -ForegroundColor Gray
}

& "$outputPath" --version