# Pomodoro com descanso ativo

Um Pomodoro para **Linux/GNOME** em que a pausa também faz parte do fluxo de trabalho.

Em vez de simplesmente iniciar um intervalo de 5 minutos e deixar o usuário cair no scroll infinito, cada pausa pode ter uma **atividade previamente definida** — como assistir a um vídeo, ouvir uma música ou abrir qualquer conteúdo através de uma URL.

Durante a pausa, o conteúdo é aberto automaticamente em uma janela dedicada e o timer continua correndo.

> **A ideia:** descanso não precisa ser tempo perdido. Ele pode ser intencional.

---

## O problema

O Pomodoro tradicional se preocupa bastante com os períodos de foco, mas trata a pausa como um intervalo vazio.

Na prática, esses minutos podem facilmente virar:

* rolagem infinita;
* redes sociais;
* vídeos aleatórios;
* perda de noção do tempo;
* dificuldade para voltar ao trabalho.

Este projeto parte de uma hipótese simples:

> **Se a pausa for definida antes de começar e tiver algum atrito para ser abandonada, ela pode continuar sendo uma pausa sem virar distração.**

A partir disso, o aplicativo foi projetado em torno de três ideias:

### 1. A pausa tem uma atividade

Antes de iniciar o ciclo, o usuário pode definir uma atividade para a pausa:

**Nome + URL**

Exemplos:

* assistir a um vídeo;
* ouvir uma música;
* acessar um artigo;
* abrir qualquer página específica.

A atividade é carregada em um **WebView isolado e em tela cheia**.

Não existe player, biblioteca de músicas ou sistema próprio de mídia. Para o aplicativo, música, vídeo e artigo são simplesmente URLs.

URLs do YouTube são tratadas de forma específica para abrir diretamente o **player**, em vez da página completa.

### 2. Sair da pausa exige uma ação consciente

Durante uma pausa ativa, o aplicativo dificulta a saída propositalmente.

Enquanto o bloqueio está ativo:

* fechar a janela é recusado;
* os atalhos próprios do aplicativo são desativados;
* atalhos selecionados do GNOME são temporariamente suprimidos;
* a única saída antecipada é através da **Urgência**.

A Urgência fica escondida no topo da janela e aparece apenas quando o mouse passa sobre a área.

Ao acioná-la, o usuário precisa confirmar a saída.

Uma pausa encerrada dessa maneira é registrada como **pausa interrompida**.

### 3. O mouse nunca é capturado

O aplicativo não captura o ponteiro do mouse.

Isso é uma decisão de projeto, não uma limitação.

A saída de emergência depende justamente da possibilidade de mover o ponteiro até o botão de Urgência.

---

## Timer baseado em prazo absoluto

O timer não depende de um acumulador de `ticks`.

Cada etapa possui um **prazo absoluto**.

Isso permite lidar corretamente com situações como suspensão da máquina.

Por exemplo:

```text
Foco: 25 minutos
↓
Computador suspende após 10 minutos
↓
Computador acorda depois de 20 minutos
↓
O prazo original já passou
↓
O aplicativo avança uma etapa
↓
A sessão recebe apenas os 10 minutos efetivamente trabalhados
```

Dessa forma, suspensão, atrasos e perda de foco do processo não corrompem o estado do timer.

---

## Persistência

O projeto separa configuração e histórico.

### Configuração

A configuração do aplicativo fica em um arquivo **TOML versionado**.

A escrita é feita de forma atômica para evitar corrupção.

Se o arquivo estiver:

* inválido;
* corrompido;
* em uma versão desconhecida;

o aplicativo:

1. inicia usando a configuração padrão;
2. preserva o arquivo inválido;
3. move o arquivo para a extensão `.corrompido`.

### Histórico

O histórico diário de sessões é armazenado separadamente em **SQLite**.

A configuração não depende do banco de dados.

---

## Screenshots

<p align="center">
  <img src="docs/screenshots/01-timer.png" width="720" alt="Janela principal do Pomodoro com anel de progresso, relógio e controles do timer">
</p>

<p align="center">
  <img src="docs/screenshots/02-menu.png" width="720" alt="Menu lateral com sessões do ciclo, modo Global ou Individual, durações, auto-start e atividade da pausa">
</p>

<p align="center">
  <img src="docs/screenshots/03-widget.png" width="280" alt="Widget flutuante sempre no topo mostrando etapa, relógio e progresso">
</p>

---

## Stack

