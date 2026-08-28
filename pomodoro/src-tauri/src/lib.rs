#![forbid(unsafe_code)]

mod bloqueio;
pub mod comandos;
mod eventos;
mod persistencia;
mod registro_de_foco;
pub mod relogio_do_sistema;
mod tomada_de_foco;
mod verificador_de_vencimento;
pub mod webview_atividade;
mod widget;

use comandos::bloqueio::confirmar_urgencia;
use comandos::historico::obter_contadores_do_dia;
use comandos::plano::{
    alterar_atividade_global, alterar_atividade_individual, alterar_duracao_global_foco,
    alterar_duracao_global_pausa, alterar_duracao_individual_foco,
    alterar_duracao_individual_pausa, obter_plano, redimensionar, trocar_modo,
};
use comandos::timer::{
    alterar_iniciar_automaticamente, alternar_execucao, obter_estado, pular_etapa, reiniciar_etapa,
};
use comandos::widget::alternar_widget;
use pomodoro_config::Configuracao;
use pomodoro_dominio::{
    CicloEmExecucao, Duracao, HistoricoDiario, NumeroDeSessao, PlanoDoCiclo, QuantidadeDeSessoes,
};
use relogio_do_sistema::RelogioDoSistema;
use std::sync::Mutex;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};

const TOTAL_SESSOES_INICIAL: u8 = 4;
const DURACAO_FOCO_INICIAL_MIN: u16 = 25;
const DURACAO_PAUSA_INICIAL_MIN: u16 = 5;

fn total_sessoes_inicial() -> QuantidadeDeSessoes {
    QuantidadeDeSessoes::nova(TOTAL_SESSOES_INICIAL)
        .expect("total de sessoes inicial esta dentro do intervalo valido")
}

/// Configuracao de fabrica — usada so quando nao ha TOML no disco, ou ele
/// esta corrompido/de versao desconhecida (ver `pomodoro_config::Armazenamento`).
fn configuracao_padrao() -> Configuracao {
    Configuracao {
        plano: PlanoDoCiclo::novo(
            total_sessoes_inicial(),
            Duracao::de_minutos(DURACAO_FOCO_INICIAL_MIN),
            Duracao::de_minutos(DURACAO_PAUSA_INICIAL_MIN),
        ),
        iniciar_automaticamente: true,
        historico: HistoricoDiario::vazio(),
        posicao_do_widget: None,
    }
}

/// Reconstroi o ciclo em execucao a partir de uma `Configuracao` carregada
/// (ou de fabrica) — sempre comeca na primeira sessao, em Foco, ocioso. O
/// estado do timer em si nunca e persistido (PRD §10).
fn ciclo_a_partir_de(config: &Configuracao) -> CicloEmExecucao {
    let (foco, pausa) = config
        .plano
        .duracoes_ativas_para(NumeroDeSessao::primeira());
    let total = QuantidadeDeSessoes::nova(config.plano.total_sessoes())
        .expect("total persistido ja foi validado ao ser escrito");
    CicloEmExecucao::novo(total, foco, pausa, config.iniciar_automaticamente)
}

/// Ciclo, plano e historico de fabrica — expostos pra testes de integracao
/// montarem estado gerenciado sem depender do disco (ver
/// `src-tauri/tests/`).
pub fn ciclo_inicial() -> CicloEmExecucao {
    ciclo_a_partir_de(&configuracao_padrao())
}

pub fn plano_inicial() -> PlanoDoCiclo {
    configuracao_padrao().plano
}

pub fn historico_inicial() -> HistoricoDiario {
    configuracao_padrao().historico
}

/// Journal do snapshot de atalhos suprimidos (PRD §7.3) — resolvido via
/// `app.path()`, nunca hardcodado. `temp_dir()` e o unico fallback razoavel
/// se o SO nao der um diretorio de config: preferir persistir em algum
/// lugar a nao suprimir atalho nenhum por falta de onde gravar o snapshot.
fn caminho_snapshot_de_atalhos(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("atalhos-gnome.snapshot.toml")
}

