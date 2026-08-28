#![forbid(unsafe_code)]

//! Persistencia em TOML versionado, com escrita atomica. Nao conhece Tauri
//! nem `app.path()` — o caminho do arquivo e resolvido por quem chama
//! (`src-tauri`), nunca aceito do frontend.

mod armazenamento;
mod configuracao_toml;
mod escrita_atomica;

pub use armazenamento::{Armazenamento, ErroDeEscrita};
use pomodoro_dominio::{HistoricoDiario, PlanoDoCiclo};

/// Tudo que persiste entre execucoes: o plano de duracoes (§3 do PRD), a
/// flag de autostart, o historico de 90 dias e a posicao do widget
/// flutuante (§9). O estado do timer e o menu aberto nao entram aqui de
/// proposito — nao sao persistidos.
pub struct Configuracao {
    pub plano: PlanoDoCiclo,
    pub iniciar_automaticamente: bool,
    pub historico: HistoricoDiario,
    pub posicao_do_widget: Option<(i32, i32)>,
}
