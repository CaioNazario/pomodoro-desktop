# PRD — Pomodoro com descanso ativo

**Versão:** 0.2 (todas as pendências fechadas) · **Data:** 2026-08-22
**Alvo:** Linux desktop, GNOME 50 / sessão Wayland, **app rodando sob XWayland** (`GDK_BACKEND=x11`)
**Stack:** Rust + Tauri 2 + React 19

## Como ler este documento

Extraído por engenharia reversa de `pomodoro/src/Pomodoro Desktop.dc.html`, que era o único
artefato de comportamento existente, e fechado com as decisões do usuário em 2026-08-22.

- **[MOCKUP]** — o protótipo já faz assim e fica assim.
- **[CORRIGE]** — o protótipo faz X, este PRD define Y, com o motivo.
- **[DECIDIDO]** — decisão tomada nesta rodada, contra o que o protótipo fazia ou omitia.

Ambiente medido na máquina alvo: dois monitores 1920×1080 (120Hz + 60Hz), **escala 1.0 em
ambos**, sem escala fracionária, `enable-hot-corners = false`.

---

## 1. Visão

Um Pomodoro em que a pausa **não é tempo morto**. Cada pausa pode carregar uma atividade
(nome + URL) que o app abre sozinho, em tela cheia e sem escapatória fácil, para que o
descanso seja dirigido em vez de virar rolagem infinita. O timer nunca para durante isso.

Não-objetivo: gerenciar tarefas, rastrear projetos, ou bloquear sites no sistema.

## 2. Glossário

| Termo | Definição |
|---|---|
| **Etapa** | A menor unidade de tempo. É `Foco` ou `Pausa`. |
| **Sessão** | Um par ordenado `Foco → Pausa`. Numerada de 1 a N. |
| **Ciclo** | O conjunto das N sessões. **Circular**: ao terminar a sessão N, volta à 1. |
| **Plano** | A lista de N sessões com duração de foco, duração de pausa e atividade. |
| **Prazo** | O instante absoluto em que a etapa em curso vence. **Fonte da verdade do tempo.** |
| **Atividade** | Nome opcional + URL opcional, exibida durante a pausa. |
| **Bloqueio** | Estado em que a atividade ocupa a tela e sair do app fica caro. |

## 3. Modelo de dados

```
Configuracao
├── versao: u32
├── quantidade_de_sessoes: 1..=24        (default 4)
├── modo_de_duracao: Global | Individual (default Global)
│
│   ── usados em modo Global ──
├── duracao_global_foco:  5..=180 min    (default 25)
├── duracao_global_pausa: 1..=60  min    (default 5)
├── atividade_global: Atividade          (uma só, repetida em todas as pausas)
│
│   ── usados em modo Individual ──
├── plano: [Sessao; quantidade_de_sessoes]
│   └── Sessao { foco: 5..=180, pausa: 1..=60, atividade: Atividade }
│
├── iniciar_automaticamente: bool        (default true)
└── posicao_do_widget: Option<(i32, i32)>

Atividade { nome: Option<String>, url: Option<UrlDeAtividade> }

ContadoresDoDia
├── dia_de_referencia: Data
├── sessoes_concluidas: u32
├── tempo_de_foco: Duracao
└── pausas_interrompidas: u32

HistoricoDiario  —  últimos 90 dias de ContadoresDoDia, anel circular
```

**Os dois modos coexistem.** As durações globais e o plano por sessão persistem ao mesmo tempo;
trocar de modo não destrói nem sobrescreve o outro conjunto. **[MOCKUP]**

**Atividade segue o modo.** **[DECIDIDO]** Em modo `Global` existe **uma atividade só**, aplicada
a todas as pausas; a lista por sessão nem aparece no menu. Em modo `Individual` cada sessão tem
a sua. **[CORRIGE]** o protótipo mantinha a lista por sessão visível nos dois modos.

## 4. Máquina de estados do timer

```
EstadoTimer
├── Ocioso                        etapa carregada, nunca iniciada
├── Correndo { prazo: Instante }  prazo é absoluto
└── Pausado  { restante: Duracao }
```

