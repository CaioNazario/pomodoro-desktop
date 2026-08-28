use std::path::PathBuf;

use crate::chave::{Chave, CHAVES_SUPRIMIDAS};
use crate::erro::ErroAtalhos;
use crate::fonte::{FonteDeAtalhos, ValorOriginal};
use crate::snapshot::Snapshot;

/// Transacao com journal em disco (PRD §7.3): snapshot gravado antes de tocar
/// em qualquer chave, restaurado em `Drop`, no shutdown normal e — via
/// [`Self::recuperar_pendente`] — na proxima inicializacao, se um snapshot
/// ficou pendente por `kill -9`, crash ou queda de energia.
pub struct GerenciadorDeAtalhos<F: FonteDeAtalhos> {
    fonte: F,
    caminho_snapshot: PathBuf,
    suprimido: bool,
}

impl<F: FonteDeAtalhos> GerenciadorDeAtalhos<F> {
    pub fn novo(fonte: F, caminho_snapshot: PathBuf) -> Self {
        Self {
            fonte,
            caminho_snapshot,
            suprimido: false,
        }
    }

    /// Chamar uma vez no `setup()`, antes de qualquer [`Self::suprimir`] desta
    /// execucao. Sem efeito se nao houver snapshot pendente.
    pub fn recuperar_pendente(&mut self) {
        let Some(snapshot) = Snapshot::ler(&self.caminho_snapshot) else {
            return;
        };
        self.aplicar_originais(&snapshot.itens);
        let _ = Snapshot::apagar(&self.caminho_snapshot);
    }

    /// Idempotente: uma segunda chamada enquanto ja suprimido e no-op.
    pub fn suprimir(&mut self) -> Result<(), ErroAtalhos> {
        if self.suprimido {
            return Ok(());
        }
        let originais = self.ler_originais();
        Snapshot::escrever(&self.caminho_snapshot, &originais)?;
        self.desligar_todas(&originais);
        self.suprimido = true;
        Ok(())
    }

    /// Idempotente: sem efeito se nao houver supressao ativa.
    pub fn restaurar(&mut self) {
        if !self.suprimido {
            return;
        }
        if let Some(snapshot) = Snapshot::ler(&self.caminho_snapshot) {
            self.aplicar_originais(&snapshot.itens);
        }
        let _ = Snapshot::apagar(&self.caminho_snapshot);
        self.suprimido = false;
    }

    /// Falha ao ler uma chave a exclui do snapshot — degradacao silenciosa,
    /// nunca panic (PRD §7.3).
    fn ler_originais(&self) -> Vec<(Chave, ValorOriginal)> {
        CHAVES_SUPRIMIDAS
            .iter()
            .filter_map(|chave| self.fonte.ler(chave).ok().map(|valor| (*chave, valor)))
            .collect()
    }

    /// Falha ao desligar uma chave nao impede as demais.
    fn desligar_todas(&self, itens: &[(Chave, ValorOriginal)]) {
        for (chave, _) in itens {
            let _ = self.fonte.desligar(chave);
        }
    }

    fn aplicar_originais(&self, itens: &[(Chave, ValorOriginal)]) {
        for (chave, original) in itens {
            let _ = self.fonte.religar(chave, original);
        }
    }
}

impl<F: FonteDeAtalhos> Drop for GerenciadorDeAtalhos<F> {
    fn drop(&mut self) {
        self.restaurar();
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::fonte::fake::FonteFake;

    fn caminho_de_teste(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("snapshot.toml")
    }

    #[test]
    fn suprimir_grava_snapshot_e_desliga_todas_as_chaves() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));

        gerenciador.suprimir().expect("suprimir");

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "desligado");
        }
        assert!(caminho_de_teste(&dir).exists());
    }

    #[test]
    fn restaurar_devolve_valores_originais_e_apaga_o_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let caminho = caminho_de_teste(&dir);
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho.clone());
        gerenciador.suprimir().expect("suprimir");

        gerenciador.restaurar();

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "original");
        }
        assert!(!caminho.exists());
    }

    #[test]
    fn drop_restaura_automaticamente() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));
        gerenciador.suprimir().expect("suprimir");

        drop(gerenciador);

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "original");
        }
    }

    #[test]
    fn suprimir_e_idempotente() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));

        gerenciador.suprimir().expect("primeira supressao");
        gerenciador.suprimir().expect("segunda supressao e no-op");

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "desligado");
        }
    }

    #[test]
    fn falha_ao_ler_uma_chave_a_exclui_sem_impedir_as_demais() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let chave_com_falha = CHAVES_SUPRIMIDAS[0];
        fonte.falhar_leitura_em(chave_com_falha);
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));

        gerenciador
            .suprimir()
            .expect("suprimir mesmo com uma falha de leitura");

        assert_eq!(fonte.valor_atual(&chave_com_falha), "original");
        for chave in &CHAVES_SUPRIMIDAS[1..] {
            assert_eq!(fonte.valor_atual(chave), "desligado");
        }
    }

    #[test]
    fn falha_ao_desligar_uma_chave_nao_impede_as_demais() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let chave_com_falha = CHAVES_SUPRIMIDAS[0];
        fonte.falhar_escrita_em(chave_com_falha);
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));

        gerenciador
            .suprimir()
            .expect("suprimir mesmo com uma falha de escrita");

        assert_eq!(fonte.valor_atual(&chave_com_falha), "original");
        for chave in &CHAVES_SUPRIMIDAS[1..] {
            assert_eq!(fonte.valor_atual(chave), "desligado");
        }
    }

    #[test]
    fn recuperar_pendente_restaura_snapshot_deixado_por_execucao_anterior() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caminho = caminho_de_teste(&dir);
        let fonte = FonteFake::nova();
        let mut gerenciador_anterior = GerenciadorDeAtalhos::novo(fonte.clone(), caminho.clone());
        gerenciador_anterior.suprimir().expect("suprimir");
        // simula `kill -9`: o processo morre sem correr o `Drop`.
        std::mem::forget(gerenciador_anterior);
        assert!(caminho.exists());

        let mut proxima_execucao = GerenciadorDeAtalhos::novo(fonte.clone(), caminho.clone());
        proxima_execucao.recuperar_pendente();

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "original");
        }
        assert!(!caminho.exists());
    }

    #[test]
    fn recuperar_pendente_sem_snapshot_nao_faz_nada() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fonte = FonteFake::nova();
        let mut gerenciador = GerenciadorDeAtalhos::novo(fonte.clone(), caminho_de_teste(&dir));

        gerenciador.recuperar_pendente();

        for chave in CHAVES_SUPRIMIDAS {
            assert_eq!(fonte.valor_atual(chave), "original");
        }
    }
}
