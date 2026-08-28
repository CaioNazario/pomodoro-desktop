use crate::{
    ciclo::NumeroDeSessao, ciclo_em_execucao::CicloEmExecucao, modo_de_duracao::ModoDeDuracao,
    quantidade_de_sessoes::QuantidadeDeSessoes, sessao::Sessao, Atividade, Duracao, Instante,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessaoForaDoPlano {
    #[error("sessao fora do plano: {recebido}, esperado entre 1 e {total}")]
    ForaDoIntervalo { recebido: u8, total: u8 },
}

/// Plano de duracoes do ciclo: guarda os dois conjuntos de duracao que
/// coexistem sempre — globais e por sessao — e o modo ativo. Nao duplica o
/// timer: as operacoes recebem o `CicloEmExecucao` corrente e devolvem a
/// versao atualizada, mantendo o timer como fonte unica de verdade (por
/// isso o app gerencia os dois num `Mutex` cada, nunca um dentro do outro).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanoDoCiclo {
    modo: ModoDeDuracao,
    duracao_global_foco: Duracao,
    duracao_global_pausa: Duracao,
    atividade_global: Atividade,
    plano_individual: Vec<Sessao>,
}

impl PlanoDoCiclo {
    pub fn novo(
        total_sessoes: QuantidadeDeSessoes,
        duracao_global_foco: Duracao,
        duracao_global_pausa: Duracao,
    ) -> Self {
        let sessao_padrao = Sessao::nova(duracao_global_foco, duracao_global_pausa);
        Self {
            modo: ModoDeDuracao::Global,
            duracao_global_foco,
            duracao_global_pausa,
            atividade_global: Atividade::vazia(),
            plano_individual: vec![sessao_padrao; total_sessoes.valor() as usize],
        }
    }

    /// Reconstroi a partir de valores persistidos — usado por
    /// `pomodoro-config` ao carregar o TOML do disco. Nao valida a relacao
    /// entre `plano_individual` e um `QuantidadeDeSessoes`: quem persistiu
    /// ja validou na escrita.
    pub fn reconstruir(
        modo: ModoDeDuracao,
        duracao_global_foco: Duracao,
        duracao_global_pausa: Duracao,
        atividade_global: Atividade,
        plano_individual: Vec<Sessao>,
    ) -> Self {
        Self {
            modo,
            duracao_global_foco,
            duracao_global_pausa,
            atividade_global,
            plano_individual,
        }
    }

    pub const fn modo(&self) -> ModoDeDuracao {
        self.modo
    }

    pub const fn duracao_global_foco(&self) -> Duracao {
        self.duracao_global_foco
    }

    pub const fn duracao_global_pausa(&self) -> Duracao {
        self.duracao_global_pausa
    }

    pub fn atividade_global(&self) -> &Atividade {
        &self.atividade_global
    }

    pub fn plano_individual(&self) -> &[Sessao] {
        &self.plano_individual
    }

    pub fn total_sessoes(&self) -> u8 {
        self.plano_individual.len() as u8
    }

    /// Duracoes de foco/pausa que `sessao` usaria agora, no modo ativo do
    /// plano — usado pra reconstruir um `CicloEmExecucao` a partir de um
    /// `PlanoDoCiclo` carregado do disco (`pomodoro-config`).
    pub fn duracoes_ativas_para(&self, sessao: NumeroDeSessao) -> (Duracao, Duracao) {
        self.duracoes_para(self.modo, sessao)
    }

    /// Atividade que `sessao` usaria agora, no modo ativo do plano (PRD
    /// §3: Global tem uma so, repetida; Individual tem uma por sessao).
    pub fn atividade_ativa_para(&self, sessao: NumeroDeSessao) -> &Atividade {
        match self.modo {
            ModoDeDuracao::Global => &self.atividade_global,
            ModoDeDuracao::Individual => self.plano_individual[indice_de(sessao)].atividade(),
        }
    }

    /// Duracoes ativas para `sessao` no `modo` informado — global ignora a
    /// sessao, individual busca a entrada correspondente no plano.
    fn duracoes_para(&self, modo: ModoDeDuracao, sessao: NumeroDeSessao) -> (Duracao, Duracao) {
        match modo {
            ModoDeDuracao::Global => (self.duracao_global_foco, self.duracao_global_pausa),
            ModoDeDuracao::Individual => {
                let sessao = &self.plano_individual[indice_de(sessao)];
                (sessao.foco(), sessao.pausa())
            }
        }
    }

    /// M<N trunca o plano individual e clampa a sessao em curso se ela ficou
    /// fora do novo total. M>N estende com as duracoes globais vigentes.
    /// Nunca toca no timer: redimensionar nao altera o prazo em curso.
    pub fn redimensionar(
        mut self,
        nova_quantidade: QuantidadeDeSessoes,
        ciclo: CicloEmExecucao,
    ) -> (Self, CicloEmExecucao) {
        let padrao = Sessao::nova(self.duracao_global_foco, self.duracao_global_pausa);
        self.plano_individual
            .resize(nova_quantidade.valor() as usize, padrao);
        let sessao_clampada = clampar_sessao(ciclo.sessao(), nova_quantidade);
        let ciclo = ciclo.com_total_e_sessao(nova_quantidade, sessao_clampada);
        (self, ciclo)
    }