**O tempo restante deriva do prazo.** Não existe acumulador de tick.
**[CORRIGE]** o protótipo faz `remaining -= 0.1` a cada 100ms; suspensão do SO corrompe isso.

O Rust é dono do prazo. O React calcula o display `mm:ss` localmente a partir dele e **não**
recebe evento a 1Hz. Eventos só em transição de estado.

### 4.1 Transições

| Ação | Origem | Resultado |
|---|---|---|
| `alternar_execucao` | `Ocioso` / `Pausado` | `Correndo { prazo = agora + restante }`, fecha o menu |
| `alternar_execucao` | `Correndo` | `Pausado { restante = prazo - agora }` |
| `reiniciar_etapa` | qualquer, **exceto sob bloqueio** | `Ocioso`, restante = duração cheia, fecha o menu |
| `pular_etapa` | qualquer | `avancar(automatico = false)` |
| prazo vence | `Correndo` | `avancar(automatico = true)` |

### 4.2 `avancar`

1. Etapa alterna: `Foco → Pausa`, `Pausa → Foco`.
2. **A sessão só incrementa ao entrar em Foco**, circularmente: `(atual % total) + 1`. **[MOCKUP]**
3. Nova duração vem do modo vigente para a etapa e sessão de destino.
4. Auto-start assimétrico: **entrando em Pausa, sempre inicia**; entrando em Foco, respeita
   `iniciar_automaticamente`. **[MOCKUP]** — confirmado como intencional. **[DECIDIDO]**
5. Se a etapa de destino é Pausa **e** há URL de atividade aplicável → entra em **bloqueio**.
6. Se `automatico`, dispara a tomada de foco da janela (seção 8).
7. Contadores do dia são atualizados (seção 6).

### 4.3 Retomada após suspensão do SO

Ao acordar, `agora > prazo`. Isso **não é erro**: a etapa venceu enquanto a máquina dormia.
O app aplica `avancar(automatico = true)` **uma vez** e não encadeia etapas retroativamente —
dormir 3 horas não gera 6 sessões. O tempo de foco creditado é limitado ao prazo, nunca ao
despertar.

## 5. Plano do ciclo

- **Quantidade de sessões:** 1 a 24, validada no Rust. Fora do intervalo é erro nomeando o
  valor recebido e o intervalo esperado, não silêncio com clamp.
- **Redimensionar** de N para M:
  - `M < N`: trunca. Se a sessão em curso era `> M`, passa a ser `M`. **[MOCKUP]**
  - `M > N`: as novas sessões nascem com as durações globais vigentes e atividade vazia.
  - **Redimensionar com o timer correndo não altera o prazo em curso.** **[MOCKUP]**
- **Alterar a duração da etapa em curso:** o prazo é recalculado como `agora + nova_duracao`,
  não a partir do início original. Só se aplica quando a duração alterada é a que está em uso
  (mesmo modo, mesma etapa, mesma sessão). **[MOCKUP]**
- **Alterar duração com o timer parado:** recarrega o restante para a duração cheia.
- **Limites, fixos no código** (Configurações está fora da v1): **[DECIDIDO]**
  foco `5..=180` passo 5; pausa `1..=60` passo 1.

## 6. Contadores diários

- `sessoes_concluidas`: incrementa quando **um Foco chega ao prazo**. Pular um foco **não**
  conta como sessão concluída. **[DECIDIDO]**
- `tempo_de_foco`: soma o tempo **efetivamente decorrido em Foco**, inclusive o de um foco
  pulado pela metade. **[DECIDIDO]**
  **[CORRIGE]** o protótipo incrementa `completedToday` em toda saída de foco e calcula
  `focusToday = completedToday * focusMin` — 8 "sessões de 25 min" com 16 minutos de trabalho
  real, e número errado em modo individual. É mentira estatística.
- `pausas_interrompidas`: incrementa toda vez que o bloqueio é rompido — pela saída de Urgência
  ou por fuga detectada (7.4). É a métrica que dá sentido ao bloqueio: sem ela, um bloqueio
  furável não deixa rastro nenhum.
