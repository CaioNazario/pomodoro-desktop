use pomodoro_dominio::{ContadoresDoDia, Data, Duracao, HistoricoDiario};
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ErroDeBanco {
    #[error("nao foi possivel abrir o banco em {caminho}: {fonte}")]
    Abertura {
        caminho: PathBuf,
        #[source]
        fonte: rusqlite::Error,
    },
    #[error("nao foi possivel preparar o schema do banco: {fonte}")]
    Schema {
        #[source]
        fonte: rusqlite::Error,
    },
    #[error("nao foi possivel salvar o historico: {fonte}")]
    Escrita {
        #[source]
        fonte: rusqlite::Error,
    },
}

/// Um banco SQLite com uma unica tabela (`dias`), resolvido pelo chamador
/// (nunca pelo frontend). Le com fallback pro historico vazio e nunca
/// panica; toda escrita roda dentro de uma transacao.
pub struct Armazenamento {
    conexao: Connection,
}

impl Armazenamento {
    pub fn abrir(caminho: PathBuf) -> Result<Self, ErroDeBanco> {
        let conexao =
            Connection::open(&caminho).map_err(|fonte| ErroDeBanco::Abertura { caminho, fonte })?;
        conexao
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS dias (
                    data TEXT PRIMARY KEY,
                    sessoes_concluidas INTEGER NOT NULL,
                    tempo_de_foco_ms INTEGER NOT NULL,
                    pausas_interrompidas INTEGER NOT NULL
                );",
            )
            .map_err(|fonte| ErroDeBanco::Schema { fonte })?;
        Ok(Self { conexao })
    }

    pub fn esta_vazio(&self) -> bool {
        self.conexao
            .query_row("SELECT COUNT(*) FROM dias", [], |linha| {
                linha.get::<_, i64>(0)
            })
            .map(|contagem| contagem == 0)
            .unwrap_or(true)
    }

    /// Le os dias mais recentes (ate a capacidade do anel em memoria, que
    /// `HistoricoDiario::reconstruir` ja aplica) — o banco em si pode ter
    /// mais linhas que isso, guardadas pra uso futuro (Estatisticas).
    pub fn carregar(&self) -> HistoricoDiario {
        let dias = self.tentar_carregar().unwrap_or_default();
        HistoricoDiario::reconstruir(dias)
    }

    fn tentar_carregar(&self) -> rusqlite::Result<Vec<ContadoresDoDia>> {
        let mut consulta = self.conexao.prepare(
            "SELECT data, sessoes_concluidas, tempo_de_foco_ms, pausas_interrompidas
             FROM dias ORDER BY data ASC",
        )?;
        let linhas = consulta.query_map([], |linha| {
            let data_texto: String = linha.get(0)?;
            let tempo_de_foco_ms: i64 = linha.get(2)?;
            Ok(ContadoresDoDia::reconstruir(
                parse_data(&data_texto),
                linha.get(1)?,
                Duracao::de_ms(tempo_de_foco_ms).unwrap_or(Duracao::ZERO),
                linha.get(3)?,
            ))
        })?;
        linhas.collect()
    }

    /// Grava todo o anel em memoria de volta no banco (UPSERT por dia) numa
    /// unica transacao — nunca deleta linha, mesmo que o dia tenha saido do
    /// anel de 90 dias.
    pub fn salvar(&mut self, historico: &HistoricoDiario) -> Result<(), ErroDeBanco> {
        let transacao = self
            .conexao
            .transaction()
            .map_err(|fonte| ErroDeBanco::Escrita { fonte })?;
        for dia in historico.dias() {
            transacao
                .execute(
                    "INSERT INTO dias (data, sessoes_concluidas, tempo_de_foco_ms, pausas_interrompidas)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(data) DO UPDATE SET
                        sessoes_concluidas = excluded.sessoes_concluidas,
                        tempo_de_foco_ms = excluded.tempo_de_foco_ms,
                        pausas_interrompidas = excluded.pausas_interrompidas",
                    params![
                        formatar_data(dia.dia_de_referencia()),
                        dia.sessoes_concluidas(),
                        dia.tempo_de_foco().em_ms(),
                        dia.pausas_interrompidas(),
                    ],
                )
                .map_err(|fonte| ErroDeBanco::Escrita { fonte })?;
        }
        transacao
            .commit()
            .map_err(|fonte| ErroDeBanco::Escrita { fonte })
    }

    /// Importa o historico embutido no `config.toml` de versoes antigas —
    /// chamado uma unica vez pelo `src-tauri`, so quando o banco ainda nao
    /// existia antes deste `abrir`. Nao faz nada se a lista vier vazia.
    pub fn migrar_historico_legado(
        &mut self,
        dias: Vec<ContadoresDoDia>,
    ) -> Result<(), ErroDeBanco> {
        if dias.is_empty() {
            return Ok(());
        }
        self.salvar(&HistoricoDiario::reconstruir(dias))
    }
}

