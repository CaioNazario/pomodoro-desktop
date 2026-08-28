//! Integracao real via `tauri::test::mock_builder` — mesmo padrao de
//! `comandos_timer.rs`. Os 6 commands novos de `comandos/plano.rs`
//! (`redimensionar`, `trocar_modo`, `alterar_duracao_global_foco`,
//! `alterar_duracao_global_pausa`, `alterar_duracao_individual_foco`,
//! `alterar_duracao_individual_pausa`) recebem `AppHandle` concreto (Wry)
//! porque emitem eventos — incompativel com `MockRuntime`, mesma restricao
//! ja documentada la. Por isso aqui exercitamos a integracao da camada
//! Tauri (estado gerenciado real, tipos na fronteira) chamando os metodos
//! de dominio diretamente sobre o `State` gerenciado, exatamente como o
//! corpo do command faria, e conferimos o resultado atraves de
//! `obter_estado` — o unico command da fatia sem `AppHandle` e, portanto,
//! o unico testavel via IPC real. A logica de transicao em si ja tem
//! cobertura exaustiva inline em `crates/pomodoro-dominio/`.

use pomodoro_contrato::EstadoDaTela;
use pomodoro_dominio::{
    Atividade, CicloEmExecucao, Duracao, NumeroDeSessao, PlanoDoCiclo, QuantidadeDeSessoes,
    Relogio, SessaoForaDoPlano, UrlDeAtividade,
};
use pomodoro_lib::comandos::plano::ErroComandoDePlano;
use pomodoro_lib::comandos::timer::*;
use pomodoro_lib::relogio_do_sistema::RelogioDoSistema;
use pomodoro_lib::{ciclo_inicial, plano_inicial};
use std::sync::Mutex;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

fn montar_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            obter_estado,
            pomodoro_lib::comandos::plano::obter_plano
        ])
        .build(tauri::generate_context!())
        .expect("falha ao montar app de teste");
    app.manage(Mutex::new(ciclo_inicial()));
    app.manage(Mutex::new(plano_inicial()));
    app.manage(RelogioDoSistema);
    app
}

fn obter_plano_via_ipc<W: AsRef<tauri::Webview<tauri::test::MockRuntime>>>(
    webview: &W,
) -> pomodoro_contrato::EstadoDoPlano {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: "obter_plano".into(),
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
            .deserialize::<pomodoro_contrato::EstadoDoPlano>()
            .expect("resposta invalida")
    })
    .expect("comando obter_plano falhou")
}

#[test]
fn obter_plano_reflete_o_estado_inicial_gerenciado() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    let plano = obter_plano_via_ipc(&webview);
    assert_eq!(plano.total_sessoes, plano_inicial().total_sessoes());
    assert_eq!(plano.modo, pomodoro_contrato::ModoDeDuracao::Global);
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
fn estado_inicial_do_plano_e_do_timer_concordam_no_total_de_sessoes() {
    let ciclo = ciclo_inicial();
    let plano = plano_inicial();
    assert_eq!(ciclo.total_sessoes(), plano.total_sessoes());
}

#[test]
fn redimensionar_via_state_gerenciado_atualiza_total_e_preserva_timer_em_curso() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    let estado_antes = obter_estado_via_ipc(&webview);

    {
        let ciclo_state = app.state::<Mutex<CicloEmExecucao>>();
        let plano_state = app.state::<Mutex<PlanoDoCiclo>>();
        let mut ciclo = ciclo_state.lock().expect("lock do ciclo");
        let mut plano = plano_state.lock().expect("lock do plano");
        let nova_quantidade = QuantidadeDeSessoes::nova(8).expect("quantidade valida");
        let (novo_plano, novo_ciclo) = plano.clone().redimensionar(nova_quantidade, *ciclo);
        *plano = novo_plano;
        *ciclo = novo_ciclo;
    }

    let estado_depois = obter_estado_via_ipc(&webview);
    assert_eq!(estado_depois.total_sessoes, 8);
    assert_eq!(estado_depois.prazo_epoca_ms, estado_antes.prazo_epoca_ms);
    assert_eq!(estado_depois.rodando, estado_antes.rodando);
}