- **Virada do dia: meia-noite, horário local da máquina.** **[DECIDIDO]** Sem hora-âncora.
- **Histórico:** os últimos **90 dias** são guardados. Estatísticas está fora da v1, então não
  há consumidor hoje — mas descartar é irreversível, e 90 dias custam ~3KB.

## 7. Atividades e modo de bloqueio

### 7.1 A atividade

- Em modo Global, uma só; em modo Individual, uma por sessão (seção 3).
- Nome e URL independentemente opcionais.
- URL validada **no Rust**: apenas `http` e `https`. `file:`, `javascript:`, `data:` e demais
  são rejeitados com erro que nomeia o esquema ofensor. Validação no React é UX, não segurança.
- Rótulo exibido: o nome; na falta dele, o hostname sem `www.`; na falta dos dois, "atividade".
  **[MOCKUP]**
- **Música entra por aqui.** **[DECIDIDO]** Não há player, volume nem biblioteca própria: você
  passa a URL do serviço que quiser, igual a qualquer outro conteúdo.

### 7.2 Entrada em bloqueio

Automática ao entrar numa pausa com URL aplicável. O conteúdo é carregado num **child webview**
do Tauri — não `<iframe>`, porque sites como `music.youtube.com` recusam ser embutidos via
`X-Frame-Options`.

**Isolamento:** o webview tem label próprio, ausente de todo `capabilities/*.json`. Não alcança
nenhum command, evento ou API do Tauri. Roda em modo incógnito.

### 7.3 Durante o bloqueio

Objetivo: **impedir o usuário de sair do app**, sem tocar no resto do sistema. As outras
aplicações continuam rodando, nada é encerrado, o computador funciona normalmente.

**Dentro do app** — garantido, é o app que decide:
- Sem barra de navegação, sem voltar/avançar, sem recarregar.
- **Atalhos do app desativados**: Espaço, R, S, M, Esc e `Ctrl+Q`.
- Sem botão de fechar; `CloseRequested` é **recusado**.
- **O timer continua correndo.** O bloqueio nunca pausa nada.
- Moldura visível: `Pausa · {rótulo}`, `restam mm:ss`, barra de progresso.
- **Uma janela de bloqueio por monitor.** `set_fullscreen` cobre um monitor só; a máquina alvo
  tem dois, então o segundo recebe uma janela de moldura sem conteúdo. Sem isso o bloqueio é
  decorativo.

**Contra o compositor** — supressão temporária de atalhos do GNOME: **[DECIDIDO]**

O app desliga ~30 chaves gsettings ao entrar no bloqueio e as restaura ao sair. É configuração
de sessão, não protocolo: funciona igual em Wayland e XWayland, não exige autorização prévia do
usuário, não rouba o teclado e não usa `unsafe`.

| Esquema | Chaves |
|---|---|
| `org.gnome.desktop.wm.keybindings` | `switch-applications`(-backward), `switch-windows`(-backward), `switch-group`(-backward), `switch-panels`, `minimize`, `close`, `show-desktop`, `panel-run-dialog`, `switch-to-workspace-*` |
| `org.gnome.shell.keybindings` | `toggle-overview`, `toggle-application-view`, `toggle-message-tray`, `switch-to-application-1..9` |
| `org.gnome.mutter` | `overlay-key` (a tecla Super sozinha) |
| `org.gnome.mutter.wayland.keybindings` | `switch-to-session-1..12` (troca de VT, `Ctrl+Alt+F1..F12`) |

**Isto escreve na configuração do usuário e o dconf persiste entre reboots.** Tratado como
transação com journal, e as três regras não são opcionais:

1. **Snapshot** dos valores originais gravado em disco atomicamente **antes** de tocar em
   qualquer chave.
2. **Restauração** em `Drop`, no shutdown normal, e **na próxima inicialização** se houver
   snapshot pendente — é isto que cobre `kill -9`, crash e queda de energia.
3. Restaura o **valor original do usuário**, nunca o default de fábrica.

