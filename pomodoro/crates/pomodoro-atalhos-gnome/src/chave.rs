/// Uma chave gsettings suprimivel: par (esquema, nome). `'static` porque a
/// lista inteira e conhecida em tempo de compilacao — nunca vem do frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chave {
    pub esquema: &'static str,
    pub nome: &'static str,
}

impl Chave {
    const fn nova(esquema: &'static str, nome: &'static str) -> Self {
        Self { esquema, nome }
    }
}

/// Atalhos do compositor suprimidos durante o bloqueio (PRD §7.3). A contagem
/// exata de `switch-to-workspace-*`/`switch-to-session-*`/`switch-to-application-*`
/// e uma escolha de cobertura, nao um limite do protocolo — o gsettings aceita
/// qualquer chave que o schema declare.
pub const CHAVES_SUPRIMIDAS: &[Chave] = &[
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-applications"),
    Chave::nova(
        "org.gnome.desktop.wm.keybindings",
        "switch-applications-backward",
    ),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-windows"),
    Chave::nova(
        "org.gnome.desktop.wm.keybindings",
        "switch-windows-backward",
    ),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-group"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-group-backward"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-panels"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-panels-backward"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "minimize"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "close"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "show-desktop"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "panel-run-dialog"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-to-workspace-1"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-to-workspace-2"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-to-workspace-3"),
    Chave::nova("org.gnome.desktop.wm.keybindings", "switch-to-workspace-4"),
    Chave::nova(
        "org.gnome.desktop.wm.keybindings",
        "switch-to-workspace-left",
    ),
    Chave::nova(
        "org.gnome.desktop.wm.keybindings",
        "switch-to-workspace-right",
    ),
    Chave::nova("org.gnome.shell.keybindings", "toggle-overview"),
    Chave::nova("org.gnome.shell.keybindings", "toggle-application-view"),
    Chave::nova("org.gnome.shell.keybindings", "toggle-message-tray"),
    Chave::nova("org.gnome.shell.keybindings", "switch-to-application-1"),
    Chave::nova("org.gnome.shell.keybindings", "switch-to-application-2"),
    Chave::nova("org.gnome.shell.keybindings", "switch-to-application-3"),
    Chave::nova("org.gnome.mutter", "overlay-key"),
    Chave::nova(
        "org.gnome.mutter.wayland.keybindings",
        "switch-to-session-1",
    ),
    Chave::nova(
        "org.gnome.mutter.wayland.keybindings",
        "switch-to-session-2",
    ),
];

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn chaves_suprimidas_nao_tem_par_esquema_nome_repetido() {
        let mut vistas = std::collections::HashSet::new();
        for chave in CHAVES_SUPRIMIDAS {
            assert!(
                vistas.insert((chave.esquema, chave.nome)),
                "chave repetida: {chave:?}"
            );
        }
    }
}
