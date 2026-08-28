use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErroAtalhos {
    #[error("falha ao escrever snapshot de atalhos em {caminho}: {origem}")]
    EscritaDoSnapshot {
        caminho: String,
        #[source]
        origem: std::io::Error,
    },
    #[error("esquema gsettings {esquema} indisponivel nesta sessao")]
    EsquemaIndisponivel { esquema: String },
    #[error("chave {chave} nao existe no esquema {esquema}")]
    ChaveInexistente { esquema: String, chave: String },
    #[error("tipo de valor nao suportado para a chave {chave}: esperado string ou lista de strings, recebido {tipo}")]
    TipoNaoSuportado { chave: String, tipo: String },
    #[error("valor original invalido para a chave {chave}: {origem}")]
    ValorOriginalInvalido { chave: String, origem: String },
    #[error("falha ao escrever a chave {chave} do esquema {esquema}: {origem}")]
    Escrita {
        esquema: String,
        chave: String,
        origem: String,
    },
}
