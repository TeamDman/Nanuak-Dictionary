try {
    Push-Location ..
    $dirs = Get-ChildItem -Directory `
    | Sort-Object -Property Name -Descending
    $latest = $dirs[0].Name
    Write-Host "Latest version: $latest"
    Push-Location $latest
    Push-Location .
} finally {
    Pop-Location
}

code -a .
code Cargo.toml
code src/main.rs
code src/lib.rs