Falha ao ler ou escrever gsettings é degradação silenciosa, nunca `panic!`: o bloqueio segue,
mais fraco.

### 7.4 Fuga detectada

`WindowEvent::Focused(false)` com bloqueio ativo significa que o usuário escapou. O app:

1. Re-pede `set_fullscreen(true)` e raise da janela — best-effort.
2. Incrementa `pausas_interrompidas`.
3. Marca a pausa em curso como **interrompida**.
4. Ao recuperar o foco, exibe a confirmação de Urgência que foi pulada: ou o usuário confirma a
   interrupção, ou volta para o bloqueio.

O timer não é afetado em nenhum dos passos.

### 7.5 Limites conhecidos — o bloqueio é um compromisso, não uma prisão

Está no PRD porque é requisito de produto saber disso, não nota de rodapé técnica.

| Fuga | Bloqueada? |
|---|---|
| Fechar a janela (`Alt+F4`, `Super+Q`, botão) | **Sim** — `CloseRequested` recusado + keybind suprimido |
| `Alt+Tab`, `Super+Tab`, Super sozinha, overview | **Sim** — keybinds suprimidos |
| Trocar de workspace, `Alt+F2` | **Sim** — keybinds suprimidos |
| Trocar de VT (`Ctrl+Alt+F2`) | **Sim** — é keybind do mutter, também suprimido |
| **Mouse** | **Não, e de propósito** — ver abaixo |
| **Matar o processo** | **Não** — e o snapshot garante que os atalhos voltam na próxima abertura |
| Desligar a máquina | **Não** |

**O mouse fica livre por decisão, não por limitação.** **[DECIDIDO]** A saída de urgência é
`hover` na zona sensível + clique no botão + clique na confirmação: sem mouse, não há saída
nenhuma, e o bloqueio deixaria de ser um compromisso para virar uma armadilha. Nenhuma tentativa
de capturar ou confinar ponteiro entra neste app.

O risco residual disso — clicar numa janela de outro app — é coberto de outra forma: com
fullscreen nos dois monitores não sobra nada visível para clicar, e `enable-hot-corners` já está
desligado na máquina alvo. Se ainda assim o usuário achar um caminho, cai em 7.4.

Toda fuga não bloqueada cai em 7.4 e vira número em `pausas_interrompidas`.

### 7.6 Saída de urgência

**Único caminho de saída.** **[DECIDIDO]** Reiniciar a etapa não existe durante o bloqueio — o
`R` fica inerte, e o protótipo, que limpava `locked` no `reset()` mas nunca chegava lá, para de
se contradizer.

Controle oculto e **inteiramente dependente do mouse** — é por isso que o ponteiro nunca é
capturado (7.5). Zona sensível de **260×74px** no topo, centrada; ao passar o mouse, o botão
**Urgência** aparece. Clicar abre confirmação:

> **Interromper a pausa?**
> A atividade será encerrada e o modo de bloqueio desativado. O temporizador da pausa continua correndo.
> `[Continuar na pausa]` `[Confirmar urgência]`

Confirmar derruba o webview, restaura os atalhos e encerra o bloqueio. **Não altera o timer.**
Também conta em `pausas_interrompidas`.

## 8. Tomada de foco da janela

**[DECIDIDO]** Ao fim de um Foco **automático** (vencimento natural, nunca skip manual), **se a
pausa que começa tem atividade com URL aplicável**, o app traz a própria janela para frente e
mostra um overlay preto com "trazendo a janela para frente…" por ~850ms. Sem URL, a Pausa aparece
normalmente, sem nenhum efeito — não há atividade nenhuma para "trazer para frente". **[CORRIGE]**
decisão original não tinha essa condição; verificação ao vivo mostrou que o raise sem atividade
alguma parecia um comportamento errático, não intencional.

**Por que XWayland.** **[DECIDIDO]** Em Wayland puro isto é impossível: `xdg_toplevel` não tem
raise nem always-on-top, e `xdg-activation-v1` exige um token amarrado a interação recente do
usuário — que um timer não tem. Sob XWayland o app é cliente X11 e recupera `XRaiseWindow` e
`_NET_WM_STATE_ABOVE`, que o mutter honra independentemente de foco.

