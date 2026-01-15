Set-StrictMode -Version Latest

$cols = 80
try {
    $cols = $Host.UI.RawUI.WindowSize.Width
} catch {
    # Defaults to 80
}

if ($cols -lt 48) {
    Write-Host "Welcome to rustytime!" -ForegroundColor Cyan
} else {
    $banner = @'
                 _         _   _                
  _ __ _   _ ___| |_ _   _| |_(_)_ __ ___   ___ 
 | '__| | | / __| __| | | | __| | '_ ` _ \ / _ \
 | |  | |_| \__ \ |_| |_| | |_| | | | | | |  __/
 |_|   \__,_|___/\__|\__, |\__|_|_| |_| |_|\___|
                     |___/                      
'@
    Write-Host $banner -ForegroundColor Cyan
}

Write-Host

$configPath = Join-Path -Path $HOME -ChildPath ".wakatime.cfg"
if (Test-Path -Path $configPath) {
    $timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $backupPath = "$configPath.$timestamp.bak"
    Move-Item -Path $configPath -Destination $backupPath -Force
    Write-Host "! Existing config file found. Backed up to $backupPath" -ForegroundColor Yellow
}

$vscodeExt = "WakaTime.vscode-wakatime"
$jetbrainsPid = "com.wakatime.intellij.plugin"

$apiKey = $env:RT_API_KEY
$apiUrl = $env:RT_API_URL

if ([string]::IsNullOrWhiteSpace($apiKey) -or [string]::IsNullOrWhiteSpace($apiUrl)) {
    Write-Host "Error: RT_API_KEY and RT_API_URL must be set" -ForegroundColor Red
    exit 1
}

$configContent = @"
[settings]
api_key = $apiKey
api_url = $apiUrl
heartbeat_rate_limit_seconds = 60
"@

Set-Content -Path $configPath -Value $configContent -Encoding UTF8
Write-Host "✓ Configuration file created at $configPath" -ForegroundColor Green
Write-Host

Write-Host "Installing WakaTime extensions..." -ForegroundColor White

function Install-VSCodeExtension {
    param(
        [Parameter(Mandatory = $true)][string]$CliName
    )

    if (Get-Command $CliName -ErrorAction SilentlyContinue) {
        Write-Host
        Write-Host "→ Installing WakaTime for $CliName..." -ForegroundColor Green
        & $CliName --install-extension $vscodeExt --force
    } else {
        Write-Host "$CliName CLI not found; skipping." -ForegroundColor DarkGray
    }
}

Install-VSCodeExtension -CliName "code"
Install-VSCodeExtension -CliName "trae"
Install-VSCodeExtension -CliName "cursor"
Install-VSCodeExtension -CliName "windsurf"

$ideExes = @(
    "idea64.exe", "pycharm64.exe", "clion64.exe", "goland64.exe",
    "webstorm64.exe", "rider64.exe", "datagrip64.exe",
    "phpstorm64.exe", "rubymine64.exe", "appcode64.exe", "datagrip64.exe"
)

$jetbrainsFound = $false
foreach ($exe in $ideExes) {
    $cmd = Get-Command $exe -ErrorAction SilentlyContinue
    if ($cmd) {
        $jetbrainsFound = $true
        Write-Host
        Write-Host "→ Installing WakaTime plugin in $exe..." -ForegroundColor Green
        & $cmd.Source installPlugins $jetbrainsPid
    }
}

if (-not $jetbrainsFound) {
    Write-Host
    Write-Host "No JetBrains IDEs found; skipping." -ForegroundColor DarkGray
}

Write-Host
Write-Host "Reading the config file..." -ForegroundColor White
if (-not (Test-Path -Path $configPath)) {
    Write-Host "Error: Config file not found at $configPath" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Successfully read config:" -ForegroundColor Green
Write-Host "API URL: $apiUrl" -ForegroundColor DarkGray
$maskedKey = if ($apiKey.Length -gt 8) { $apiKey.Substring(0, 8) + "..." } else { $apiKey }
Write-Host "API Key: $maskedKey" -ForegroundColor DarkGray

Write-Host
Write-Host "Sending test heartbeat..." -ForegroundColor White
$time = [Math]::Floor([decimal](Get-Date(Get-Date).ToUniversalTime()-uformat '%s'))
$heartbeat = @{
    type = 'file'
    time = $time
    entity = 'test.txt'
    language = 'Text'
}
$heartbeatJson = $heartbeat | ConvertTo-Json -Depth 5

try {
    $response = Invoke-WebRequest -Uri ("{0}/users/current/heartbeats" -f $apiUrl.TrimEnd('/')) `
        -Method Post `
        -Headers @{ Authorization = "Bearer $apiKey" } `
        -ContentType 'application/json' `
        -Body $heartbeatJson
    $statusCode = [int]$response.StatusCode
    $body = $response.Content
} catch {
    $statusCode = 0
    $body = $_.Exception.Message
    $resp = $_.Exception.Response
    if ($resp) {
        try { $statusCode = [int]$resp.StatusCode } catch {}
        try {
            $stream = $resp.GetResponseStream()
            if ($stream) {
                $reader = New-Object System.IO.StreamReader($stream)
                $body = $reader.ReadToEnd()
                $reader.Dispose()
                $stream.Dispose()
            }
        } catch {}
    }
}

if ($statusCode -eq 200 -or $statusCode -eq 202) {
    Write-Host "✓ Test heartbeat sent successfully!" -ForegroundColor Green
    Write-Host
    Write-Host "✓ Installation complete!" -ForegroundColor Green
    Write-Host "Please restart your editors/IDEs for changes to take effect."
} else {
    Write-Host "Error: Failed to send heartbeat: $body" -ForegroundColor Red
    exit 1
}
