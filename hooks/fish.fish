# Fish hook for berri-recall
# Records commands automatically

function __berri_postexec --on-event fish_postexec
    set -l exit_code $status
    set -l cmd $argv[1]

    # Skip if no command
    if test -z "$cmd"
        return 0
    end

    # Don't record berri commands
    if string match -q "berri*" -- $cmd
        return 0
    end

    # Background job so it doesn't block
    fish -c "berri-recall record \
        --command '$cmd' \
        --exit-code $exit_code \
        --cwd '$PWD' \
        &> /dev/null" &
end

set -g __berri_installed 1
