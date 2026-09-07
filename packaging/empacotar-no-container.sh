#!/usr/bin/env bash
# Roda dentro do container.
#   /src   repositorio do host, somente leitura
#   /out   pasta de saida no host
#   /build volume de cache (target do cargo, store do pnpm)
#
# O codigo e copiado para /work antes de compilar: buildar direto no bind-mount
# faria o pnpm do container brigar com o node_modules do host.
set -euo pipefail

export CI=true
export CARGO_TARGET_DIR=/build/target
export APPIMAGE_EXTRACT_AND_RUN=1

passo() { printf '\n\033[1m>>> %s\033[0m\n' "$*"; }

passo "copiando o codigo para /work"
rm -rf /work && mkdir -p /work
tar -C /src \
    --exclude=./node_modules --exclude=./pomodoro/node_modules \
    --exclude=./target --exclude=./pomodoro/target \
    --exclude=./pomodoro/src-tauri/target \
    --exclude=./dist-instalador --exclude=./.git \
    -cf - . | tar -C /work -xf -

APP=/work/pomodoro
SAIDA=/out

passo "dependencias do frontend"
cd "$APP"
pnpm install --frozen-lockfile --store-dir /build/pnpm-store

passo "build dos pacotes (deb, rpm, appimage)"
pnpm exec tauri build

BUNDLE="$CARGO_TARGET_DIR/release/bundle"

passo "montando a pasta de instalacao"
find "$SAIDA" -mindepth 1 -delete
mkdir -p "$SAIDA/icones"

cp "$BUNDLE"/deb/*.deb           "$SAIDA/"
cp "$BUNDLE"/rpm/*.rpm           "$SAIDA/"
cp "$BUNDLE"/appimage/*.AppImage "$SAIDA/"
chmod +x "$SAIDA"/*.AppImage

for tam in 32 64 128 256 512; do
  cp "$APP/src-tauri/icons/${tam}x${tam}.png" "$SAIDA/icones/${tam}x${tam}.png"
done
cp "$APP/src-tauri/icons/icon.svg" "$SAIDA/icones/icon.svg"

cp /src/packaging/instalar.sh    "$SAIDA/"
cp /src/packaging/desinstalar.sh "$SAIDA/"
cp /src/packaging/PKGBUILD       "$SAIDA/"
cp /src/packaging/LEIA-ME.txt    "$SAIDA/"
cp "/src/packaging/Instalar Pomodoro.desktop" "$SAIDA/"
cp /src/LICENSE                  "$SAIDA/"
chmod +x "$SAIDA/instalar.sh" "$SAIDA/desinstalar.sh" "$SAIDA/Instalar Pomodoro.desktop"

# O PKGBUILD referencia o .deb pelo nome e valida por sha256.
DEB=$(basename "$SAIDA"/*.deb)
SOMA=$(sha256sum "$SAIDA/$DEB" | cut -d' ' -f1)
sed -i "s|@DEB@|$DEB|g; s|@SHA256@|$SOMA|g" "$SAIDA/PKGBUILD"

passo "glibc maximo exigido pelo binario"
objdump -T "$CARGO_TARGET_DIR/release/pomodoro" \
  | grep -oP 'GLIBC_\d+\.\d+' | sort -uV | tail -1

if [ -n "${HOST_UID:-}" ] && [ -n "${HOST_GID:-}" ]; then
  chown -R "$HOST_UID:$HOST_GID" "$SAIDA"
fi

printf '\n\033[1;32mPacotes prontos em dist-instalador/\033[0m\n'
ls -lh "$SAIDA"