#[test]
fn alterar_duracao_global_foco_com_etapa_rodando_recalcula_prazo_a_partir_de_agora() {
    let app = montar_app();
    let webview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("falha ao criar webview de teste");

    {
        let ciclo_state = app.state::<Mutex<CicloEmExecucao>>();
        let relogio = app.state::<RelogioDoSistema>();
        let mut ciclo = ciclo_state.lock().expect("lock do ciclo");
        let agora = Relogio::agora(&*relogio);
        *ciclo = ciclo.alternar_execucao(agora);
    }
    let estado_antes = obter_estado_via_ipc(&webview);
    assert!(
        estado_antes.rodando,
        "etapa deve estar rodando apos alternar_execucao"
    );

    {
        let ciclo_state = app.state::<Mutex<CicloEmExecucao>>();
        let plano_state = app.state::<Mutex<PlanoDoCiclo>>();
        let relogio = app.state::<RelogioDoSistema>();
        let mut ciclo = ciclo_state.lock().expect("lock do ciclo");
        let mut plano = plano_state.lock().expect("lock do plano");
        let agora = Relogio::agora(&*relogio);
        let nova = Duracao::de_minutos_de_foco(50).expect("duracao valida");
        let (novo_plano, novo_ciclo) = plano
            .clone()
            .alterar_duracao_global_foco(nova, *ciclo, agora);
        *plano = novo_plano;
        *ciclo = novo_ciclo;
    }

    let estado_depois = obter_estado_via_ipc(&webview);
    assert_eq!(estado_depois.duracao_total_ms, 50 * 60 * 1000);
    assert_ne!(estado_depois.prazo_epoca_ms, estado_antes.prazo_epoca_ms);
}

#[test]
fn quantidade_de_sessoes_invalida_propaga_como_erro_de_comando_de_plano() {
    let erro = QuantidadeDeSessoes::nova(0).expect_err("0 esta fora do intervalo 1..=24");
    let erro_comando: ErroComandoDePlano = erro.into();
    assert!(matches!(
        erro_comando,
        ErroComandoDePlano::QuantidadeDeSessoesInvalida { recebido: 0 }
    ));
}

#[test]
fn sessao_fora_do_plano_ao_alterar_duracao_individual_propaga_como_erro_de_comando() {
    let ciclo = ciclo_inicial();
    let plano = plano_inicial();
    let total = plano.total_sessoes();
    let sessao_invalida = NumeroDeSessao::de(total + 1);
    let nova = Duracao::de_minutos_de_foco(30).expect("duracao valida");
    let agora = pomodoro_dominio::Instante::desde_epoca_ms(0);

    let resultado = plano.alterar_duracao_individual_foco(sessao_invalida, nova, ciclo, agora);
    let erro = resultado.expect_err("sessao fora do plano deve falhar");
    assert_eq!(
        erro,
        SessaoForaDoPlano::ForaDoIntervalo {
            recebido: total + 1,
            total
        }
    );

    let erro_comando: ErroComandoDePlano = erro.into();
    assert!(matches!(
        erro_comando,
        ErroComandoDePlano::SessaoForaDoPlano { .. }
    ));
}

#[test]
fn alterar_atividade_global_via_state_gerenciado_atualiza_a_atividade() {
    let app = montar_app();
    let plano_state = app.state::<Mutex<PlanoDoCiclo>>();
    let url = UrlDeAtividade::nova("https://exemplo.com").expect("url valida");
    let atividade = Atividade::nova(Some("Foco".to_string()), Some(url));

    {
        let mut plano = plano_state.lock().expect("lock do plano");
        *plano = plano.clone().alterar_atividade_global(atividade.clone());
    }

    let plano = plano_state.lock().expect("lock do plano");
    assert_eq!(plano.atividade_global(), &atividade);
}

#[test]
fn alterar_atividade_individual_fora_do_plano_propaga_como_erro_de_comando() {
    let plano = plano_inicial();
    let total = plano.total_sessoes();
    let sessao_invalida = NumeroDeSessao::de(total + 1);

    let resultado = plano.alterar_atividade_individual(sessao_invalida, Atividade::vazia());
    let erro = resultado.expect_err("sessao fora do plano deve falhar");
    assert_eq!(
        erro,
        SessaoForaDoPlano::ForaDoIntervalo {
            recebido: total + 1,
            total
        }
    );

    let erro_comando: ErroComandoDePlano = erro.into();
    assert!(matches!(
        erro_comando,
        ErroComandoDePlano::SessaoForaDoPlano { .. }
    ));
}

#[test]
fn url_de_atividade_com_esquema_nao_permitido_propaga_como_erro_de_comando() {
    let erro = UrlDeAtividade::nova("javascript:alert(1)").expect_err("esquema nao permitido");
    let erro_comando: ErroComandoDePlano = erro.into();
    assert!(matches!(
        erro_comando,
        ErroComandoDePlano::EsquemaDeUrlNaoPermitido { .. }
    ));
}
