# PowerShell hook for berri-recall
# Records commands automatically

if (-not $global:__berri_installed) {
    $global:__berri_installed = $true
    $global:__berri_last_cmd = ""

    $ExecutionContext.InvokeCommand.PreCommandLookupAction = {
        param($CommandName, $CommandLookupEventArgs)
        $global:__berri_last_cmd = $CommandName
    }

    function global:__berri_hook {
        $exit_code = $LASTEXITCODE
        if ($null -eq $exit_code) { $exit_code = 0 }

        $cmd = (Get-History -Count 1).CommandLine

        if ([string]::IsNullOrWhiteSpace($cmd)) {
            return
        }

        # Don't record berri commands
        if ($cmd -match "^berri") {
            return
        }

        # Background job so it doesn't block
        Start-Job -ScriptBlock {
            param($command, $exitCode, $workingDir)
            & berri-recall record `
                --command $command `
                --exit-code $exitCode `
                --cwd $workingDir `
                2>&1 | Out-Null
        } -ArgumentList $cmd, $exit_code, $PWD | Out-Null
    }

    $originalPrompt = $function:prompt
    function global:prompt {
        __berri_hook
        & $originalPrompt
    }

    Write-Host "berri-recall hook installed" -ForegroundColor Green
}
