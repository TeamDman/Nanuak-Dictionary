cargo run
if ($?) {
    Write-Host "Successfully created new version"
    . ./AfterCreate.ps1
} else {
    Write-Host "Failed"
}