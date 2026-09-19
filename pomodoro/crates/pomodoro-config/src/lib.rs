#![forbid(unsafe_code)]

//! Persistencia em TOML versionado, com escrita atomica. Nao conhece Tauri
//! nem `app.path()` — o caminho do arquivo e resolvido por quem chama
//! (`src-tauri`), nunca aceito do frontend.

mod armazenamento;
mod configuracao_toml;
mod escrita_atomica;

pub use armazenamento::{Armazenamento, ErroDeEscrita};
use pomodoro_dominio::PlanoDoCiclo;

/// Tudo que persiste entre execucoes neste arquivo: o plano de duracoes
/// (§3 do PRD), a flag de autostart e a posicao do widget flutuante (§9).
/// O historico diario vive em `pomodoro-historico` (SQLite), nao aqui — ver
/// `Armazenamento::historico_legado` pra migracao de arquivos antigos que
/// ainda o tinham embutido. O estado do timer e o menu aberto tambem nao
/// entram aqui de proposito — nao sao persistidos.
pub struct Configuracao {
    pub plano: PlanoDoCiclo,
    pub iniciar_automaticamente: bool,
    pub posicao_do_widget: Option<(i32, i32)>,
}
