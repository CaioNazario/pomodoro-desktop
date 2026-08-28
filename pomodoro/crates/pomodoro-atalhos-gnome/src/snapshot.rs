use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::chave::{Chave, CHAVES_SUPRIMIDAS};
use crate::erro::ErroAtalhos;
use crate::fonte::ValorOriginal;

#[derive(Debug, Serialize, Deserialize)]
struct ItemPersistido {
    esquema: String,
    chave: String,
    valor: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotToml {
    itens: Vec<ItemPersistido>,
}

pub(crate) struct Snapshot {
    pub(crate) itens: Vec<(Chave, ValorOriginal)>,
}

impl Snapshot {
    pub(crate) fn escrever(
        caminho: &Path,
        itens: &[(Chave, ValorOriginal)],
    ) -> Result<(), ErroAtalhos> {
        let bruto = SnapshotToml {
            itens: itens
                .iter()
                .map(|(chave, valor)| ItemPersistido {
                    esquema: chave.esquema.to_string(),
                    chave: chave.nome.to_string(),
                    valor: valor.0.clone(),
                })
                .collect(),
        };
        let conteudo =
            toml::to_string(&bruto).expect("serializar snapshot de atalhos nao deveria falhar");
        escrever_atomico(caminho, &conteudo).map_err(|origem| ErroAtalhos::EscritaDoSnapshot {
            caminho: caminho.display().to_string(),
            origem,
        })
    }

    /// `None` quando nao ha snapshot pendente ou quando o arquivo esta
    /// corrompido — nesse caso e movido para `.corrompido` e tratado como
    /// inexistente, nunca panic. Itens que nao correspondem a nenhuma chave
    /// conhecida (versao antiga, schema mudou) sao descartados em silencio.
    pub(crate) fn ler(caminho: &Path) -> Option<Self> {
        let conteudo = std::fs::read_to_string(caminho).ok()?;
        let bruto: SnapshotToml = match toml::from_str(&conteudo) {
            Ok(bruto) => bruto,
            Err(_) => {
                let _ = std::fs::rename(caminho, caminho.with_extension("corrompido"));
                return None;
            }
        };
        let itens = bruto
            .itens
            .into_iter()
            .filter_map(|item| {
                let chave = CHAVES_SUPRIMIDAS
                    .iter()
                    .find(|chave| chave.esquema == item.esquema && chave.nome == item.chave)?;
                Some((*chave, ValorOriginal(item.valor)))
            })
            .collect();
        Some(Self { itens })
    }

    pub(crate) fn apagar(caminho: &Path) -> Result<(), ErroAtalhos> {
        match std::fs::remove_file(caminho) {
            Ok(()) => Ok(()),
            Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(origem) => Err(ErroAtalhos::EscritaDoSnapshot {
                caminho: caminho.display().to_string(),
                origem,
            }),
        }
    }
}

/// Temporario no mesmo diretorio -> `fsync` -> `rename`, mesmo padrao de
/// `pomodoro-config` — nao compartilhado entre crates porque e pequeno e
/// cada crate de I/O tem seu proprio ponto de escrita atomica.
fn escrever_atomico(caminho: &Path, conteudo: &str) -> std::io::Result<()> {
    let diretorio = caminho.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(diretorio)?;

    let mut temporario = tempfile::NamedTempFile::new_in(diretorio)?;
    temporario.write_all(conteudo.as_bytes())?;
    temporario.as_file().sync_all()?;
    temporario.persist(caminho).map_err(|erro| erro.error)?;
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn primeira_chave() -> Chave {
        CHAVES_SUPRIMIDAS[0]
    }

    #[test]
    fn escreve_e_le_de_volta_os_mesmos_itens() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("snapshot.toml");
        let itens = vec![(primeira_chave(), ValorOriginal("['<Alt>F4']".to_string()))];

        Snapshot::escrever(&caminho, &itens).expect("escrita do snapshot");
        let lido = Snapshot::ler(&caminho).expect("snapshot deveria existir");

        assert_eq!(lido.itens, itens);
    }

    #[test]
    fn ler_sem_arquivo_devolve_none() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("nunca-existiu.toml");

        assert!(Snapshot::ler(&caminho).is_none());
    }

    #[test]
    fn arquivo_corrompido_e_movido_e_tratado_como_inexistente() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("snapshot.toml");
        std::fs::write(&caminho, "isto nao e toml valido {{{").expect("escrever lixo");

        assert!(Snapshot::ler(&caminho).is_none());
        assert!(!caminho.exists());
        assert!(caminho.with_extension("corrompido").exists());
    }

    #[test]
    fn item_de_chave_desconhecida_e_descartado_em_silencio() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("snapshot.toml");
        let bruto = SnapshotToml {
            itens: vec![ItemPersistido {
                esquema: "org.gnome.nao.existe.mais".to_string(),
                chave: "fantasma".to_string(),
                valor: "x".to_string(),
            }],
        };
        std::fs::write(&caminho, toml::to_string(&bruto).unwrap()).expect("escrever snapshot");

        let lido = Snapshot::ler(&caminho).expect("arquivo valido, so item desconhecido");

        assert!(lido.itens.is_empty());
    }

    #[test]
    fn apagar_arquivo_inexistente_nao_e_erro() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("nunca-existiu.toml");

        Snapshot::apagar(&caminho).expect("apagar inexistente e no-op");
    }
}
