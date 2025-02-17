cargo run
if ($?) {
    Write-Host "Successfully created new version"
} else {
    Write-Host "Failed"
}
. ./AfterCreate.ps1