| Camada                   | Tecnologia                                     |
| ------------------------ | ---------------------------------------------- |
| Núcleo / domínio         | Rust 2021                                      |
| Desktop                  | Tauri 2 + WebKitGTK                            |
| Frontend                 | React 19 + TypeScript + Vite                   |
| Contrato Frontend ↔ Rust | `bindings.ts` gerado a partir do contrato Rust |
| Configuração             | TOML versionado + escrita atômica              |
| Histórico                | SQLite                                         |
| Integração GNOME         | `gsettings`                                    |
| CI                       | GitHub Actions + `scripts/ci.sh`               |

O núcleo de domínio é mantido independente do Tauri, filesystem e relógio do sistema sempre que possível. Isso permite testar a lógica do aplicativo sem depender do ambiente gráfico.

---

## Plataforma

O projeto é desenvolvido principalmente para:

* **Linux**
* **GNOME**
* **Wayland + XWayland**

O aplicativo força `GDK_BACKEND=x11` internamente.

Isso é necessário porque algumas funcionalidades utilizadas pelo widget — como posicionamento preciso da janela, always-on-top e gerenciamento de foco — dependem do backend X11.

### Fora do GNOME

O aplicativo pode executar em outros ambientes Linux, mas algumas funcionalidades de bloqueio são degradadas.

Nesse caso, o bloqueio funciona como:

```text
Tela cheia
+
Fechamento recusado
```

A supressão de atalhos do desktop depende especificamente da integração com o GNOME.

---

## Requisitos

| Dependência   | Versão                |
| ------------- | --------------------- |
| Linux + GNOME | GNOME 50+ recomendado |
| Rust          | Stable                |
| Node.js       | 22+                   |
| pnpm          | 9+                    |

### Dependências de sistema

#### Arch Linux

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  libappindicator-gtk3 \
  librsvg \
  xdotool
```

#### Debian / Ubuntu

```bash
sudo apt install \
  libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libxdo-dev \
  patchelf \
  build-essential
```

---

## Desenvolvimento

Clone o projeto:

```bash
git clone git@github.com:CaioNazario/pomodoro-desktop.git
cd pomodoro-desktop/pomodoro
```

Instale as dependências:

```bash
pnpm install
```

Execute em modo desenvolvimento:

```bash
pnpm tauri dev
```

O modo de desenvolvimento possui hot reload para o frontend.

---

## Build

Para gerar os pacotes:

```bash
pnpm tauri build
```

Os artefatos serão gerados em:

```text
pomodoro/target/release/bundle/
```

Entre os formatos disponíveis:

| Formato  | Localização               | Uso                 |
| -------- | ------------------------- | ------------------- |
| Binário  | `target/release/pomodoro` | Execução direta     |
| AppImage | `bundle/appimage/`        | Distribuições Linux |
| `.deb`   | `bundle/deb/`             | Debian / Ubuntu     |
| `.rpm`   | `bundle/rpm/`             | Fedora / RHEL       |

---

## Instalação

O projeto ainda não possui releases publicadas.

A instalação atualmente é feita a partir do código-fonte.

O script:

```bash
packaging/instalar.sh
```

automatiza a instalação e tenta utilizar o pacote nativo da distribuição quando disponível.

Caso contrário, utiliza o AppImage instalado no `$HOME`, sem necessidade de root.

Para remover:

```bash
packaging/desinstalar.sh
```

---

## AppImage

Em sistemas sem `fuse2`, execute o AppImage com:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./pomodoro_*.AppImage
```

Em algumas versões rolling do Arch, a geração do AppImage pode falhar durante o `linuxdeploy` por incompatibilidade do `strip` com determinados segmentos ELF.

Nesse caso, o **binário direto**, `.deb` e `.rpm` continuam disponíveis.

---

## Arquitetura

O projeto separa a aplicação em três grandes partes:

```text
┌─────────────────────────────┐
│          React UI           │
│       React + TypeScript    │
└──────────────┬──────────────┘
               │
               │ Tauri Commands
               ▼
┌─────────────────────────────┐
│         Tauri Shell         │
│       Windows / WebView     │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│       Rust Core / Domain    │
│                             │
│ Timer · Sessions · Config   │
│ Persistence · State         │
└──────────────┬──────────────┘
               │
       ┌───────┴────────┐
       ▼                ▼
    SQLite             TOML
   Histórico         Configuração
```

A lógica de negócio fica no Rust e não depende diretamente da interface gráfica.

Isso facilita testes, manutenção e evolução do aplicativo.

---

## CI

O projeto utiliza GitHub Actions para validação do backend e frontend.

Localmente, a porta de entrada para as verificações é:

```bash
scripts/ci.sh
```

A ideia é manter o mesmo conjunto básico de verificações disponível tanto no ambiente local quanto no CI.

---

## Licença

MIT © Caio Nazário
