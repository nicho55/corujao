#!/usr/bin/env bash
# Abre o launcher. Compila na primeira vez (ou com --compilar); depois só abre o binário pronto.
#   abrir.sh              abre
#   abrir.sh --compilar   recompila (interface Flutter + núcleo Rust) e abre
set -euo pipefail
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$AQUI/interface/build/linux/x64/release/bundle/pog"
if [ ! -x "$BIN" ] || [ "${1:-}" = "--compilar" ]; then
  [ "${1:-}" = "--compilar" ] && shift
  ( cd "$AQUI/interface" && flutter build linux --release )
fi
exec setsid "$BIN" "$@" >/dev/null 2>&1 </dev/null
