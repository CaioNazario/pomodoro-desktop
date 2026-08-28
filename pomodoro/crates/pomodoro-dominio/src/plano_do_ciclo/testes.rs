use super::*;
use crate::relogio_fake::RelogioFake;
use crate::{Atividade, CicloEmExecucao, Relogio, UrlDeAtividade};

fn total(quantidade: u8) -> QuantidadeDeSessoes {
    QuantidadeDeSessoes::nova(quantidade).expect("teste")
}

fn ciclo_de_oito() -> CicloEmExecucao {
    CicloEmExecucao::novo(
        total(8),
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        true,
    )
}

fn ciclo_na_sessao(numero: u8) -> CicloEmExecucao {
    let mut ciclo = ciclo_de_oito();
    for _ in 0..(2 * (numero - 1)) {
        ciclo = ciclo.avancar(Instante::desde_epoca_ms(0));
    }
    ciclo
}

#[test]
fn duracoes_ativas_para_reflete_o_modo_individual() {
    let plano = PlanoDoCiclo::reconstruir(
        ModoDeDuracao::Individual,
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        Atividade::vazia(),
        vec![
            Sessao::nova(Duracao::de_minutos(30), Duracao::de_minutos(10)),
            Sessao::nova(Duracao::de_minutos(15), Duracao::de_minutos(5)),
        ],
    );
    assert_eq!(
        plano.duracoes_ativas_para(NumeroDeSessao::primeira()),
        (Duracao::de_minutos(30), Duracao::de_minutos(10))
    );
}

#[test]
fn duracoes_ativas_para_reflete_o_modo_global() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    assert_eq!(
        plano.duracoes_ativas_para(NumeroDeSessao::primeira()),
        (Duracao::de_minutos(25), Duracao::de_minutos(5))
    );
}

#[test]
fn reconstruir_preserva_modo_duracoes_e_plano_individual() {
    let plano_individual = vec![
        Sessao::nova(Duracao::de_minutos(30), Duracao::de_minutos(10)),
        Sessao::nova(Duracao::de_minutos(15), Duracao::de_minutos(5)),
    ];
    let plano = PlanoDoCiclo::reconstruir(
        ModoDeDuracao::Individual,
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        Atividade::vazia(),
        plano_individual.clone(),
    );
    assert_eq!(plano.modo(), ModoDeDuracao::Individual);
    assert_eq!(plano.duracao_global_foco(), Duracao::de_minutos(25));
    assert_eq!(plano.plano_individual(), plano_individual.as_slice());
}

#[test]
fn redimensionar_para_baixo_com_sessao_em_curso_acima_do_novo_total_clampa() {
    let plano = PlanoDoCiclo::novo(total(8), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_na_sessao(6);
    assert_eq!(ciclo.sessao().valor(), 6);

    let (plano, ciclo) = plano.redimensionar(total(3), ciclo);
    assert_eq!(plano.total_sessoes(), 3);
    assert_eq!(ciclo.total_sessoes(), 3);
    assert_eq!(ciclo.sessao().valor(), 3);
}

#[test]
fn redimensionar_para_cima_preserva_existentes_e_usa_duracoes_globais_vigentes() {
    let plano = PlanoDoCiclo::novo(total(2), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_de_oito().com_total_e_sessao(total(2), NumeroDeSessao::primeira());
    let plano = plano
        .alterar_duracao_global_foco(Duracao::de_minutos(50), ciclo, Instante::desde_epoca_ms(0))
        .0;

    let (plano, _) = plano.redimensionar(total(4), ciclo);
    assert_eq!(plano.total_sessoes(), 4);
    assert_eq!(plano.plano_individual()[0].foco(), Duracao::de_minutos(25));
    assert_eq!(plano.plano_individual()[2].foco(), Duracao::de_minutos(50));
    assert_eq!(plano.plano_individual()[3].foco(), Duracao::de_minutos(50));
}

#[test]
fn redimensionar_com_timer_rodando_nao_altera_o_prazo_em_curso() {
    let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
    let plano = PlanoDoCiclo::novo(total(8), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_de_oito().alternar_execucao(relogio.agora());
    relogio.avancar(Duracao::de_minutos(10));

    let (_, redimensionado) = plano.redimensionar(total(3), ciclo);
    assert!(redimensionado.rodando());
    assert_eq!(
        redimensionado.restante_em(relogio.agora()),
        Duracao::de_minutos(15)
    );
}

#[test]
fn alterar_duracao_global_foco_em_curso_recalcula_o_prazo_a_partir_de_agora() {
    let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = CicloEmExecucao::novo(
        total(4),
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        true,
    )
    .alternar_execucao(relogio.agora());
    relogio.avancar(Duracao::de_minutos(12));

    let (_, ciclo) =
        plano.alterar_duracao_global_foco(Duracao::de_minutos(50), ciclo, relogio.agora());
    assert_eq!(ciclo.restante_em(relogio.agora()), Duracao::de_minutos(50));
}

#[test]
fn alterar_duracao_que_nao_e_a_ativa_nao_mexe_no_timer_corrente() {
    let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = CicloEmExecucao::novo(
        total(4),
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        true,
    )
    .alternar_execucao(relogio.agora());
    relogio.avancar(Duracao::de_minutos(12));

    // etapa atual e Foco, modo Global: alterar a PAUSA global nao mexe.
    let (_, ciclo) =
        plano.alterar_duracao_global_pausa(Duracao::de_minutos(20), ciclo, relogio.agora());
    assert_eq!(ciclo.restante_em(relogio.agora()), Duracao::de_minutos(13));
}

#[test]
fn alterar_duracao_com_timer_parado_recarrega_o_restante_para_a_duracao_cheia() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_de_oito().com_total_e_sessao(total(4), NumeroDeSessao::primeira());
    assert!(!ciclo.rodando());

    let (_, ciclo) = plano.alterar_duracao_global_foco(
        Duracao::de_minutos(50),
        ciclo,
        Instante::desde_epoca_ms(0),
    );
    assert!(!ciclo.rodando());
    assert_eq!(
        ciclo.restante_em(Instante::desde_epoca_ms(0)),
        Duracao::de_minutos(50)
    );
}

#[test]
fn trocar_de_modo_ida_e_volta_preserva_os_dois_conjuntos() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_de_oito()
        .com_total_e_sessao(total(4), NumeroDeSessao::primeira())
        .alternar_execucao(Instante::desde_epoca_ms(0));

    let (plano, ciclo) = plano.trocar_modo(
        ModoDeDuracao::Individual,
        ciclo,
        Instante::desde_epoca_ms(0),
    );
    assert_eq!(plano.duracao_global_foco(), Duracao::de_minutos(25));
    assert_eq!(plano.duracao_global_pausa(), Duracao::de_minutos(5));

    let (plano, _) = plano.trocar_modo(ModoDeDuracao::Global, ciclo, Instante::desde_epoca_ms(0));
    assert_eq!(plano.modo(), ModoDeDuracao::Global);
    assert_eq!(plano.duracao_global_foco(), Duracao::de_minutos(25));
    assert_eq!(plano.plano_individual().len(), 4);
}

#[test]
fn alterar_duracao_individual_de_sessao_fora_do_plano_e_erro() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let ciclo = ciclo_de_oito().com_total_e_sessao(total(4), NumeroDeSessao::primeira());

    let erro = plano
        .alterar_duracao_individual_foco(
            NumeroDeSessao::de(5),
            Duracao::de_minutos(50),
            ciclo,
            Instante::desde_epoca_ms(0),
        )
        .expect_err("teste");
    assert_eq!(
        erro,
        SessaoForaDoPlano::ForaDoIntervalo {
            recebido: 5,
            total: 4
        }
    );
}

