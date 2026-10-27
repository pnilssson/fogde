codex() {
  local arg
  for arg in "$@"; do
    case "$arg" in
      agents | queue | remote-control | --remote | --remote=* | --no-daemon)
        command codex "$@"
        return
        ;;
    esac
  done
  command codex --no-daemon "$@"
}
