if set -q FOGDE_XDG_DATA_DIRS
    set -gx XDG_DATA_DIRS $FOGDE_XDG_DATA_DIRS
    set -e FOGDE_XDG_DATA_DIRS
else
    set -e XDG_DATA_DIRS
end

function codex
    if string match -qr -- '^(agents|queue|remote-control|--remote|--remote=.*|--no-daemon)$' $argv
        command codex $argv
    else
        command codex --no-daemon $argv
    end
end
