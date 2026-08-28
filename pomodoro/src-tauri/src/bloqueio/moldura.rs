use pomodoro_contrato::EstadoDaTela;
use pomodoro_dominio::Atividade;
use serde::Serialize;

/// O que a moldura de bloqueio (`public/bloqueio.js`) precisa pra desenhar
/// `Pausa · {rotulo}`, o relogio e a barra de progresso — injetado como
/// `initialization_script` na criacao da janela, nao via IPC (a janela de
/// bloqueio nao tem capability nenhuma). O prazo nao muda durante o
/// bloqueio (reiniciar nao existe aqui), entao um unico push no momento da
/// criacao basta; o JS deriva `mm:ss` localmente via `requestAnimationFrame`,
/// mesmo padrao de `useEstadoDoTimer`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DadosDaMoldura {
    rotulo: String,
    prazo_epoca_ms: Option<i64>,
    duracao_total_ms: i64,
}

pub(super) fn script_de_inicializacao(atividade: &Atividade, tela: &EstadoDaTela) -> String {
    let dados = DadosDaMoldura {
        rotulo: atividade.rotulo(),
        prazo_epoca_ms: tela.prazo_epoca_ms,
        duracao_total_ms: tela.duracao_total_ms,
    };
    let json = serde_json::to_string(&dados).unwrap_or_else(|_| "null".to_string());
    format!("window.__moldura = {json};")
}
