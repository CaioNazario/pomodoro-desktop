use gio::prelude::SettingsExt;
use glib::variant::ToVariant;

use crate::chave::Chave;
use crate::erro::ErroAtalhos;
use crate::fonte::{FonteDeAtalhos, ValorOriginal};

/// I/O real de gsettings via `gio::Settings`. Sem `#[cfg(test)]` cobrindo
/// isto: exige uma sessao GNOME de verdade (D-Bus + dconf), que o runner de
/// CI nao tem — verificado manualmente com `gsettings get`/`gsettings monitor`
/// durante o bloqueio de verdade, mesmo padrao de outras superficies que
/// exigem display (ver `especificar_contrato` no `CLAUDE.md`).
pub struct FonteGio;

impl FonteGio {
    /// `gio::Settings::new` aborta o processo (`g_error`) se o schema nao
    /// existir — por isso o `lookup` antes, nunca construimos `Settings` as
    /// cegas.
    fn abrir(&self, chave: &Chave) -> Result<gio::Settings, ErroAtalhos> {
        let fonte_de_esquemas = gio::SettingsSchemaSource::default().ok_or_else(|| {
            ErroAtalhos::EsquemaIndisponivel {
                esquema: chave.esquema.to_string(),
            }
        })?;
        fonte_de_esquemas
            .lookup(chave.esquema, true)
            .ok_or_else(|| ErroAtalhos::EsquemaIndisponivel {
                esquema: chave.esquema.to_string(),
            })?;
        Ok(gio::Settings::new(chave.esquema))
    }

    /// O "valor desligado" e generico por tipo declarado no schema, nao por
    /// chave: lista de strings vira lista vazia, string vira string vazia.
    /// Cobre os dois tipos que o PRD §7.3 realmente usa.
    fn valor_vazio_para(
        &self,
        configuracoes: &gio::Settings,
        chave: &Chave,
    ) -> Result<glib::Variant, ErroAtalhos> {
        let esquema =
            configuracoes
                .settings_schema()
                .ok_or_else(|| ErroAtalhos::EsquemaIndisponivel {
                    esquema: chave.esquema.to_string(),
                })?;
        if !esquema.has_key(chave.nome) {
            return Err(ErroAtalhos::ChaveInexistente {
                esquema: chave.esquema.to_string(),
                chave: chave.nome.to_string(),
            });
        }
        let tipo = esquema.key(chave.nome).value_type();
        match tipo.as_str() {
            "as" => Ok(Vec::<String>::new().to_variant()),
            "s" => Ok(String::new().to_variant()),
            outro => Err(ErroAtalhos::TipoNaoSuportado {
                chave: chave.nome.to_string(),
                tipo: outro.to_string(),
            }),
        }
    }
}

impl FonteDeAtalhos for FonteGio {
    fn ler(&self, chave: &Chave) -> Result<ValorOriginal, ErroAtalhos> {
        let configuracoes = self.abrir(chave)?;
        Ok(ValorOriginal(
            configuracoes.value(chave.nome).print(true).to_string(),
        ))
    }

    fn desligar(&self, chave: &Chave) -> Result<(), ErroAtalhos> {
        let configuracoes = self.abrir(chave)?;
        let vazio = self.valor_vazio_para(&configuracoes, chave)?;
        configuracoes
            .set_value(chave.nome, &vazio)
            .map_err(|origem| ErroAtalhos::Escrita {
                esquema: chave.esquema.to_string(),
                chave: chave.nome.to_string(),
                origem: origem.to_string(),
            })
    }

    fn religar(&self, chave: &Chave, original: &ValorOriginal) -> Result<(), ErroAtalhos> {
        let configuracoes = self.abrir(chave)?;
        let variante = glib::Variant::parse(None, &original.0).map_err(|origem| {
            ErroAtalhos::ValorOriginalInvalido {
                chave: chave.nome.to_string(),
                origem: origem.to_string(),
            }
        })?;
        configuracoes
            .set_value(chave.nome, &variante)
            .map_err(|origem| ErroAtalhos::Escrita {
                esquema: chave.esquema.to_string(),
                chave: chave.nome.to_string(),
                origem: origem.to_string(),
            })
    }
}
