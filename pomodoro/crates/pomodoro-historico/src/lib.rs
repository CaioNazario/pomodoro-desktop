#![forbid(unsafe_code)]

//! Persistencia do historico diario em SQLite. Nao conhece Tauri nem
//! `app.path()` — o caminho do banco e resolvido por quem chama
//! (`src-tauri`), nunca aceito do frontend.
//!
//! Ao contrario do `pomodoro-config` (que sobrescreve o arquivo inteiro a
//! cada `salvar`), este crate nunca apaga uma linha: cada dia e gravado via
//! `UPSERT` e permanece no banco indefinidamente, mesmo que o
//! `HistoricoDiario` em memoria (limitado a 90 dias, ver
//! `pomodoro_dominio::HistoricoDiario`) ja o tenha descartado do anel. E o
//! banco, nao o tipo em memoria, que sustenta uma futura tela de
//! Estatisticas com o acumulado de verdade.

mod armazenamento;

pub use armazenamento::{Armazenamento, ErroDeBanco};
