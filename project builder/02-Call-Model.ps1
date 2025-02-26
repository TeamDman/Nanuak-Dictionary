$prompt = Get-Content -Raw "bruh.prompt"
$model = "deepseek-r1:32b"
ollama run $model $prompt --nowordwrap | Tee-Object "response.md"