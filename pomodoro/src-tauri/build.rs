fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "obter_estado",
            "alternar_execucao",
            "reiniciar_etapa",
            "pular_etapa",
            "alterar_iniciar_automaticamente",
            "obter_plano",
            "redimensionar",
            "trocar_modo",
            "alterar_duracao_global_foco",
            "alterar_duracao_global_pausa",
            "alterar_duracao_individual_foco",
            "alterar_duracao_individual_pausa",
            "alterar_atividade_global",
            "alterar_atividade_individual",
            "obter_contadores_do_dia",
            "confirmar_urgencia",
            "alternar_widget",
        ]));
    tauri_build::try_build(attributes).expect("gerar manifesto de permissoes ACL do app");
}
