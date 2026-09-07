#!/usr/bin/env bash
# Instalador do Pomodoro. Escolhe o formato nativo da distro quando existe e
# cai no AppImage (instalacao no HOME, sem root) quando nao existe.
set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")"

APP_NOME="Pomodoro"
BIN_NOME="pomodoro"
APP_ID="com.caionazario.pomodoro"

vermelho() { printf '\033[1;31m%s\033[0m\n' "$*" >&2; }
verde()    { printf '\033[1;32m%s\033[0m\n' "$*"; }
passo()    { printf '\n\033[1m>>> %s\033[0m\n' "$*"; }

# Mantem a janela aberta quando o script foi aberto com duplo-clique, senao o
# terminal fecha antes de a pessoa ler o resultado.
segurar_janela() {
  if [ -t 0 ] && [ "${POMODORO_INSTALADOR_INTERATIVO:-}" = "1" ]; then
    printf '\nPressione Enter para fechar.'
    read -r _
  fi
}
trap segurar_janela EXIT

# --- privilegio ------------------------------------------------------------
# Sem terminal (duplo-clique, launcher grafico) o sudo nao tem como pedir senha,
# entao o pkexec vem primeiro: ele abre o dialogo do polkit.
COMO_ROOT=()
if [ "$(id -u)" -ne 0 ]; then
  if [ ! -t 0 ] && command -v pkexec >/dev/null 2>&1; then
    COMO_ROOT=(pkexec)
  elif command -v sudo >/dev/null 2>&1; then
    COMO_ROOT=(sudo)
  elif command -v pkexec >/dev/null 2>&1; then
    COMO_ROOT=(pkexec)
  fi
fi

