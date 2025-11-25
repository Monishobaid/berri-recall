#!/bin/bash
# Bash hook for berri-recall
# Records every command you type automatically

__berri_hook() {
    local exit_code=$?
    local cmd="${__berri_last_cmd}"

    # Nothing to record? bail out
    [[ -z "$cmd" ]] && return 0

    # Don't record berri commands (that would be weird)
    [[ "$cmd" =~ ^berri ]] && return 0

    # Run this in background so it doesn't slow your terminal
    (
        berri-recall record \
            --command "$cmd" \
            --exit-code "$exit_code" \
            --cwd "$PWD" \
            &> /dev/null
    ) &
}

# Grab the command before it runs
__berri_preexec() {
    __berri_last_cmd="$BASH_COMMAND"
}

# Set everything up
if [[ -z "$__berri_installed" ]]; then
    export __berri_installed=1

    # Newer bash (4.4+) has better command capture
    if [[ ${BASH_VERSINFO[0]} -ge 4 ]] && [[ ${BASH_VERSINFO[1]} -ge 4 ]]; then
        trap '__berri_preexec' DEBUG
    else
        # Older bash needs to use history
        __berri_last_cmd=""
        PROMPT_COMMAND="__berri_last_cmd=\"\$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')\"${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
    fi

    # Hook into the prompt to record after each command
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="__berri_hook"
    elif [[ "$PROMPT_COMMAND" != *"__berri_hook"* ]]; then
        PROMPT_COMMAND="__berri_hook; $PROMPT_COMMAND"
    fi
fi
