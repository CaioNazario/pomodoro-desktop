# App Pomodoro

> Comportamento é definido em `PRD.md`. Este arquivo é sobre **como** escrever, não sobre o quê.

## Stack
Rust (edition 2021), React 19, Tauri 2. Configuração em **TOML versionado** com escrita atômica —
não há banco. Testes: `#[cfg(test)]` nativo + `tests/` para integração. Sem framework externo.

**Requisito de execução: `GDK_BACKEND=x11`.** O app roda sob XWayland de propósito. Em Wayland
puro, `set_position` é ignorado, always-on-top não existe e a janela não sobe sozinha — o widget
flutuante e a tomada de foco no fim do pomodoro seriam impossíveis. Medido, não suposto (ver
Hurdles). O custo clássico do XWayland (escala fracionária borrada) não se aplica enquanto os
monitores estiverem em escala 1.0. **Se o hardware virar HiDPI, esta decisão precisa ser revista.**

## Comandos
- `./scripts/ci.sh` — **a porta única**. Toda fatia fecha com isto verde antes da próxima começar.
- `cargo test --workspace` — suíte completa. **Nunca** `--manifest-path src-tauri/Cargo.toml`:
  isso testa só o crate do Tauri e deixa o domínio inteiro de fora, com o CI passando verde e
  mentindo.
- `cargo clippy --workspace --all-targets -- -D warnings` — clippy quebra o build
- `cargo fmt --all --check` — formatação não é opinião
- `pnpm tauri dev` / `pnpm tauri build`
- `cargo test -- --nocapture` — quando precisar ver `dbg!`

## Fluxo de correção de bug
- Toda vez que o usuário pedir para corrigir um bug: depois da correção, suba
  o binário (`GDK_BACKEND=x11 pnpm tauri dev`, em background) pra ele testar
  antes de qualquer commit/push. CI verde não é prova de que o bug sumiu —
  ver Hurdles: só roda de verdade sob XWayland.

## Regras de código
- Funções: 4-20 linhas. Arquivos: abaixo de 400 linhas.
  Exceção única: `match` exaustivo sobre enum grande. Se estourou por outro motivo, extraia.
- Nomes específicos e grepáveis. Proibido: `Manager`, `Handler`, `Util(s)`, `data`, `process`,
  `mod common`, `mod helpers`. **Carve-out:** vale para os *nossos* nomes — a API do Tauri é
  `Manager` e `AppHandle` e não vai mudar.
- Sem lógica no `#[tauri::command]`. Command = desserializar + validar input + delegar + mapear
  erro. Se tem `if` de regra de negócio ali dentro, está no lugar errado.
- Dependência entra por parâmetro ou campo de struct, construída no `setup()` do Tauri e injetada
  via `State<T>`. Proibido: `static mut`, `lazy_static!`, `OnceCell` global, singleton disfarçado
  de "cache".
- I/O atrás de trait (`Relogio`, repositórios). O domínio não conhece `std::fs` nem `SystemTime`.
- Erro sempre com `thiserror` e enum próprio por camada. A mensagem carrega valor ofensor E
  esperado: `#[error("duracao invalida: {got}min, esperado entre {min} e {max}")]`
- Proibido `unwrap()` / `expect()` / `panic!()` fora de teste, do `main` **e do bootstrap do Tauri
  (`lib.rs::run()`)**. `anyhow` só no binário, `thiserror` nas libs. Erro como `String` é preguiça.
- Early return com `?`, `let ... else` e guard clause. Máximo 2 níveis de indentação.
- Sem `.clone()` para calar o borrow checker. Se clonou, o comentário explica por quê.

### Regras do CLAUDE.md global que NÃO valem aqui
Rust não é Java. Estas foram suspensas de propósito:
- **"4 pilares de OOP", incluindo herança** — Rust não tem herança. Composição e traits.
- **"Um ponto por linha"** (object calisthenics) — hostil a Rust idiomático: mata `?`, cadeia de
  iterador e `estado.plano().sessao(n)?`.
- **"Proibido `else`"** — em Rust `if/else` é expressão; banir produz código pior.
  Vale a intenção: prefira `match` e expressão. Guard clause continua obrigatória.
- **"Sem getters/setters", `@Data`, Records** — folclore de repositório Java.
- Acessor em newtype (`Duracao::em_ms`) é idiomático, não é getter de anemic model.

