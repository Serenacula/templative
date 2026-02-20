pub const VERSION: u32 = 3;

pub const SCRIPT: &str = r#"# templative-completions-version: 3

Register-ArgumentCompleter -Native -CommandName templative -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements
    $subcommands = @('init', 'add', 'change', 'remove', 'list', 'completions', 'update')

    $subcommand = $null
    foreach ($word in $words[1..($words.Count - 1)]) {
        if ($subcommands -contains $word.ToString()) {
            $subcommand = $word.ToString()
            break
        }
    }

    $prev = if ($words.Count -ge 2) { $words[$words.Count - 2].ToString() } else { '' }

    $completions = if ($null -eq $subcommand) {
        $subcommands + @('--color', '--no-color', '--version', '-v', '--help', '-h')
    } else {
        switch ($subcommand) {
            'init' {
                switch ($prev) {
                    '--git'        { @('fresh', 'preserve', 'no-git') }
                    '--write-mode' { @('strict', 'no-overwrite', 'skip-overwrite', 'overwrite', 'ask') }
                    'init'         { templative list --names-only 2>$null }
                    default        { @('--git', '--write-mode', '--help', '-h') }
                }
            }
            'add' {
                switch ($prev) {
                    '--git'        { @('fresh', 'preserve', 'no-git') }
                    '--write-mode' { @('strict', 'no-overwrite', 'skip-overwrite', 'overwrite', 'ask') }
                    default        { @('--name', '-n', '--description', '-d', '--git', '--git-ref', '--exclude', '--write-mode', '--help', '-h') }
                }
            }
            'change' {
                switch ($prev) {
                    '--git'        { @('fresh', 'preserve', 'no-git', 'unset') }
                    '--write-mode' { @('strict', 'no-overwrite', 'skip-overwrite', 'overwrite', 'ask', 'unset') }
                    'change'       { templative list --names-only 2>$null }
                    default        { @('--name', '--description', '--unset-description', '--location', '--git', '--pre-init', '--unset-pre-init', '--post-init', '--unset-post-init', '--git-ref', '--unset-git-ref', '--exclude', '--clear-exclude', '--write-mode', '--help', '-h') }
                }
            }
            'remove' {
                if ($prev -eq 'remove') { templative list --names-only 2>$null }
                else { @() }
            }
            'list' {
                @('--names-only', '--color', '--no-color', '--help', '-h')
            }
            'completions' {
                switch ($prev) {
                    '--check'      { @() }
                    'completions'  { @('zsh', 'bash', 'fish', 'powershell') }
                    default        { @('zsh', 'bash', 'fish', 'powershell', '--check', '--help', '-h') }
                }
            }
            'update' {
                switch ($prev) {
                    'update'  { templative list --names-only 2>$null }
                    default   { @('--check', '--help', '-h') }
                }
            }
        }
    }

    $completions | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#;
