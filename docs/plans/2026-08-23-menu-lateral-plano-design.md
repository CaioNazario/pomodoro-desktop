# Design: expandir o menu lateral (plano de sessões)

## Contexto

Desde a fatia 2, `MenuLateral.tsx` só tem o shell (painel + scrim) e `Sair`. Os controles de
plano do design canvas (`Pomodoro Desktop.dc.html`, linhas ~120-200) — contador de sessões,
switch Global/Individual, durações, "iniciar automaticamente", atividades nas pausas — nunca
foram portados, embora os commands de backend para a maioria já existam desde as fatias 2 e 4.

Confirmado com o usuário: esses controles ficam no **corpo** do menu, sem gate por clique. Os
botões separados "Estatísticas" e "Configurações" (com atalho `Ctrl ,` no design) continuam
inertes — o `[DECIDIDO]` do PRD §11 fala desses dois botões especificamente, não dos controles de
plano. Nenhuma mudança de PRD necessária.

## Escopo

Um commit só (mudança coesa dentro de um componente).

## Backend — dois pedaços novos

Todo o resto (redimensionar, trocar_modo, durações, atividades) já tem command pronto desde as
fatias 2 e 4 — só falta wiring de frontend.

### 1. `obter_plano` — command de leitura

Não existe hoje nenhum command de leitura para `PlanoDoCiclo`: só os de mutação, que devolvem
`EstadoDoPlano` como efeito colateral. Sem isso o menu abriria sem dado nenhum até a primeira
mutação. Espelha exatamente `obter_estado` — sem `AppHandle`, lê `State<Mutex<PlanoDoCiclo>>`,
mapeia para `EstadoDoPlano`, testável via `tauri::test::mock_builder`.

### 2. `alterar_iniciar_automaticamente(bool)` + campo novo em `EstadoDaTela`

`iniciar_automaticamente` hoje só existe como campo lido uma vez no boot
(`CicloEmExecucao::novo`), sem setter no domínio. Trabalho:
- Setter no domínio (`CicloEmExecucao`) que troca só esse campo, sem tocar timer/etapa/sessão em
  curso.
- Command novo em `comandos/timer.rs` (mexe em `CicloState`, não `PlanoState`), chama
  `persistencia::persistir` depois (já lê `ciclo.iniciar_automaticamente()`).
- Campo novo `iniciarAutomaticamente: bool` em `EstadoDaTela` — decidido em vez de um command de
  leitura próprio, porque reaproveita o listener que o timer já usa (`obterEstado`/
  `estadoMudou`), sem criar um terceiro fetch/listener só para um boolean.
- Teste de domínio (setter reflete o valor) + teste de integração do command.

`bindings.ts` regenerado rodando o binário de verdade (não só o diff-check do CI, que é teatro —
ver Brain, sessão fatia 4).

## Frontend — `MenuLateral.tsx`

Hoje é só shell. Vira: hook `usePlanoDoCiclo()` (busca inicial via `obterPlano`, atualiza via
evento `planoMudou`, mesmo padrão de `useContadoresDoDia`) + sub-componentes extraídos do design
canvas, cada um pequeno o bastante pra não estourar o limite de 400 linhas do arquivo:

- `SeletorDeSessoes` — contador +/-, chama `redimensionar`.
- `SeletorDeModo` + `DuracoesDoPlano` — switch Global/Individual (`trocarModo`); renderiza os
  controles certos conforme `modo`: sliders globais (`alterarDuracaoGlobalFoco/Pausa`) ou lista
  individual por sessão (`alterarDuracaoIndividualFoco/Pausa`).
- `ToggleAutoStart` — lê `iniciarAutomaticamente` do `EstadoDaTela` (via o hook do timer já
  existente, `useEstadoDoTimer`), chama `alterarIniciarAutomaticamente`.
- `AtividadesNasPausas` — lista de campos nome+URL por sessão de pausa, chama
  `alterarAtividadeGlobal`/`alterarAtividadeIndividual` conforme o modo.

## Dados e erros

`EstadoDoPlano`/`EstadoDaTela` são a fonte da verdade — nenhum estado local duplicado além do
buffer de input controlado nos campos de texto/número (padrão já usado no design canvas:
`onChange` bufferiza, `onBlur`/commit chama o command). Todo command de plano devolve
`ErroComandoDePlano` tipado; erro de validação (ex.: duração fora do passo) reverte o campo pro
último valor confirmado — sem alert/toast, mensagem inline discreta é suficiente pro v1 (mesmo
nível de polish dos outros commands de plano, que hoje não têm UI nenhuma de erro porque não
tinham UI nenhuma).

## Testes

- Domínio: setter de `iniciar_automaticamente` (novo).
- Integração Tauri: `obter_plano` e `alterar_iniciar_automaticamente` via `mock_builder`.
- Frontend: sem framework de teste configurado no projeto (ver `CLAUDE.md` — só `#[cfg(test)]`
  nativo Rust); verificação do frontend é manual, `GDK_BACKEND=x11 pnpm tauri dev`, mesma
  disciplina das sessões anteriores (CI verde não é prova de pronto).
