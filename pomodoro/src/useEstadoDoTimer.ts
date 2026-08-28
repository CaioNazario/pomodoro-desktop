import { useEffect, useState } from "react";
import { commands, events, type EstadoDaTela } from "./bindings";

export interface EstadoDoTimer {
  estado: EstadoDaTela | null;
  restanteMs: number;
  alternarExecucao: () => void;
  reiniciarEtapa: () => void;
  pularEtapa: () => void;
  alterarIniciarAutomaticamente: (valor: boolean) => void;
}

function restanteAgora(estado: EstadoDaTela): number {
  if (estado.rodando && estado.prazoEpocaMs !== null) {
    return Math.max(0, estado.prazoEpocaMs - Date.now());
  }
  return estado.restanteMs;
}

function ateProximoSegundo(restanteMs: number): number {
  const resto = restanteMs % 1000;
  return resto === 0 ? 1000 : resto;
}

/// O Rust nao emite tick: cada transicao chega via evento, e este hook deriva
/// o `mm:ss` localmente a partir do prazo absoluto. Reagenda-se sempre para o
/// proximo segundo cheio em vez de todo quadro (rAF/60Hz): so o segundo
/// exibido muda, e recalcular a cada 16ms so joga trabalho fora — em dobro,
/// ja que este hook roda tanto na janela principal quanto no widget.
export function useEstadoDoTimer(): EstadoDoTimer {
  const [estado, setEstado] = useState<EstadoDaTela | null>(null);

  const aplicar = (novo: EstadoDaTela) => setEstado(novo);

  useEffect(() => {
    let ativo = true;
    commands.obterEstado().then((resposta) => {
      if (ativo && resposta.status === "ok") aplicar(resposta.data);
    });
    const inscricao = events.estadoMudou.listen((evento) => aplicar(evento.payload));
    return () => {
      ativo = false;
      inscricao.then((parar) => parar());
    };
  }, []);

  const [restanteMs, setRestanteMs] = useState(0);
  useEffect(() => {
    if (!estado?.rodando) {
      if (estado) setRestanteMs(estado.restanteMs);
      return;
    }
    let temporizador: ReturnType<typeof setTimeout>;
    const passo = () => {
      const atual = restanteAgora(estado);
      setRestanteMs(atual);
      if (atual <= 0) return;
      temporizador = setTimeout(passo, ateProximoSegundo(atual));
    };
    passo();
    return () => clearTimeout(temporizador);
  }, [estado]);

  return {
    estado,
    restanteMs,
    alternarExecucao: () => commands.alternarExecucao().then((r) => r.status === "ok" && aplicar(r.data)),
    reiniciarEtapa: () => commands.reiniciarEtapa().then((r) => r.status === "ok" && aplicar(r.data)),
    pularEtapa: () => commands.pularEtapa().then((r) => r.status === "ok" && aplicar(r.data)),
    alterarIniciarAutomaticamente: (valor: boolean) =>
      commands.alterarIniciarAutomaticamente(valor).then((r) => r.status === "ok" && aplicar(r.data)),
  };
}
