#![forbid(unsafe_code)]

//! Supressao temporaria de atalhos do GNOME durante o bloqueio (PRD §7.3),
//! com snapshot atomico em disco e restauracao em `Drop`, no shutdown normal
//! e na proxima inicializacao se um snapshot ficou pendente.

mod chave;
mod erro;
mod fonte;
mod fonte_gio;
mod gerenciador;
mod snapshot;

pub use chave::{Chave, CHAVES_SUPRIMIDAS};
pub use erro::ErroAtalhos;
pub use fonte::FonteDeAtalhos;
pub use fonte_gio::FonteGio;
pub use gerenciador::GerenciadorDeAtalhos;
