use crate::{Data, Instante};

/// Fonte de tempo do dominio.
///
/// Nenhuma funcao daqui chama o relogio do SO: o `clippy.toml` deste crate proibe
/// `SystemTime::now` e `Instant::now`. Quem precisa da hora recebe este trait por
/// injecao ou o `agora` por parametro.
pub trait Relogio: Send + Sync {
    fn agora(&self) -> Instante;

    /// Data de calendario "de hoje" no fuso local da maquina. Independente
    /// de `agora()`: quem resolve o fuso e a implementacao real
    /// (`RelogioDoSistema`, em `src-tauri`), nunca o dominio.
    fn hoje(&self) -> Data;
}