Custo aceito: não é Wayland nativo. O argumento clássico contra — escala fracionária borrada —
**não se aplica nesta máquina** (escala 1.0 nos dois monitores). Se o hardware mudar para HiDPI,
a decisão precisa ser revista.

Ressalva honesta: **raise ≠ foco de teclado.** O mutter aplica prevenção de roubo de foco também
a clientes X11 via `_NET_WM_USER_TIME`, então o teclado pode não seguir a janela. Para o
bloqueio isso é irrelevante (não há input a digitar). Tudo isto é medido em spike antes de virar
código definitivo.

## 9. Widget flutuante

**[DECIDIDO]** **Janela Tauri separada**, não `div` dentro do app. **[CORRIGE]** o protótipo
desenha o widget como `position: absolute` dentro da própria janela, o que some junto com ela.

- Sem decoração, **always-on-top** (`_NET_WM_STATE_ABOVE`), posicionável pelo app.
- **Fechado por padrão ao abrir o app.** Só abre quando o usuário clica no botão "Widget" do
  cabeçalho; só fecha (esconde) quando ele clica de novo, ou fecha a própria janela do widget pelo
  X. **[CORRIGE]** decisão original (2026-08-22) era "visível o tempo todo enquanto o app estiver
  aberto"; revertida a pedido do usuário (2026-08-25) — abrir o widget sem ação nenhuma do usuário
  incomodava mais do que ajudava.
- Dois estados: **expandido** (262px — relógio 44px, reset, play/pause, barra de progresso) e
  **minimizado** (pílula — ponto pulsante, relógio 17px, barra de 26px).
- Arrastável. Posição **persistida** entre execuções.
- Alternado pelo botão "Widget" no cabeçalho da janela principal.
- Durante o bloqueio o widget é escondido — a moldura já mostra o tempo restante.

## 10. Persistência

- Arquivo único TOML, caminho resolvido pelo Rust via `app.path()`. **Nenhum command aceita
  caminho vindo do frontend.**
- Escrita **atômica**: temporário no mesmo diretório → `fsync` → `rename`.
- Campo `versao` no topo. TOML inválido ou versão desconhecida ⇒ **o app abre com o default** e
  move o arquivo ruim para `.corrompido`. Nunca `panic!`.
- Persistido: tudo da seção 3, mais o histórico de 90 dias.
- **Não** persistido: estado do timer, menu aberto.
  **[CORRIGE]** o protótipo grava no `localStorage` a cada render e persiste `session`, o que faz
  o app reabrir no meio do ciclo com o timer zerado.
- **Snapshot de keybinds** (7.3) é arquivo separado, com o mesmo rigor de escrita atômica.

## 11. Atalhos e menu

| Tecla | Ação |
|---|---|
| `Espaço` | iniciar / pausar |
| `R` | reiniciar a etapa |
| `S` | pular a etapa |
| `M` | abrir / fechar o menu |
| `Esc` | fechar o menu |
| `Ctrl+Q` | sair |

Todos inertes durante o bloqueio, `Ctrl+Q` incluído.

**Menu:** `Sair` é implementado. **`Estatísticas` e `Configurações` ficam fora da v1** —
**[DECIDIDO]** — renderizados como no design, porém inertes e esmaecidos, para preservar a
paridade visual que é o contrato de porte.

## 12. Superfícies de UI

Sete estados. Cinco na janela principal, dois no widget:

1. **Timer principal** — cabeçalho (menu, badge `Foco|Pausa` + `Sessão N de M`, atalhos, botão
   Widget); anel SVG 480×480 com 60 ticks (maiores a cada 5) e arco de progresso; rótulo de
   estado (`pronto`/`em foco`/`em pausa`/`pausado`/`concluído`); relógio `mm:ss`; `termina HH:MM`;
   três botões (reset 54px, primário 76px, pular 54px); pontos do ciclo (o ativo vira barra de
   22px); rodapé com contadores à esquerda e `{foco}/{pausa} min · Ciclo N/M · {modo}` à direita.
