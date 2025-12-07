#!/bin/zsh
# Zsh hook for berri-recall
# Records commands automatically

# Called right before a command runs
__berri_preexec() {
    typeset -g __berri_last_cmd="$1"
}

# Called right after a command finishes
__berri_precmd() {
    local exit_code=$?

    [[ -z "$__berri_last_cmd" ]] && return 0

    # Don't record berri commands
    [[ "$__berri_last_cmd" =~ ^berri ]] && return 0

    # Run silently in background without job notification
    {
        berri-recall record \
            --command "$__berri_last_cmd" \
            --exit-code "$exit_code" \
            --cwd "$PWD" \
            &> /dev/null
    } &!

    __berri_last_cmd=""
}

# Install everything (zsh has native hook support which is nice)
if [[ -z "$__berri_installed" ]]; then
    typeset -g __berri_installed=1

    # preexec runs before commands
    if [[ -z "${preexec_functions[(r)__berri_preexec]}" ]]; then
        preexec_functions+=(__berri_preexec)
    fi

    # precmd runs after commands
    if [[ -z "${precmd_functions[(r)__berri_precmd]}" ]]; then
        precmd_functions+=(__berri_precmd)
    fi
fi
