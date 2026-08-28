mod fuga;
mod janela;
mod moldura;

use std::path::PathBuf;
use std::sync::Mutex;

use pomodoro_atalhos_gnome::{FonteGio, GerenciadorDeAtalhos};
use pomodoro_contrato::{EstadoDaTela, EstadoDoBloqueio};
use pomodoro_dominio::{Atividade, Etapa};
use tauri::{AppHandle, Manager};

use janela::criar_janela_de_bloqueio;

/// Orquestra a entrada/saida do modo de bloqueio (PRD §7): janelas fullscreen
/// por monitor, o webview isolado da atividade e a supressao de atalhos do
/// GNOME. Gerenciado como state do Tauri, construido uma vez no `setup()`.
pub struct GerenciadorDeBloqueio {
    atalhos: Mutex<GerenciadorDeAtalhos<FonteGio>>,
    janelas_ativas: Mutex<Vec<String>>,
}

impl GerenciadorDeBloqueio {
    pub fn novo(caminho_snapshot: PathBuf) -> Self {
        Self {
            atalhos: Mutex::new(GerenciadorDeAtalhos::novo(FonteGio, caminho_snapshot)),
            janelas_ativas: Mutex::new(Vec::new()),
        }
    }

    /// Chamar uma vez no `setup()`, antes de qualquer entrada em bloqueio
    /// desta execucao — cobre `kill -9` da execucao anterior.
    pub fn recuperar_pendente(&self) {
        if let Ok(mut atalhos) = self.atalhos.lock() {
            atalhos.recuperar_pendente();
        }
    }

    /// Idempotente: chamar com o bloqueio ja ativo e no-op.
    fn entrar(&self, app: &AppHandle, atividade: &Atividade, tela: &EstadoDaTela) {
        let Ok(mut janelas) = self.janelas_ativas.lock() else {
            return;
        };
        if !janelas.is_empty() {
            return;
        }
        let Some(janela_principal) = app.get_webview_window("main") else {
            return;
        };
        let Ok(monitores) = janela_principal.available_monitors() else {
            return;
        };
        let monitor_principal = janela_principal.current_monitor().ok().flatten();
        let script = moldura::script_de_inicializacao(atividade, tela);

        for (indice, monitor) in monitores.iter().enumerate() {
            let label = format!("bloqueio-{indice}");
            let Ok(janela) = criar_janela_de_bloqueio(app, &label, monitor, &script) else {
                continue;
            };
            janelas.push(label);

            let e_a_principal = monitor_principal
                .as_ref()
                .map(|atual| atual.position() == monitor.position())
                .unwrap_or(indice == 0);
            if e_a_principal {
                if let Some(url) = atividade.url() {
                    let webview: &tauri::Webview<_> = janela.as_ref();
                    let destino = url.destino_de_carregamento();
                    let _ = crate::webview_atividade::abrir(&webview.window(), &destino);
                }
            }
        }
        drop(janelas);

        if let Ok(mut atalhos) = self.atalhos.lock() {
            let _ = atalhos.suprimir();
        }
        crate::widget::esconder_para_bloqueio(app);
        crate::eventos::emitir_bloqueio(app, EstadoDoBloqueio { ativo: true });
    }

    /// Idempotente: chamar sem bloqueio ativo e no-op. Usa `destroy()`, nao
    /// `close()` — `close()` dispara `CloseRequested` de novo, que as
    /// proprias janelas de bloqueio recusam (e travariam a propria saida).
    fn sair(&self, app: &AppHandle) {
        let Ok(mut janelas) = self.janelas_ativas.lock() else {
            return;
        };
        if janelas.is_empty() {
            return;
        }
        let _ = crate::webview_atividade::fechar(app);
        for label in janelas.drain(..) {
            if let Some(janela) = app.get_webview_window(&label) {
                let _ = janela.destroy();
            }
        }
        drop(janelas);

        if let Ok(mut atalhos) = self.atalhos.lock() {
            atalhos.restaurar();
        }
        crate::widget::restaurar_apos_bloqueio(app);
        crate::eventos::emitir_bloqueio(app, EstadoDoBloqueio { ativo: false });
    }

    /// Unica saida real do bloqueio (PRD §7.6): mesmo efeito de janela que
    /// `sair()` (derruba o webview da atividade, destroi as janelas de
    /// bloqueio, restaura atalhos), mas — ao contrario de `sair()`, chamado
    /// so quando a Pausa ja acabou de verdade — aqui a etapa continua Pausa,
    /// o timer nao e tocado, e conta como interrupcao. Chamado pelo command
    /// `confirmar_urgencia`, disparado de dentro da propria janela de
    /// bloqueio.
    pub fn confirmar_urgencia(&self, app: &AppHandle) {
        self.sair(app);
        crate::registro_de_foco::registrar_pausa_interrompida(app);
        crate::persistencia::persistir(app);
    }
}

/// Chamado apos toda transicao de etapa (vencimento natural ou
/// `pular_etapa`): entra em bloqueio ao alcancar uma Pausa com URL
/// aplicavel, sai ao deixar uma Pausa. Sem efeito pra qualquer outra
/// transicao (Foco->Foco nao existe; Pausa->Pausa idem).
pub fn reagir_a_transicao(
    app: &AppHandle,
    etapa_antes: Etapa,
    etapa_depois: Etapa,
    atividade_da_pausa: &Atividade,
    tela_depois: &EstadoDaTela,
) {
    let Some(gerenciador) = app.try_state::<GerenciadorDeBloqueio>() else {
        return;
    };
    match (etapa_antes, etapa_depois) {
        (Etapa::Foco, Etapa::Pausa) if atividade_da_pausa.tem_url_aplicavel() => {
            gerenciador.entrar(app, atividade_da_pausa, tela_depois);
        }
        (Etapa::Pausa, Etapa::Foco) => gerenciador.sair(app),
        _ => {}
    }
}