fn formatar_data(data: Data) -> String {
    format!("{:04}-{:02}-{:02}", data.ano(), data.mes(), data.dia())
}

/// Formato invalido (banco editado a mao) degrada pra epoca — mesma
/// filosofia de `Duracao::de_ms(..).unwrap_or(Duracao::ZERO)`: nunca panica.
fn parse_data(texto: &str) -> Data {
    let mut partes = texto.splitn(3, '-');
    let ano = partes.next().and_then(|p| p.parse().ok()).unwrap_or(1970);
    let mes = partes.next().and_then(|p| p.parse().ok()).unwrap_or(1);
    let dia = partes.next().and_then(|p| p.parse().ok()).unwrap_or(1);
    Data::de(ano, mes, dia)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn dia_de_teste(ano: u16, mes: u8, dia: u8, sessoes: u32) -> ContadoresDoDia {
        ContadoresDoDia::reconstruir(Data::de(ano, mes, dia), sessoes, Duracao::de_minutos(25), 0)
    }

    #[test]
    fn banco_novo_carrega_historico_vazio() {
        let dir = tempfile::tempdir().expect("tempdir");
        let banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");

        assert!(banco.esta_vazio());
        assert_eq!(banco.carregar(), HistoricoDiario::vazio());
    }

    #[test]
    fn round_trip_preserva_os_dias() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");
        let historico = HistoricoDiario::reconstruir(vec![
            dia_de_teste(2026, 8, 22, 1),
            dia_de_teste(2026, 8, 23, 2),
        ]);

        banco.salvar(&historico).expect("salvar");
        let carregado = banco.carregar();

        assert_eq!(carregado, historico);
    }

    #[test]
    fn salvar_de_novo_atualiza_o_dia_existente_em_vez_de_duplicar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");
        let dia = Data::de(2026, 8, 23);

        banco
            .salvar(&HistoricoDiario::reconstruir(vec![dia_de_teste(
                2026, 8, 23, 1,
            )]))
            .expect("salvar 1");
        banco
            .salvar(&HistoricoDiario::reconstruir(vec![dia_de_teste(
                2026, 8, 23, 5,
            )]))
            .expect("salvar 2");

        let carregado = banco.carregar();
        assert_eq!(carregado.dias().count(), 1);
        assert_eq!(carregado.dia_atual(dia).sessoes_concluidas(), 5);
    }

    #[test]
    fn reabrir_o_mesmo_arquivo_preserva_os_dados() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = dir.path().join("historico.db");

        Armazenamento::abrir(caminho.clone())
            .expect("abrir 1")
            .salvar(&HistoricoDiario::reconstruir(vec![dia_de_teste(
                2026, 8, 23, 3,
            )]))
            .expect("salvar");

        let reaberto = Armazenamento::abrir(caminho).expect("abrir 2");
        assert!(!reaberto.esta_vazio());
        assert_eq!(
            reaberto
                .carregar()
                .dia_atual(Data::de(2026, 8, 23))
                .sessoes_concluidas(),
            3
        );
    }

    #[test]
    fn migrar_historico_legado_vazio_nao_cria_linha_nenhuma() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");

        banco.migrar_historico_legado(vec![]).expect("migrar vazio");

        assert!(banco.esta_vazio());
    }

    #[test]
    fn migrar_historico_legado_importa_os_dias_uma_vez() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");

        banco
            .migrar_historico_legado(vec![dia_de_teste(2026, 8, 20, 4)])
            .expect("migrar");

        assert_eq!(
            banco
                .carregar()
                .dia_atual(Data::de(2026, 8, 20))
                .sessoes_concluidas(),
            4
        );
    }

    #[test]
    fn banco_acumula_dias_alem_da_capacidade_do_anel_em_memoria() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut banco = Armazenamento::abrir(dir.path().join("historico.db")).expect("abrir");

        for dia in 1..=95u8 {
            let historico = HistoricoDiario::reconstruir(vec![dia_de_teste(2026, 1, dia, 1)]);
            banco.salvar(&historico).expect("salvar");
        }

        let contagem: i64 = banco
            .conexao
            .query_row("SELECT COUNT(*) FROM dias", [], |linha| linha.get(0))
            .expect("contar linhas");
        assert_eq!(contagem, 95);
    }
}
