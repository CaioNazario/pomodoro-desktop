//! PRD §7.2: o webview da atividade nao pode alcancar nenhum command, evento
//! ou API do Tauri. A garantia e estrutural: nem `windows` (que no Tauri 2
//! escopa TODOS os webviews da janela, filhos inclusive — usar isso pro
//! webview principal vazaria pro nosso filho) nem `webviews` de nenhuma
//! capability podem casar com o label da atividade, direta ou via glob `*`.
//! Teste estatico: nao instancia o Tauri, so le os arquivos de verdade que
//! vao pro bundle.

use serde_json::Value;
use std::path::Path;

fn coletar_padroes(valor: &Value, chave: &str, saida: &mut Vec<String>) {
    if let Value::Object(mapa) = valor {
        if let Some(Value::Array(itens)) = mapa.get(chave) {
            itens
                .iter()
                .filter_map(Value::as_str)
                .for_each(|padrao| saida.push(padrao.to_string()));
        }
        mapa.values().for_each(|v| coletar_padroes(v, chave, saida));
    }
    if let Value::Array(itens) = valor {
        itens.iter().for_each(|v| coletar_padroes(v, chave, saida));
    }
}

#[test]
fn nenhuma_capability_escopa_windows_ou_webviews_para_a_atividade() {
    let diretorio = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
    let mut arquivos_verificados = 0;

    for entrada in std::fs::read_dir(&diretorio).expect("ler diretorio de capabilities") {
        let caminho = entrada.expect("entrada de diretorio").path();
        if caminho.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let conteudo = std::fs::read_to_string(&caminho)
            .unwrap_or_else(|erro| panic!("ler {}: {erro}", caminho.display()));
        let json: Value = serde_json::from_str(&conteudo)
            .unwrap_or_else(|erro| panic!("parsear {}: {erro}", caminho.display()));

        let mut padroes = Vec::new();
        coletar_padroes(&json, "windows", &mut padroes);
        coletar_padroes(&json, "webviews", &mut padroes);

        for padrao in padroes {
            assert!(
                padrao != pomodoro_lib::webview_atividade::LABEL && padrao != "*",
                "{} escopa '{}' (windows/webviews), o que alcancaria o webview isolado da atividade",
                caminho.display(),
                padrao
            );
        }
        arquivos_verificados += 1;
    }

    assert!(
        arquivos_verificados > 0,
        "nenhum arquivo de capability encontrado em {}",
        diretorio.display()
    );
}