#[test]
fn atividade_nova_e_vazia_para_toda_sessao() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    assert!(!plano
        .atividade_ativa_para(NumeroDeSessao::primeira())
        .tem_url_aplicavel());
}

#[test]
fn atividade_ativa_para_reflete_o_modo_global() {
    let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
    let atividade = Atividade::nova(None, Some(url));
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5))
        .alterar_atividade_global(atividade.clone());
    assert_eq!(
        plano.atividade_ativa_para(NumeroDeSessao::de(3)),
        &atividade
    );
}

#[test]
fn atividade_ativa_para_reflete_o_modo_individual() {
    let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
    let atividade = Atividade::nova(None, Some(url));
    let plano = PlanoDoCiclo::reconstruir(
        ModoDeDuracao::Individual,
        Duracao::de_minutos(25),
        Duracao::de_minutos(5),
        Atividade::vazia(),
        vec![
            Sessao::nova(Duracao::de_minutos(25), Duracao::de_minutos(5)),
            Sessao::nova(Duracao::de_minutos(25), Duracao::de_minutos(5))
                .com_atividade(atividade.clone()),
        ],
    );
    assert_eq!(
        plano.atividade_ativa_para(NumeroDeSessao::de(2)),
        &atividade
    );
    assert!(!plano
        .atividade_ativa_para(NumeroDeSessao::primeira())
        .tem_url_aplicavel());
}

#[test]
fn alterar_atividade_individual_de_sessao_fora_do_plano_e_erro() {
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5));
    let erro = plano
        .alterar_atividade_individual(NumeroDeSessao::de(5), Atividade::vazia())
        .expect_err("teste");
    assert_eq!(
        erro,
        SessaoForaDoPlano::ForaDoIntervalo {
            recebido: 5,
            total: 4
        }
    );
}

#[test]
fn alterar_atividade_individual_nao_afeta_outras_sessoes() {
    let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
    let atividade = Atividade::nova(None, Some(url));
    let plano = PlanoDoCiclo::novo(total(4), Duracao::de_minutos(25), Duracao::de_minutos(5))
        .alterar_atividade_individual(NumeroDeSessao::de(2), atividade.clone())
        .expect("sessao dentro do plano");
    assert_eq!(plano.plano_individual()[1].atividade(), &atividade);
    assert!(!plano.plano_individual()[0].atividade().tem_url_aplicavel());
}

#[test]
fn redimensionar_para_cima_novas_sessoes_nascem_com_atividade_vazia() {
    let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
    let plano = PlanoDoCiclo::novo(total(2), Duracao::de_minutos(25), Duracao::de_minutos(5))
        .alterar_atividade_individual(NumeroDeSessao::de(1), Atividade::nova(None, Some(url)))
        .expect("sessao dentro do plano");
    let ciclo = ciclo_de_oito().com_total_e_sessao(total(2), NumeroDeSessao::primeira());

    let (plano, _) = plano.redimensionar(total(4), ciclo);
    assert!(plano.plano_individual()[0].atividade().tem_url_aplicavel());
    assert!(!plano.plano_individual()[3].atividade().tem_url_aplicavel());
}
