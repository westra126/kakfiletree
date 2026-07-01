declare-option -docstring "kakfiletree binary path" str kakfiletree_bin "%sh{echo ${kak_source%%/rc/*}/target/release/kakfiletree}"
declare-option -docstring "kakfiletree split width (tmux cols / herdr ratio * 100)" str kakfiletree_width "40"
declare-option -docstring "herdr pane id of the kakfiletree pane" str kakfiletree_pane_id ""

define-command kakfiletree -docstring "Open file tree in terminal multiplexer side split" %{
    nop %sh{
        if [ -n "$TMUX" ]; then
            tmux split-window -hb -l "$kak_opt_kakfiletree_width" \
                "$kak_opt_kakfiletree_bin" \
                --session "$kak_session" \
                --client "$kak_client" \
                --root "$PWD"
        elif [ "$HERDR_ENV" = "1" ]; then
            W="${kak_opt_kakfiletree_width:-40}"
            case "$W" in *[!0-9]*) W=40 ;; esac
            if [ "$W" -le 25 ]; then
                RATIO="0.2"
            elif [ "$W" -le 35 ]; then
                RATIO="0.3"
            elif [ "$W" -le 45 ]; then
                RATIO="0.4"
            elif [ "$W" -le 60 ]; then
                RATIO="0.5"
            else
                RATIO="0.5"
            fi
            NEW_PANE=$(herdr pane split "$kak_client_env_HERDR_PANE_ID" \
                --direction left --ratio "$RATIO" 2>/dev/null | \
                sed -n 's/.*"pane_id":"\([^"]*\)".*/\1/p')
            if [ -n "$NEW_PANE" ]; then
                echo "set-option global kakfiletree_pane_id '$NEW_PANE'" | kak -p "$kak_session"
                herdr pane run "$NEW_PANE" \
                    "$kak_opt_kakfiletree_bin --session $kak_session --client $kak_client --root $PWD"
            fi
        fi
    }
}

define-command kakfiletree-close -docstring "Close the kakfiletree multiplexer pane" %{
    nop %sh{
        if [ -n "$TMUX" ]; then
            tmux kill-pane -t "$kak_client_env_TMUX_PANE" 2>/dev/null
        elif [ "$HERDR_ENV" = "1" ]; then
            if [ -n "$kak_opt_kakfiletree_pane_id" ]; then
                herdr pane close "$kak_opt_kakfiletree_pane_id" 2>/dev/null && \
                    echo "set-option global kakfiletree_pane_id ''" | kak -p "$kak_session"
            fi
        fi
    }
}
