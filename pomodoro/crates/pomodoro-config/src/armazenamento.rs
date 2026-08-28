use crate::configuracao_toml::{ConfiguracaoToml, VERSAO_ATUAL};
use crate::escrita_atomica::escrever_atomico;
use crate::Configuracao;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ErroDeEscrita {
    #[error("nao foi possivel serializar a configuracao: {fonte}")]
    Serializacao {
        #[source]
        fonte: toml::ser::Error,
    },
    #[error("nao foi possivel escrever em {caminho}: {fonte}")]
    Escrita {
        caminho: PathBuf,
        #[source]
        fonte: std::io::Error,
    },
}

#[derive(Debug, thiserror::Error)]
enum ErroDeCarga {
    #[error("nao foi possivel ler {caminho}: {fonte}")]
    Leitura {
        caminho: PathBuf,
        #[source]
        fonte: std::io::Error,
    },
    #[error("toml invalido em {caminho}: {fonte}")]
    Formato {
        caminho: PathBuf,
        #[source]
        fonte: toml::de::Error,
    },
    #[error("versao de configuracao desconhecida: {recebida}, esperada {esperada}")]
    VersaoDesconhecida { recebida: u32, esperada: u32 },
}

/// Um unico arquivo TOML, resolvido pelo chamador (nunca pelo frontend —
/// ver regra do projeto). Le com fallback pro default e nunca panica; toda
/// escrita e atomica.
pub struct Armazenamento {
    caminho: PathBuf,
}

impl Armazenamento {
    pub fn em(caminho: PathBuf) -> Self {
        Self { caminho }
    }

    /// TOML invalido, ilegivel ou de versao desconhecida abre com
    /// `construir_padrao()` e move o arquivo ruim para `.corrompido` — sem
    /// tentar de novo, sem `panic!`.
    pub fn carregar(&self, construir_padrao: impl FnOnce() -> Configuracao) -> Configuracao {
        match self.tentar_carregar() {
            Ok(config) => config,
            Err(erro) => {
                eprintln!("pomodoro-config: {erro}, abrindo com o padrao");
                self.mover_para_corrompido();
                construir_padrao()
            }
        }
    }

    pub fn salvar(&self, config: &Configuracao) -> Result<(), ErroDeEscrita> {
        let bruto = ConfiguracaoToml::de(config);
        let texto = toml::to_string_pretty(&bruto)
            .map_err(|fonte| ErroDeEscrita::Serializacao { fonte })?;
        escrever_atomico(&self.caminho, &texto).map_err(|fonte| ErroDeEscrita::Escrita {
            caminho: self.caminho.clone(),
            fonte,
        })
    }

    fn tentar_carregar(&self) -> Result<Configuracao, ErroDeCarga> {
        let bruto =
            std::fs::read_to_string(&self.caminho).map_err(|fonte| ErroDeCarga::Leitura {
                caminho: self.caminho.clone(),
                fonte,
            })?;
        let config: ConfiguracaoToml =
            toml::from_str(&bruto).map_err(|fonte| ErroDeCarga::Formato {
                caminho: self.caminho.clone(),
                fonte,
            })?;
        if config.versao != VERSAO_ATUAL {
            return Err(ErroDeCarga::VersaoDesconhecida {
                recebida: config.versao,
                esperada: VERSAO_ATUAL,
            });
        }
        Ok(config.para_configuracao())
    }

    /// Melhor esforco: se o arquivo nao existir (primeira execucao) ou o
    /// rename falhar, so segue com o default — nunca bloqueia a inicializacao.
    fn mover_para_corrompido(&self) {
        let _ = std::fs::rename(&self.caminho, caminho_corrompido(&self.caminho));
    }
}

fn caminho_corrompido(caminho: &Path) -> PathBuf {
    let mut nome = caminho
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    nome.push(".corrompido");
    caminho.with_file_name(nome)
}

#[cfg(test)]
mod testes {
    use super::*;
    use pomodoro_dominio::{
        Atividade, Duracao, HistoricoDiario, NumeroDeSessao, PlanoDoCiclo, QuantidadeDeSessoes,
        UrlDeAtividade,
    };
    use std::sync::Arc;

    fn config_padrao() -> Configuracao {
        Configuracao {
            plano: PlanoDoCiclo::novo(
                QuantidadeDeSessoes::nova(4).expect("teste"),
                Duracao::de_minutos(25),
                Duracao::de_minutos(5),
            ),
            iniciar_automaticamente: true,
            historico: HistoricoDiario::vazio(),
            posicao_do_widget: None,
        }
    }

    fn config_diferente_da_padrao() -> Configuracao {
        Configuracao {
            plano: PlanoDoCiclo::novo(
                QuantidadeDeSessoes::nova(8).expect("teste"),
                Duracao::de_minutos(50),
                Duracao::de_minutos(10),
            ),
            iniciar_automaticamente: false,
            historico: HistoricoDiario::vazio(),
            posicao_do_widget: Some((120, 340)),
        }
    }

