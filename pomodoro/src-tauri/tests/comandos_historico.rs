//! `obter_contadores_do_dia` nao recebe `AppHandle` (so le estado), entao
//! e testavel via IPC real no `mock_builder` — mesmo padrao de
//! `obter_estado` em `comandos_timer.rs`.

use pomodoro_contrato::EstadoDoHistorico;
use pomodoro_dominio::{Duracao, HistoricoDiario, Relogio};
use pomodoro_lib::comandos::historico::*;
use pomodoro_lib::comandos::timer::*;
use pomodoro_lib::relogio_do_sistema::RelogioDoSistema;
use pomodoro_lib::{ciclo_inicial, historico_inicial};
use std::sync::Mutex;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

fn montar_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            obter_estado,
            obter_contadores_do_dia
        ])
        .build(tauri::generate_context!())
        .expect("falha ao montar app de teste");
    app.manage(Mutex::new(ciclo_inicial()));
    app.manage(Mutex::new(historico_inicial()));
    app.manage(RelogioDoSistema);
    app
}

fn obter_contadores_via_ipc<W: AsRef<tauri::Webview<tauri::test::MockRuntime>>>(
    webview: &W,
) -> EstadoDoHistorico {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: "obter_contadores_do_dia".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().expect("url de teste"),
            body: InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|corpo| {
        corpo
            .deserialize::<EstadoDoHistorico>()
            .expect("resposta invalida")
    })
    .expect("comando obter_contadores_do_dia falhou")
}

#[test]
fn contadores_iniciais_do_dia_estao_zerados() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    let estado = obter_contadores_via_ipc(&webview);
    assert_eq!(estado.sessoes_concluidas_hoje, 0);
    assert_eq!(estado.tempo_de_foco_hoje_ms, 0);
}

#[test]
fn contadores_refletem_mutacao_no_historico_gerenciado_pelo_tauri() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    {
        let historico_state = app.state::<Mutex<HistoricoDiario>>();
        let relogio = app.state::<RelogioDoSistema>();
        let mut historico = historico_state.lock().expect("lock do historico");
        *historico = historico
            .clone()
            .registrar_sessao_concluida(relogio.hoje(), Duracao::de_minutos(25));
    }

    let estado = obter_contadores_via_ipc(&webview);
    assert_eq!(estado.sessoes_concluidas_hoje, 1);
    assert_eq!(
        estado.tempo_de_foco_hoje_ms,
        Duracao::de_minutos(25).em_ms()
    );
}
