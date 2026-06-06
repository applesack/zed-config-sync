$url = 'https://github.com/applesack/zed-config-sync/releases/latest/download/zed-config-x86_64-pc-windows-msvc.exe'

$wc = [System.Net.WebClient]::new()
$bytes = $wc.DownloadData($url)
$wc.Dispose()

$hashBytes = [System.Security.Cryptography.SHA256]::Create().ComputeHash($bytes)
$hash = -join ($hashBytes | ForEach-Object { $_.ToString('X2') })

Write-Host $hash
