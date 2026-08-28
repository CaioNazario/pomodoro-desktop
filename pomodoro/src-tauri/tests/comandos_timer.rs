//! Integracao real via `tauri::test::mock_builder` — ver CLAUDE.md do projeto:
//! "Teste de command Tauri e integracao e vive em tests/, usando a feature
//! test do proprio Tauri". `alternar_execucao`/`reiniciar_etapa`/`pular_etapa`
//! usam `AppHandle` concreto (Wry), incompativel com `MockRuntime` — a
//! transicao em si ja tem cobertura exaustiva no dominio (`CicloEmExecucao`).
//! Aqui validamos o que e especifico da camada Tauri: o command real lendo
//! estado gerenciado de verdade atraves do IPC.

use pomodoro_contrato::EstadoDaTela;
use pomodoro_dominio::CicloEmExecucao;
use pomodoro_lib::ciclo_inicial;
use pomodoro_lib::comandos::timer::*;
use pomodoro_lib::relogio_do_sistema::RelogioDoSistema;
use std::sync::Mutex;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

fn montar_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![obter_estado])
        .build(tauri::generate_context!())
        .expect("falha ao montar app de teste");
    app.manage(Mutex::new(ciclo_inicial()));
    app.manage(RelogioDoSistema);
    app
}

fn obter_estado_via_ipc<W: AsRef<tauri::Webview<tauri::test::MockRuntime>>>(
    webview: &W,
) -> EstadoDaTela {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: "obter_estado".into(),
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
            .deserialize::<EstadoDaTela>()
            .expect("resposta invalida")
    })
    .expect("comando obter_estado falhou")
}

#[test]
fn obter_estado_reflete_o_estado_inicial_gerenciado() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    let estado = obter_estado_via_ipc(&webview);
    assert_eq!(estado.numero_sessao, 1);
    assert_eq!(estado.total_sessoes, 4);
    assert!(!estado.rodando);
    assert!(estado.prazo_epoca_ms.is_none());
}

#[test]
fn obter_estado_reflete_mutacao_no_estado_gerenciado_pelo_tauri() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    {
        let ciclo_state = app.state::<Mutex<CicloEmExecucao>>();
        let mut ciclo = ciclo_state.lock().expect("lock do ciclo");
        let relogio = app.state::<RelogioDoSistema>();
        *ciclo = ciclo.alternar_execucao(pomodoro_dominio::Relogio::agora(&*relogio));
    }

    let estado = obter_estado_via_ipc(&webview);
    assert!(estado.rodando);
    assert!(estado.prazo_epoca_ms.is_some());
}