tem_root() { [ "$(id -u)" -eq 0 ] || [ ${#COMO_ROOT[@]} -gt 0 ]; }

# --- localizacao dos pacotes ----------------------------------------------
achar() { find . -maxdepth 2 -name "$1" -print -quit | sed 's|^\./||'; }

DEB=$(achar '*.deb' || true)
RPM=$(achar '*.rpm' || true)
APPIMAGE=$(achar '*.AppImage' || true)

# --- caminhos de instalacao -----------------------------------------------
# Uma instalacao de sistema torna a copia no HOME obsoleta; deixar as duas
# faz aparecer "Pomodoro" duas vezes no menu.
limpar_instalacao_no_home() {
  local appdir="$HOME/.local/share/applications"
  rm -f "$HOME/.local/bin/$BIN_NOME" \
        "$appdir/$APP_ID.desktop" \
        "$appdir/pomodoro.desktop"
  for tam in 32x32 64x64 128x128 256x256 512x512; do
    rm -f "$HOME/.local/share/icons/hicolor/$tam/apps/$BIN_NOME.png"
  done
  rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/$BIN_NOME.svg"
  command -v update-desktop-database >/dev/null 2>&1 \
    && update-desktop-database "$appdir" >/dev/null 2>&1 || true
  # Sem regenerar o cache, o hicolor do HOME continua anunciando os PNGs que
  # acabaram de ser apagados. Como ~/.local/share/icons tem precedencia sobre
  # /usr/share/icons, o tema resolve o icone para um arquivo inexistente e o
  # app aparece sem logo mesmo com o pacote instalado corretamente.
  command -v gtk-update-icon-cache >/dev/null 2>&1 \
    && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true
}

instalar_deb() {
  limpar_instalacao_no_home
  passo "Instalando via .deb ($(basename "$DEB"))"
  if command -v apt-get >/dev/null 2>&1; then
    "${COMO_ROOT[@]}" apt-get install -y "./$DEB"
  else
    "${COMO_ROOT[@]}" dpkg -i "$DEB" || "${COMO_ROOT[@]}" apt-get -f install -y
  fi
}

instalar_rpm() {
  limpar_instalacao_no_home
  passo "Instalando via .rpm ($(basename "$RPM"))"
  if command -v dnf >/dev/null 2>&1; then
    "${COMO_ROOT[@]}" dnf install -y "$RPM"
  elif command -v zypper >/dev/null 2>&1; then
    "${COMO_ROOT[@]}" zypper --non-interactive install --allow-unsigned-rpm "$RPM"
  else
    "${COMO_ROOT[@]}" rpm -Uvh --force "$RPM"
  fi
}

instalar_pacman() {
  tem_root || { vermelho "Precisa de sudo ou pkexec para instalar."; exit 1; }
  limpar_instalacao_no_home
  passo "Instalando via pacman (PKGBUILD)"
  local tmp pacote
  tmp=$(mktemp -d)
  cp PKGBUILD "$tmp/"
  cp "$DEB" "$tmp/"
  # makepkg so empacota; quem escala privilegio e o pacman -U logo abaixo. Com
  # "makepkg -si" o sudo seria chamado por dentro do makepkg, fora do COMO_ROOT,
  # e o pkexec nunca entraria em jogo. O pacman -U resolve as dependencias dos
  # repositorios sozinho.
  ( cd "$tmp" && makepkg -f --noconfirm )
  pacote=$(find "$tmp" -maxdepth 1 -name '*.pkg.tar.*' -print -quit)
  [ -n "$pacote" ] || { vermelho "makepkg nao gerou pacote."; rm -rf "$tmp"; exit 1; }
  "${COMO_ROOT[@]}" pacman -U --noconfirm "$pacote"
  rm -rf "$tmp"
}

instalar_appimage() {
  passo "Instalando o AppImage no seu HOME (sem root)"
  local bindir="$HOME/.local/bin"
  local appdir="$HOME/.local/share/applications"
  local icondir="$HOME/.local/share/icons/hicolor"

  mkdir -p "$bindir" "$appdir"
  # rm antes do install: sobrescrever binario em execucao da "Text file busy".
  rm -f "$bindir/$BIN_NOME"
  install -m 755 "$APPIMAGE" "$bindir/$BIN_NOME"

  for png in icones/*.png; do
    [ -e "$png" ] || continue
    local tam
    tam=$(basename "$png" .png)
    mkdir -p "$icondir/$tam/apps"
    install -m 644 "$png" "$icondir/$tam/apps/$BIN_NOME.png"
  done
  if [ -e icones/icon.svg ]; then
    mkdir -p "$icondir/scalable/apps"
    install -m 644 icones/icon.svg "$icondir/scalable/apps/$BIN_NOME.svg"
  fi

  cat > "$appdir/$APP_ID.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Version=1.0
Name=$APP_NOME
GenericName=Timer Pomodoro
Comment=Timer pomodoro com tomada de foco
Exec=$bindir/$BIN_NOME
Icon=$BIN_NOME
Terminal=false
Categories=Utility;Clock;
Keywords=pomodoro;foco;timer;cronometro;produtividade;concentracao;estudo;focus;
StartupNotify=true
StartupWMClass=$BIN_NOME
DESKTOP
  chmod 644 "$appdir/$APP_ID.desktop"

  # Entrada antiga escrita a mao antes deste instalador existir.
  rm -f "$appdir/pomodoro.desktop"

  command -v update-desktop-database >/dev/null 2>&1 \
    && update-desktop-database "$appdir" >/dev/null 2>&1 || true
  command -v gtk-update-icon-cache >/dev/null 2>&1 \
    && gtk-update-icon-cache -f -t "$icondir" >/dev/null 2>&1 || true

  case ":$PATH:" in
    *":$bindir:"*) ;;
    *) printf '\nAviso: %s nao esta no PATH. O atalho grafico funciona mesmo assim.\n' "$bindir" ;;
  esac
}

# --- decisao ---------------------------------------------------------------
METODO=auto
case "${1:-}" in
  --appimage|--deb|--rpm|--pacman) METODO="${1#--}" ;;
  --ajuda|-h|--help)
    cat <<AJUDA
Uso: ./instalar.sh [opcao]

  (sem opcao)   escolhe o formato nativo da sua distro
  --appimage    instala o AppImage no seu HOME, sem root
  --deb         forca o .deb
  --rpm         forca o .rpm
  --pacman      forca o PKGBUILD (Arch)
  --ajuda       mostra isto
AJUDA
    exit 0 ;;
  "") ;;
  *) vermelho "Opcao desconhecida: $1 (veja ./instalar.sh --ajuda)"; exit 2 ;;
esac

passo "Detectando a distribuicao"
if [ -r /etc/os-release ]; then
  echo "Sistema: $(. /etc/os-release; echo "${PRETTY_NAME:-desconhecido}")"
fi
echo "Metodo: $METODO"

exigir() {
  [ -n "$2" ] || { vermelho "Nenhum $1 encontrado nesta pasta."; exit 1; }
}

case "$METODO" in
  appimage) exigir AppImage "$APPIMAGE"; instalar_appimage ;;
  deb)      exigir .deb "$DEB"; tem_root || { vermelho "Precisa de sudo."; exit 1; }; instalar_deb ;;
  rpm)      exigir .rpm "$RPM"; tem_root || { vermelho "Precisa de sudo."; exit 1; }; instalar_rpm ;;
  pacman)   exigir .deb "$DEB"; instalar_pacman ;;
  auto)
    if command -v pacman >/dev/null 2>&1 && command -v makepkg >/dev/null 2>&1 \
       && [ -f PKGBUILD ] && [ -n "$DEB" ] && [ "$(id -u)" -ne 0 ]; then
      instalar_pacman
    elif command -v apt-get >/dev/null 2>&1 || command -v dpkg >/dev/null 2>&1; then
      exigir .deb "$DEB"; tem_root || { vermelho "Precisa de sudo para instalar o .deb."; exit 1; }
      instalar_deb
    elif command -v dnf >/dev/null 2>&1 || command -v zypper >/dev/null 2>&1 || command -v rpm >/dev/null 2>&1; then
      exigir .rpm "$RPM"; tem_root || { vermelho "Precisa de sudo para instalar o .rpm."; exit 1; }
      instalar_rpm
    elif [ -n "$APPIMAGE" ]; then
      instalar_appimage
    else
      vermelho "Nao achei nenhum pacote instalavel nesta pasta."
      exit 1
    fi ;;
esac

verde ""
verde "$APP_NOME instalado."
echo "Procure por \"Pomodoro\" na busca de aplicativos. Se nao aparecer na hora,"
echo "faca logout/login (alguns ambientes so releem o menu no proximo login)."
