#!/usr/bin/env bash
# Porta unica de qualidade. Toda fatia fecha com isto verde antes da proxima comecar.
set -euo pipefail
cd "$(dirname "$0")/.."

etapa() { printf '\n\033[1m>>> %s\033[0m\n' "$1"; }

etapa "formatacao"
cargo fmt --all --check

etapa "clippy (warnings sao erro)"
cargo clippy --workspace --all-targets -- -D warnings

etapa "testes"
cargo test --workspace

etapa "nenhum unsafe em codigo nosso"
if grep -rn --include='*.rs' -E '(^|[^_[:alnum:]])unsafe([^_[:alnum:]]|$)' crates/ src-tauri/src/ 2>/dev/null; then
  echo "ERRO: 'unsafe' encontrado. Este projeto nao tem excecao." >&2
  exit 1
fi

etapa "tipos do frontend"
pnpm exec tsc --noEmit

etapa "build do frontend"
pnpm build

etapa "contrato Rust<->TS em dia"
if [ -f src/bindings.ts ]; then
  git diff --exit-code -- src/bindings.ts \
    || { echo "ERRO: bindings.ts fora de sincronia. Rode a geracao e commite." >&2; exit 1; }
else
  echo "(bindings.ts ainda nao existe — chega na fatia 1)"
fi

printf '\n\033[1;32mCI verde\033[0m\n'