2. **Menu lateral** 300px + scrim.
3. **Overlay de takeover.**
4. **Tela de bloqueio** (uma por monitor).
5. **Modal de confirmação de urgência.**
6. **Widget expandido** (janela própria).
7. **Widget minimizado** (janela própria).

### 12.1 Contrato visual

O design não tem folha de estilo: são estilos inline com interpolação, mais os pseudo-atributos
`style-hover` / `style-focus` / `style-active` do runtime do canvas. **Não há classe nem cascata
para preservar.** O porte garante **paridade de valor computado**, verificada por screenshot lado
a lado por estado.

| Token | Valor |
|---|---|
| Fundo | `#060607` + radial `rgba(255,255,255,.035)` a 90%/70% em 50%/42% |
| Texto | primário `#f3f2ef` · secundário `#b6b5b0` · terciário `#5f5f66` |
| Acento Foco | `oklch(0.79 0.135 62)` |
| Acento Pausa | `oklch(0.80 0.09 178)` |
| Menu / modal | `#0d0d10`, borda `rgba(255,255,255,.1)` |
| Widget | `#000`, borda `rgba(255,255,255,.12)`, raio 20px |
| Fontes | **Geist** (300/400/500/600) e **Geist Mono** (400/500) — **vendorizadas**, não CDN |
| Numerais | `font-variant-numeric: tabular-nums` em todo relógio |

Ícones: todos SVG inline. Nenhuma imagem, nenhum ícone externo.

## 13. Fora de escopo na v1

- **Estatísticas** e **Configurações** — itens de menu inertes.
- **Som ao fim da etapa, tray icon, autostart no login** — sem desenho.
- **Fechar a janela com o timer correndo:** o app encerra e o timer morre. Decorre da seção 10,
  que não persiste estado de timer.
- **Bloqueio em nível de SO**: encerrar, esconder ou impedir outras aplicações. O computador
  funciona normalmente durante o bloqueio.
- **Grab de teclado** (`XGrabKeyboard` / `xwayland-keyboard-grab-v1`): exigiria autorização
  prévia via `xwayland-grab-access-rules` e carrega risco de teclado preso. A supressão de
  keybinds cobre o mesmo caso com risco menor.
- **Extensão do GNOME Shell**: único caminho para o widget aparecer dentro do overview, mas é um
  segundo artefato em JS que quebra a cada release.
- Sincronização, conta, nuvem. Windows e macOS.

## 14. Decisões — todas fechadas em 2026-08-22

| # | Assunto | Decisão |
|---|---|---|
| 1 | Reiniciar durante o bloqueio | **Não existe.** Urgência é a única saída. |
| 2 | Pular o foco pela metade | Sessão só conta ao vencer o prazo; `tempo_de_foco` soma o decorrido real. |
| 3 | Escopo do bloqueio | Impedir sair **do app**, sem tocar no SO. |
| 4 | Virada do dia | Meia-noite, horário local da máquina. Histórico de 90 dias guardado. |
| 5 | Música na pausa | É URL, como qualquer outro conteúdo. Sem player próprio. |
| 6 | Estatísticas e Configurações | Fora da v1, renderizados inertes. |
| 7 | Auto-start assimétrico | Intencional. Pausa sempre inicia; foco respeita o toggle. |
| 8 | Atividade em modo Global | **Uma só**, repetida em todas as pausas. Lista por sessão só em modo Individual. |
| 9 | Limites de duração | Fixos no código: foco 5–180 passo 5; pausa 1–60 passo 1. |
| 10 | Som, tray, autostart | Sem desenho; fora da v1. |
| 11 | Widget | Janela separada, always-on-top, sob XWayland. |
| 12 | Supressão de atalhos | ~30 chaves gsettings, com journal e restauração na próxima abertura. |
| 13 | Mouse | **Nunca capturado nem confinado.** A saída de urgência depende dele. |
