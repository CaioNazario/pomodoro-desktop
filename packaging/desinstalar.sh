#!/usr/bin/env bash
# Remove o Pomodoro instalado por qualquer um dos caminhos do instalar.sh.
set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")"

APP_ID="com.caionazario.pomodoro"
BIN_NOME="pomodoro"

passo() { printf '\n\033[1m>>> %s\033[0m\n' "$*"; }
verde() { printf '\033[1;32m%s\033[0m\n' "$*"; }

segurar_janela() {
  if [ -t 0 ] && [ "${POMODORO_INSTALADOR_INTERATIVO:-}" = "1" ]; then
    printf '\nPressione Enter para fechar.'
    read -r _
  fi
}
trap segurar_janela EXIT

COMO_ROOT=()
if [ "$(id -u)" -ne 0 ]; then
  if command -v sudo >/dev/null 2>&1; then
    COMO_ROOT=(sudo)
  elif command -v pkexec >/dev/null 2>&1; then
    COMO_ROOT=(pkexec)
  fi
fi

removeu=0

passo "Removendo pacote do sistema (se houver)"
if command -v pacman >/dev/null 2>&1 && pacman -Qq pomodoro-desktop-bin >/dev/null 2>&1; then
  "${COMO_ROOT[@]}" pacman -Rns --noconfirm pomodoro-desktop-bin && removeu=1
elif command -v dpkg >/dev/null 2>&1 && dpkg -s pomodoro >/dev/null 2>&1; then
  "${COMO_ROOT[@]}" apt-get remove -y pomodoro && removeu=1
elif command -v rpm >/dev/null 2>&1 && rpm -q pomodoro >/dev/null 2>&1; then
  "${COMO_ROOT[@]}" rpm -e pomodoro && removeu=1
else
  echo "Nenhum pacote de sistema encontrado."
fi

passo "Removendo instalacao no HOME (se houver)"
alvos=(
  "$HOME/.local/bin/$BIN_NOME"
  "$HOME/.local/share/applications/$APP_ID.desktop"
  "$HOME/.local/share/applications/pomodoro.desktop"
  "$HOME/.local/share/icons/hicolor/scalable/apps/$BIN_NOME.svg"
)
for tam in 32x32 64x64 128x128 256x256 512x512; do
  alvos+=("$HOME/.local/share/icons/hicolor/$tam/apps/$BIN_NOME.png")
done
for alvo in "${alvos[@]}"; do
  if [ -e "$alvo" ]; then
    rm -f "$alvo"
    echo "removido: $alvo"
    removeu=1
  fi
done

command -v update-desktop-database >/dev/null 2>&1 \
  && update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 \
  && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

passo "Dados do usuario"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/$APP_ID"
if [ -d "$CONFIG" ]; then
  echo "Preservado: $CONFIG"
  echo "Apague a mao se quiser zerar plano e historico."
fi

verde ""
if [ "$removeu" -eq 1 ]; then
  verde "Pomodoro removido."
else
  verde "Nada a remover."
fi
