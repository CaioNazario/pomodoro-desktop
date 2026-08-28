use std::io::Write;
use std::path::Path;

/// Temporario no mesmo diretorio (nome unico, via `tempfile`) -> `fsync` ->
/// `rename`. O nome unico e o que torna escritas concorrentes seguras: cada
/// escritor tem seu proprio temporario, e so o `rename` final decide quem
/// "ganha" — nunca ha interleaving de conteudo.
pub(crate) fn escrever_atomico(caminho: &Path, conteudo: &str) -> std::io::Result<()> {
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

    #[test]
    fn escreve_e_le_de_volta_o_mesmo_conteudo() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("config.toml");

        escrever_atomico(&caminho, "conteudo = 1").expect("escrita atomica");

        assert_eq!(
            std::fs::read_to_string(&caminho).expect("leitura"),
            "conteudo = 1"
        );
    }

    #[test]
    fn nao_deixa_arquivo_temporario_para_tras() {
        let dir = tempfile::tempdir().expect("tempdir de teste");
        let caminho = dir.path().join("config.toml");

        escrever_atomico(&caminho, "x = 1").expect("escrita atomica");

        let entradas: Vec<_> = std::fs::read_dir(dir.path())
            .expect("listar dir")
            .map(|e| e.expect("entrada").file_name())
            .collect();
        assert_eq!(entradas, vec![std::ffi::OsString::from("config.toml")]);
    }
}