    /// Troca o modo e realinha a duracao ativa do ciclo para o conjunto
    /// novo — mesma mecanica de alterar a duracao da etapa em curso.
    pub fn trocar_modo(
        self,
        novo_modo: ModoDeDuracao,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> (Self, CicloEmExecucao) {
        let (foco, pausa) = self.duracoes_para(novo_modo, ciclo.sessao());
        let ciclo = ciclo
            .com_duracao_foco(foco, agora)
            .com_duracao_pausa(pausa, agora);
        (
            Self {
                modo: novo_modo,
                ..self
            },
            ciclo,
        )
    }

    /// Realinha a duracao ativa do ciclo apos um avanco natural de etapa —
    /// em modo individual, a sessao recem-entrada pode ter uma duracao
    /// diferente da que estava valendo antes.
    pub fn realinhar_apos_avancar(
        &self,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> CicloEmExecucao {
        let (foco, pausa) = self.duracoes_para(self.modo, ciclo.sessao());
        ciclo
            .com_duracao_foco(foco, agora)
            .com_duracao_pausa(pausa, agora)
    }

    pub fn alterar_duracao_global_foco(
        self,
        nova: Duracao,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> (Self, CicloEmExecucao) {
        let ciclo = self.aplicar_se_global(ciclo, |c| c.com_duracao_foco(nova, agora));
        (
            Self {
                duracao_global_foco: nova,
                ..self
            },
            ciclo,
        )
    }

    pub fn alterar_duracao_global_pausa(
        self,
        nova: Duracao,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> (Self, CicloEmExecucao) {
        let ciclo = self.aplicar_se_global(ciclo, |c| c.com_duracao_pausa(nova, agora));
        (
            Self {
                duracao_global_pausa: nova,
                ..self
            },
            ciclo,
        )
    }

    pub fn alterar_atividade_global(self, nova: Atividade) -> Self {
        Self {
            atividade_global: nova,
            ..self
        }
    }

    fn aplicar_se_global(
        &self,
        ciclo: CicloEmExecucao,
        alterar: impl FnOnce(CicloEmExecucao) -> CicloEmExecucao,
    ) -> CicloEmExecucao {
        if self.modo == ModoDeDuracao::Global {
            alterar(ciclo)
        } else {
            ciclo
        }
    }

    pub fn alterar_duracao_individual_foco(
        mut self,
        sessao_alvo: NumeroDeSessao,
        nova: Duracao,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> Result<(Self, CicloEmExecucao), SessaoForaDoPlano> {
        self.validar_sessao(sessao_alvo)?;
        let indice = indice_de(sessao_alvo);
        self.plano_individual[indice] = self.plano_individual[indice].clone().com_foco(nova);
        let ciclo =
            self.aplicar_se_sessao_ativa(sessao_alvo, ciclo, |c| c.com_duracao_foco(nova, agora));
        Ok((self, ciclo))
    }

    pub fn alterar_duracao_individual_pausa(
        mut self,
        sessao_alvo: NumeroDeSessao,
        nova: Duracao,
        ciclo: CicloEmExecucao,
        agora: Instante,
    ) -> Result<(Self, CicloEmExecucao), SessaoForaDoPlano> {
        self.validar_sessao(sessao_alvo)?;
        let indice = indice_de(sessao_alvo);
        self.plano_individual[indice] = self.plano_individual[indice].clone().com_pausa(nova);
        let ciclo =
            self.aplicar_se_sessao_ativa(sessao_alvo, ciclo, |c| c.com_duracao_pausa(nova, agora));
        Ok((self, ciclo))
    }

    pub fn alterar_atividade_individual(
        mut self,
        sessao_alvo: NumeroDeSessao,
        nova: Atividade,
    ) -> Result<Self, SessaoForaDoPlano> {
        self.validar_sessao(sessao_alvo)?;
        let indice = indice_de(sessao_alvo);
        self.plano_individual[indice] = self.plano_individual[indice].clone().com_atividade(nova);
        Ok(self)
    }

    fn aplicar_se_sessao_ativa(
        &self,
        sessao_alvo: NumeroDeSessao,
        ciclo: CicloEmExecucao,
        alterar: impl FnOnce(CicloEmExecucao) -> CicloEmExecucao,
    ) -> CicloEmExecucao {
        let e_a_ativa = self.modo == ModoDeDuracao::Individual && ciclo.sessao() == sessao_alvo;
        if e_a_ativa {
            alterar(ciclo)
        } else {
            ciclo
        }
    }

    fn validar_sessao(&self, sessao: NumeroDeSessao) -> Result<(), SessaoForaDoPlano> {
        let total = self.total_sessoes();
        if sessao.valor() < 1 || sessao.valor() > total {
            return Err(SessaoForaDoPlano::ForaDoIntervalo {
                recebido: sessao.valor(),
                total,
            });
        }
        Ok(())
    }
}

fn indice_de(sessao: NumeroDeSessao) -> usize {
    (sessao.valor() - 1) as usize
}

fn clampar_sessao(atual: NumeroDeSessao, total: QuantidadeDeSessoes) -> NumeroDeSessao {
    if atual.valor() > total.valor() {
        NumeroDeSessao::de(total.valor())
    } else {
        atual
    }
}

#[cfg(test)]
#[path = "plano_do_ciclo/testes.rs"]
mod testes;
