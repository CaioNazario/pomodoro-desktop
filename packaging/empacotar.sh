#!/usr/bin/env bash
# Gera dist-instalador/ com os pacotes e o instalador.
#
# O build roda dentro de um container Ubuntu 22.04 de proposito: o glibc da
# maquina de build vira o piso de compatibilidade do binario. Compilando no
# Arch (glibc 2.44) o pacote nao roda em Ubuntu 22.04 nem Debian 12.
set -euo pipefail

RAIZ="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
IMAGEM=pomodoro-empacotador
VOLUME=pomodoro-empacotador-cache

passo() { printf '\n\033[1m>>> %s\033[0m\n' "$*"; }

command -v docker >/dev/null 2>&1 || {
  echo "docker nao encontrado. Ele e obrigatorio: sem container o pacote so roda em distro tao nova quanto esta." >&2
  exit 1
}

passo "imagem de build"
docker build -t "$IMAGEM" -f "$RAIZ/packaging/Dockerfile.build" "$RAIZ/packaging"

passo "build dos pacotes"
mkdir -p "$RAIZ/dist-instalador"
docker volume create "$VOLUME" >/dev/null
docker run --rm \
  -v "$RAIZ:/src:ro" \
  -v "$RAIZ/dist-instalador:/out" \
  -v "$VOLUME:/build" \
  -e HOST_UID="$(id -u)" \
  -e HOST_GID="$(id -g)" \
  "$IMAGEM"
