use crate::chave::Chave;
use crate::erro::ErroAtalhos;

/// O valor de uma chave gsettings antes da supressao, opaco para quem chama —
/// so a `FonteDeAtalhos` que o produziu sabe decodifica-lo de volta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValorOriginal(pub(crate) String);

/// I/O de gsettings atras de trait — o gerenciador nunca fala com `gio`
/// diretamente, so com isto. Real em [`crate::fonte_gio::FonteGio`], fake so
/// em teste.
pub trait FonteDeAtalhos {
    fn ler(&self, chave: &Chave) -> Result<ValorOriginal, ErroAtalhos>;
    fn desligar(&self, chave: &Chave) -> Result<(), ErroAtalhos>;
    fn religar(&self, chave: &Chave, original: &ValorOriginal) -> Result<(), ErroAtalhos>;
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    type ParDeChave = (&'static str, &'static str);

    /// Fake de `FonteDeAtalhos`: um mapa em memoria simulando o dconf, mais
    /// listas configuraveis de chaves que devem falhar ao ler/desligar/religar
    /// — para testar degradacao silenciosa sem depender de sessao real.
    #[derive(Clone)]
    pub(crate) struct FonteFake {
        valores: Rc<RefCell<HashMap<ParDeChave, String>>>,
        falha_leitura: Rc<RefCell<HashSet<ParDeChave>>>,
        falha_escrita: Rc<RefCell<HashSet<ParDeChave>>>,
    }

    impl FonteFake {
        pub(crate) fn nova() -> Self {
            let valores = crate::chave::CHAVES_SUPRIMIDAS
                .iter()
                .map(|chave| ((chave.esquema, chave.nome), "original".to_string()))
                .collect();
            Self {
                valores: Rc::new(RefCell::new(valores)),
                falha_leitura: Rc::new(RefCell::new(HashSet::new())),
                falha_escrita: Rc::new(RefCell::new(HashSet::new())),
            }
        }

        pub(crate) fn valor_atual(&self, chave: &Chave) -> String {
            self.valores.borrow()[&(chave.esquema, chave.nome)].clone()
        }

        pub(crate) fn falhar_leitura_em(&self, chave: Chave) {
            self.falha_leitura
                .borrow_mut()
                .insert((chave.esquema, chave.nome));
        }

        pub(crate) fn falhar_escrita_em(&self, chave: Chave) {
            self.falha_escrita
                .borrow_mut()
                .insert((chave.esquema, chave.nome));
        }

        fn erro_indisponivel(chave: &Chave) -> ErroAtalhos {
            ErroAtalhos::EsquemaIndisponivel {
                esquema: chave.esquema.to_string(),
            }
        }
    }

    impl FonteDeAtalhos for FonteFake {
        fn ler(&self, chave: &Chave) -> Result<ValorOriginal, ErroAtalhos> {
            if self
                .falha_leitura
                .borrow()
                .contains(&(chave.esquema, chave.nome))
            {
                return Err(Self::erro_indisponivel(chave));
            }
            Ok(ValorOriginal(self.valor_atual(chave)))
        }

        fn desligar(&self, chave: &Chave) -> Result<(), ErroAtalhos> {
            if self
                .falha_escrita
                .borrow()
                .contains(&(chave.esquema, chave.nome))
            {
                return Err(Self::erro_indisponivel(chave));
            }
            self.valores
                .borrow_mut()
                .insert((chave.esquema, chave.nome), "desligado".to_string());
            Ok(())
        }

        fn religar(&self, chave: &Chave, original: &ValorOriginal) -> Result<(), ErroAtalhos> {
            if self
                .falha_escrita
                .borrow()
                .contains(&(chave.esquema, chave.nome))
            {
                return Err(Self::erro_indisponivel(chave));
            }
            self.valores
                .borrow_mut()
                .insert((chave.esquema, chave.nome), original.0.clone());
            Ok(())
        }
    }
}
