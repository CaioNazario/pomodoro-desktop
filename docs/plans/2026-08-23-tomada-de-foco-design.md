# Design: fatia 6 — tomada de foco ao fim do Foco

## Contexto

PRD §8: ao fim de um Foco, o app traz a própria janela pra frente e mostra um overlay preto
"trazendo a janela para frente…" por ~850ms. PRD §4.2 passo 6 fala de "automatico" disparando a
tomada de foco — no código isso já existe como distinção implícita: `verificador_de_vencimento`
trata vencimento natural (o único caminho automático), `pular_etapa` é sempre manual e nunca
passa por ali.

Confirmado com o usuário: o gatilho é **só fim de Foco** (`etapa_antes == Foco`), não qualquer
transição automática — fim de Pausa não dispara raise. O overlay é inferido pelo próprio
frontend a partir de `estadoMudou`, sem campo novo no contrato Rust→TS.

## Backend

Função pura `deve_tomar_foco(etapa_antes: Etapa, etapa_depois: Etapa) -> bool` — só `true` para
`Foco -> Pausa`. Sem Tauri, sem `AppHandle`, testável isoladamente (TDD).

Chamada em `verificador_de_vencimento::verificar()`, que já tem `ciclo_antes.etapa()` e
`etapa_depois` disponíveis no mesmo lugar onde já chama `bloqueio::reagir_a_transicao`. Se
`true`, chama `app.get_webview_window("main").and_then(|w| w.set_focus().ok())` — best-effort,
erro vira `eprintln!`, nunca interrompe o verificador (mesmo padrão de `persistencia::persistir`
e do resto do módulo).

Já medido no spike da fatia 0 (`CLAUDE.md`, "Hurdles conhecidos"): `set_focus()` funciona sob
XWayland pra roubar foco de outra janela. Sem crate X11 nova, sem `unsafe`.

## Frontend

Sem mudança de contrato/bindings. Hook novo `useOverlayDeTomadaDeFoco(estado: EstadoDaTela |
null)`: guarda a etapa anterior via `useRef`, compara a cada mudança de `estado.etapa` recebida
via `estadoMudou`; ao detectar `Foco -> Pausa`, liga `overlayAtivo` por 850ms via `setTimeout`.
Componente novo `OverlayDeTomadaDeFoco` — paridade de valor computado com `takeoverOpacity` do
design canvas: preto, `inset: 0`, `z-index: 30`, `pointer-events: none`, fade `opacity .35s ease`.

## Testes

`deve_tomar_foco`: as 4 combinações de etapa (unitário, RED antes de GREEN). `set_focus()` real
não é testável via `mock_builder` — mesma limitação já documentada pra todo command com
`AppHandle`/`Window` real (`alternar_execucao`, `reiniciar_etapa`, etc.).

## Verificação

Boot sem crash com o novo código no `setup()`/loop do verificador, via `GDK_BACKEND=x11 pnpm
tauri dev`. A janela subindo de verdade ao fim de um Foco real (mínimo 5min, limite fixo do
domínio) fica **pendência registrada** — decisão do usuário, mesmo padrão já aceito pra fatia 5.
