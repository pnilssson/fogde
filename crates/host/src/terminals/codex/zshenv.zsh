if [[ -n "${FOGDE_ZDOTDIR+set}" ]]; then
  builtin export ZDOTDIR="$FOGDE_ZDOTDIR"
  builtin unset FOGDE_ZDOTDIR
else
  builtin unset ZDOTDIR
fi

builtin source "${${(%):-%x}:A:h}/fogde-codex.sh"

if [[ -r "${ZDOTDIR-$HOME}/.zshenv" ]]; then
  builtin source "${ZDOTDIR-$HOME}/.zshenv"
fi
