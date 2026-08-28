#![forbid(unsafe_code)]
// `forbid`, nao `deny`: com `deny` bastaria um `#[allow(clippy::disallowed_methods)]`
// em cima da funcao para furar a regra. `forbid` nao pode ser sobrescrito.
#![forbid(clippy::disallowed_methods)]

//! Nucleo do pomodoro. Nao conhece Tauri, sistema de arquivos nem relogio do SO.

mod atividade;
mod ciclo;
mod ciclo_em_execucao;
mod contadores_do_dia;
mod data;
mod destino_de_carregamento;
mod duracao_de_etapa;
mod etapa;
mod historico_diario;
mod modo_de_duracao;
mod plano_do_ciclo;
mod quantidade_de_sessoes;
mod relogio;
#[cfg(test)]
mod relogio_fake;
mod sessao;
mod tempo;
mod timer;
mod url_de_atividade;
mod video_incorporado;

pub use atividade::Atividade;
pub use ciclo::NumeroDeSessao;
pub use ciclo_em_execucao::CicloEmExecucao;
pub use contadores_do_dia::ContadoresDoDia;
pub use data::Data;
pub use destino_de_carregamento::DestinoDeCarregamento;
pub use duracao_de_etapa::DuracaoDeEtapaInvalida;
pub use etapa::Etapa;
pub use historico_diario::HistoricoDiario;
pub use modo_de_duracao::ModoDeDuracao;
pub use plano_do_ciclo::{PlanoDoCiclo, SessaoForaDoPlano};
pub use quantidade_de_sessoes::{QuantidadeDeSessoes, QuantidadeDeSessoesInvalida};
pub use relogio::Relogio;
pub use sessao::Sessao;
pub use tempo::{Duracao, DuracaoInvalida, Instante};
pub use timer::EstadoTimer;
pub use url_de_atividade::{UrlDeAtividade, UrlDeAtividadeInvalida};