/// Fechar a `main` encerra o app inteiro — o produto nao tem tray icon pra
/// minimizar (PRD §13), entao nao ha estado intermediario.
///
/// Reage a `Destroyed`, nao a `CloseRequested` — duas abordagens mais obvias
/// foram tentadas e medidas ao vivo, e as duas sao racy:
///
/// 1. Destruir o widget a partir do `CloseRequested` da `main` (via
///    `destroy()` por janela auxiliar): `destroy()`/`close()` apenas
///    enfileiram uma mensagem no event loop via `EventLoopProxy::send_event`
///    (ver `tauri-runtime-wry::lib::destroy`, comentado como "cannot use
///    send_user_message because it accesses the event loop callback") —
///    processada numa passada *futura* do loop, nao nesta. ~30-40% das vezes
///    essa mensagem nunca chegava a ser processada e o processo ficava preso
///    pra sempre com o widget ainda no mapa — o bug original, via outro
///    caminho.
/// 2. `api.prevent_close()` na `main` + `app.exit(0)` direto no
///    `CloseRequested`: ainda ~20% de falha. O log mostrava
///    "GdkWindow ... unexpectedly destroyed" na propria `main` mesmo com
///    `prevent_close()` chamado — o GTK as vezes já decidiu destruir a
///    janela nativa antes do nosso handler correr, e as duas coisas correm
///    em paralelo.
///
/// A janela `main` sempre acaba sendo destruida de verdade, de forma
/// confiavel (visto nos dois casos acima). Entao, em vez de tentar prevenir
/// ou correr contra esse fechamento, so **observamos** ele terminar
/// (`WindowEvent::Destroyed`, que so dispara depois que a janela ja sumiu de
/// verdade) e ai chamamos `app.exit(0)` — mesmo `AppHandle::exit` que o
/// "Sair" do menu lateral ja usa via `tauri-plugin-process`, historicamente
/// confiavel. `exit()` enfileira `RequestExit`, que seta `ControlFlow::Exit`
/// **incondicionalmente**, sem checar se o mapa de janelas esta vazio —
/// mata qualquer janela auxiliar ainda viva (o widget) junto, sem precisar
/// destruir cada uma manualmente.
fn encerrar_app_ao_fechar_main(app: &tauri::AppHandle) {
    let Some(janela_principal) = app.get_webview_window("main") else {
        return;
    };
    let handle = app.clone();
    janela_principal.on_window_event(move |evento| {
        if matches!(evento, tauri::WindowEvent::Destroyed) {
            handle.exit(0);
        }
    });
}

/// Callback de segunda instancia (`tauri-plugin-single-instance`): o SO
/// relanca o binario ao clicar no atalho de novo, mas so deve existir um
/// processo. Em vez de deixar uma segunda janela `main` nascer, trazemos a
/// existente de volta — cobre tanto oculta (nunca acontece hoje, ja que
/// fechar a `main` encerra o app, mas cobre minimizada) quanto atras de
/// outras janelas.
fn focar_janela_principal(app: &tauri::AppHandle) {
    let Some(janela) = app.get_webview_window("main") else {
        return;
    };
    let _ = janela.unminimize();
    let _ = janela.show();
    let _ = janela.set_focus();
}

fn especificar_contrato() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            obter_estado,
            alternar_execucao,
            reiniciar_etapa,
            pular_etapa,
            alterar_iniciar_automaticamente,
            obter_plano,
            redimensionar,
            trocar_modo,
            alterar_duracao_global_foco,
            alterar_duracao_global_pausa,
            alterar_duracao_individual_foco,
            alterar_duracao_individual_pausa,
            alterar_atividade_global,
            alterar_atividade_individual,
            obter_contadores_do_dia,
            confirmar_urgencia,
            alternar_widget
        ])
        .events(collect_events![
            eventos::EstadoMudou,
            eventos::PlanoMudou,
            eventos::HistoricoMudou,
            eventos::BloqueioMudou,
            eventos::TomadaDeFocoOcorreu
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = especificar_contrato();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("gerar src/bindings.ts: contrato Rust<->TS invalido");

    let invoke_handler = builder.invoke_handler();

    let mut tauri_builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        // Precisa ser o primeiro plugin registrado (doc oficial do
        // `tauri-plugin-single-instance`) pra interceptar o relancamento
        // antes de qualquer outra inicializacao.
        tauri_builder = tauri_builder.plugin(tauri_plugin_single_instance::init(
            |app, _argumentos, _diretorio| {
                focar_janela_principal(app);
            },
        ));
    }

    tauri_builder
        .plugin(tauri_plugin_process::init())
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            builder.mount_events(app);
            let handle = app.handle().clone();
            let config = persistencia::carregar(&handle, configuracao_padrao);
            app.manage(Mutex::new(ciclo_a_partir_de(&config)));
            app.manage(Mutex::new(config.plano));
            app.manage(Mutex::new(config.historico));
            app.manage(RelogioDoSistema);
            let gerenciador_de_bloqueio =
                bloqueio::GerenciadorDeBloqueio::novo(caminho_snapshot_de_atalhos(&handle));
            gerenciador_de_bloqueio.recuperar_pendente();
            app.manage(gerenciador_de_bloqueio);
            app.manage(widget::EstadoDoWidget::novo(config.posicao_do_widget));
            encerrar_app_ao_fechar_main(&handle);
            verificador_de_vencimento::iniciar(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Tauri");
}