## Regras de segurança (não negociáveis)
- `#![forbid(unsafe_code)]` no topo de **cada crate** do workspace. **Sem exceção neste projeto** —
  o CI falha se `unsafe` aparecer em `crates/` ou `src-tauri/src/`.
- `#![forbid(clippy::disallowed_methods)]` no domínio, junto com `crates/pomodoro-dominio/clippy.toml`.
  `forbid`, não `deny`: com `deny` bastaria um `#[allow]` em cima da função para furar a regra.
- Capabilities do Tauri 2 no mínimo absoluto. Nada de `core:default` inflado "só pra destravar".
  O webview da atividade tem label próprio, **ausente de todo `capabilities/*.json`** ⇒ zero IPC.
- CSP definida e restritiva em `tauri.conf.json`. Proibido `dangerousDisableAssetCspModification`.
  Fontes vendorizadas localmente — app desktop não busca fonte em CDN.
- O frontend é entrada não confiável. Todo input de command é validado no Rust, ponto.
  Validação no React é UX, não é segurança.
- Nenhum command aceita caminho de arquivo cru vindo do frontend. Path resolvido pelo Rust via
  `app.path()`, sempre.
- Segredo/token nunca chega no bundle do frontend.
- Configuração versionada e migração idempotente. Versão desconhecida ou arquivo corrompido ⇒
  abre com o default e move o arquivo ruim para `.corrompido`. Nunca `panic!`.
- **Efeito colateral fora do app:** o modo de bloqueio escreve em gsettings do usuário. Nenhuma
  escrita sem snapshot atômico prévio em disco, e restauração obrigatória em `Drop`, no shutdown
  **e na próxima inicialização** se houver snapshot pendente. É isto que cobre `kill -9`.
- O mouse **nunca** é capturado nem confinado. A saída de urgência depende dele.

## Testes
- Toda função de domínio nova tem teste. Todo bug fix nasce com teste de regressão que falha antes.
- Caminho de erro tem tanto teste quanto caminho feliz. Enum de erro sem teste é decoração.
- Nada de mock do próprio código de negócio. Fake de trait só para I/O externo.
- Config de teste é arquivo real em `tempdir`. Não se mocka `std::fs`.
- `Relogio` é trait. Nenhum teste de pomodoro espera 25 minutos de verdade.
- Teste de command Tauri é integração e vive em `tests/`, usando a feature `test` do próprio Tauri
  (`tauri::test::mock_builder`) — isso conta como "do Tauri", não como framework externo.
- Unitário fica inline no módulo.

## Hurdles conhecidos

### Wayland vs XWayland (medido em 2026-08-22, GNOME 50.4, mutter, tao 0.35.3)
Spike com duas janelas reais, observado por fora com `xprop`/`xdotool`:

| Chamada | XWayland | Wayland nativo |
|---|---|---|
| `set_outer_position(150,150)` | funciona; `xdotool` confirma o movimento | **ignorado**, reporta `x=0 y=0` |
| `set_always_on_top(true)` | funciona; `_NET_WM_STATE_ABOVE` aparece | conceito não existe em `xdg_toplevel` |
| `set_fullscreen` | funciona; `0,0 1920x1080` | aceito |
| `set_focus()` **roubando foco de outra janela** | **funciona** — `_NET_WM_STATE_FOCUSED` migra | impossível: `xdg-activation-v1` exige token de interação recente, e o receptor está no GTK4 (usamos GTK3) |

Consequências para o código:
- **`_NET_ACTIVE_WINDOW` não aponta para a nossa janela** mesmo com `_NET_WM_STATE_FOCUSED` setado
  nela — quirk do mutter com XWayland. O sinal de foco no app é `WindowEvent::Focused` do Tauri,
  **nunca** `xdotool getactivewindow` nem a propriedade X11.
- **`_NET_CLIENT_LIST_STACKING` fica vazio** sob mutter: não dá para verificar ordem de
  empilhamento por fora. Se precisar provar "está na frente", prove por `ABOVE` + `FOCUSED`.

### Não verificado
- **Multi-monitor.** O `monitors.xml` lista `eDP-1` e `HDMI-1`, mas só `eDP-1` estava conectado
  durante o spike. "Uma janela de bloqueio por monitor" (PRD 7.3) **nunca foi testado com dois
  monitores de verdade**. Testar ao plugar o segundo.
