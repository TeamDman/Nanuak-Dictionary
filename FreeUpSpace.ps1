$target_dirs = Get-ChildItem -Recurse -Directory -Filter "target"
foreach ($dir in $target_dirs) {
    try {
        $parent = $dir.Parent.FullName
        Push-Location $parent
        cargo clean
    } finally {
        Pop-Location
    }
}