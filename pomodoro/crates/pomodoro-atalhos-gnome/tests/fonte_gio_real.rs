//! So roda contra uma sessao GNOME de verdade (D-Bus + dconf) — nunca em CI,
//! que nao tem sessao nenhuma (ver `FonteGio` em `src/fonte_gio.rs`).
//! Manual: `cargo test -p pomodoro-atalhos-gnome -- --ignored`.
//!
//! Usa `close` (`Alt+F4`/`Super+Q`), uma chave de baixo risco pra alterar
//! temporariamente numa sessao de desenvolvimento — nao interfere com
//! atalhos de terminal/shell.

use pomodoro_atalhos_gnome::{Chave, FonteDeAtalhos, FonteGio};

#[test]
#[ignore]
fn desligar_e_religar_uma_chave_de_verdade_muda_e_restaura_o_gsettings() {
    let chave = Chave {
        esquema: "org.gnome.desktop.wm.keybindings",
        nome: "close",
    };
    let fonte = FonteGio;

    let original = fonte.ler(&chave).expect("ler o valor original de verdade");

    fonte.desligar(&chave).expect("desligar de verdade");
    let desligado = fonte.ler(&chave).expect("ler apos desligar");
    assert_ne!(desligado, original, "desligar deveria mudar o valor real");

    fonte
        .religar(&chave, &original)
        .expect("religar de verdade");
    let restaurado = fonte.ler(&chave).expect("ler apos religar");
    assert_eq!(
        restaurado, original,
        "religar deveria devolver exatamente o valor original"
    );
}