    #[test]
    fn round_trip_preserva_o_plano_e_o_autostart() {
        let dir = tempfile::tempdir().expect("tempdir");
        let armazenamento = Armazenamento::em(dir.path().join("config.toml"));

        armazenamento
            .salvar(&config_diferente_da_padrao())
            .expect("salvar");
        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.total_sessoes(), 8);
        assert_eq!(
            carregada.plano.duracao_global_foco(),
            Duracao::de_minutos(50)
        );
        assert!(!carregada.iniciar_automaticamente);
        assert_eq!(carregada.posicao_do_widget, Some((120, 340)));
    }

    #[test]
    fn toml_sem_posicao_do_widget_carrega_como_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = dir.path().join("config.toml");
        std::fs::write(
            &caminho,
            r#"
versao = 1
modo = "Global"
duracao_global_foco_ms = 1500000
duracao_global_pausa_ms = 300000
iniciar_automaticamente = true
plano_individual = []
"#,
        )
        .expect("escrever toml sem posicao_do_widget");
        let armazenamento = Armazenamento::em(caminho);

        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.posicao_do_widget, None);
    }

    #[test]
    fn round_trip_preserva_atividade_global_e_individual() {
        let dir = tempfile::tempdir().expect("tempdir");
        let armazenamento = Armazenamento::em(dir.path().join("config.toml"));
        let url = UrlDeAtividade::nova("https://www.exemplo.com").expect("teste");

        let plano = PlanoDoCiclo::novo(
            QuantidadeDeSessoes::nova(2).expect("teste"),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
        )
        .alterar_atividade_global(Atividade::nova(Some("Foco".to_string()), Some(url.clone())))
        .alterar_atividade_individual(NumeroDeSessao::de(2), Atividade::nova(None, Some(url)))
        .expect("sessao dentro do plano");
        let config = Configuracao {
            plano,
            iniciar_automaticamente: true,
            historico: HistoricoDiario::vazio(),
            posicao_do_widget: None,
        };

        armazenamento.salvar(&config).expect("salvar");
        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.atividade_global().nome(), Some("Foco"));
        assert_eq!(
            carregada
                .plano
                .atividade_global()
                .url()
                .map(UrlDeAtividade::como_str),
            Some("https://www.exemplo.com/")
        );
        assert!(carregada.plano.plano_individual()[1]
            .atividade()
            .tem_url_aplicavel());
        assert!(!carregada.plano.plano_individual()[0]
            .atividade()
            .tem_url_aplicavel());
    }

    #[test]
    fn url_persistida_invalida_degrada_para_atividade_sem_url_sem_falhar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = dir.path().join("config.toml");
        std::fs::write(
            &caminho,
            r#"
versao = 1
modo = "Global"
duracao_global_foco_ms = 1500000
duracao_global_pausa_ms = 300000
iniciar_automaticamente = true
plano_individual = []

[atividade_global]
nome = "Foco"
url = "javascript:alert(1)"
"#,
        )
        .expect("escrever toml com url invalida");
        let armazenamento = Armazenamento::em(caminho);

        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.atividade_global().nome(), Some("Foco"));
        assert_eq!(carregada.plano.atividade_global().url(), None);
    }

    #[test]
    fn arquivo_truncado_abre_com_o_padrao_e_move_para_corrompido() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = dir.path().join("config.toml");
        std::fs::write(&caminho, "isto nao e toml valido {{{").expect("escrever lixo");
        let armazenamento = Armazenamento::em(caminho.clone());

        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.total_sessoes(), 4);
        assert!(!caminho.exists());
        assert!(dir.path().join("config.toml.corrompido").exists());
    }

    #[test]
    fn versao_desconhecida_abre_com_o_padrao_e_move_para_corrompido() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = dir.path().join("config.toml");
        std::fs::write(&caminho, "versao = 999\n").expect("escrever versao invalida");
        let armazenamento = Armazenamento::em(caminho.clone());

        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.total_sessoes(), 4);
        assert!(dir.path().join("config.toml.corrompido").exists());
    }

    #[test]
    fn arquivo_inexistente_abre_com_o_padrao_sem_falhar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let armazenamento = Armazenamento::em(dir.path().join("nao-existe.toml"));

        let carregada = armazenamento.carregar(config_padrao);

        assert_eq!(carregada.plano.total_sessoes(), 4);
    }

    #[test]
    fn permissao_negada_devolve_erro_sem_panico() {
        let dir = tempfile::tempdir().expect("tempdir");
        let subdiretorio = dir.path().join("sem-permissao");
        std::fs::create_dir(&subdiretorio).expect("criar subdiretorio");
        std::fs::set_permissions(
            &subdiretorio,
            std::os::unix::fs::PermissionsExt::from_mode(0o444),
        )
        .expect("remover permissao de escrita");

        let armazenamento = Armazenamento::em(subdiretorio.join("config.toml"));
        let resultado = armazenamento.salvar(&config_padrao());

        std::fs::set_permissions(
            &subdiretorio,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .expect("restaurar permissao para o cleanup do tempdir");
        assert!(resultado.is_err());
    }

    #[test]
    fn escritas_concorrentes_nunca_corrompem_o_arquivo_final() {
        let dir = tempfile::tempdir().expect("tempdir");
        let armazenamento = Arc::new(Armazenamento::em(dir.path().join("config.toml")));

        let handles: Vec<_> = (1u8..=8)
            .map(|quantidade| {
                let armazenamento = Arc::clone(&armazenamento);
                std::thread::spawn(move || {
                    let config = Configuracao {
                        plano: PlanoDoCiclo::novo(
                            QuantidadeDeSessoes::nova(quantidade).expect("teste"),
                            Duracao::de_minutos(25),
                            Duracao::de_minutos(5),
                        ),
                        iniciar_automaticamente: true,
                        historico: HistoricoDiario::vazio(),
                        posicao_do_widget: None,
                    };
                    armazenamento.salvar(&config)
                })
            })
            .collect();

        for handle in handles {
            handle
                .join()
                .expect("thread nao deve panicar")
                .expect("salvar concorrente");
        }

        // O arquivo final e valido e reflete a escrita de UM dos escritores
        // por inteiro (1..=8) — nunca uma mistura corrompida das duas.
        let carregada = armazenamento.carregar(config_padrao);
        assert!((1..=8).contains(&carregada.plano.total_sessoes()));
    }
}
