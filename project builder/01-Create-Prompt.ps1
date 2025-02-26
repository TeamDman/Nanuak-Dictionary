$x = Get-Content -Raw "./src/get_description.rs";
$prompt = "Implement all todo items and return the new content in full.`n`n$x";
$prompt | Set-Content "bruh.prompt";
