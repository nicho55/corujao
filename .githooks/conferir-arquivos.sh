#!/usr/bin/env bash
# Recusa arquivo grande ou binário de jogo. Lê caminhos separados por NUL no stdin.
#   git diff --cached --name-only --diff-filter=AM -z | conferir-arquivos.sh --staged   (hook)
#   git ls-files -z | conferir-arquivos.sh                                              (CI)
# O .gitignore bloqueia por nome; isto bloqueia por tamanho e por tipo, para pegar o commit
# distraído. Jogos e binários NUNCA entram no repositório (LEIAME.md, "Por que o jogo não vem junto").
set -euo pipefail
LIMITE_MB=5
LIMITE=$((LIMITE_MB * 1024 * 1024))
PROIBIDOS='\.(so|so\.[0-9]+|dll|dylib|AppImage|appimage|iso|gcm|rvz|wbfs|ciso|z64|n64|v64|rom|bin|elf|dol|apk|exe|zip|7z|tar|tgz|xz|gz|bwstate|card|gci|sav|srm|state)$'
modo="${1:-}"
erros=0
while IFS= read -r -d '' arquivo; do
  if [[ "$arquivo" =~ $PROIBIDOS ]]; then
    echo "recusado (tipo proibido): $arquivo"
    erros=1
  fi
  if [[ "$modo" == "--staged" ]]; then
    tamanho=$(git cat-file -s ":$arquivo" 2>/dev/null || echo 0)
  else
    tamanho=$(stat -c %s "$arquivo" 2>/dev/null || echo 0)
  fi
  if (( tamanho > LIMITE )); then
    echo "recusado (maior que ${LIMITE_MB} MB): $arquivo ($((tamanho / 1024 / 1024)) MB)"
    erros=1
  fi
done
if (( erros )); then
  echo "Binário de jogo e arquivo grande não entram no repositório. Tire com: git rm --cached <arquivo>"
  exit 1
fi
