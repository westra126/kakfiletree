declare-option -docstring "kakfiletree binary path" str kakfiletree_bin "%sh{echo ${kak_source%%/rc/*}/target/release/kakfiletree}"
declare-option -docstring "kakfiletree split width" str kakfiletree_width "40"

define-command kakfiletree -docstring "Open file tree in tmux left split" %{
    nop %sh{
        if [ -n "$TMUX" ]; then
            tmux split-window -hb -l "$kak_opt_kakfiletree_width" \
                "$kak_opt_kakfiletree_bin" \
                --session "$kak_session" \
                --client "$kak_client" \
                --root "$PWD"
        fi
    }
}

define-command kakfiletree-close -docstring "Close the kakfiletree tmux pane" %{
    nop %sh{
        if [ -n "$TMUX" ]; then
            tmux kill-pane -t "$kak_client_env_TMUX_PANE" 2>/dev/null
        fi
    }
}
