import { useEffect, useState } from "react";
import { commands, events, type EstadoDoHistorico } from "./bindings";

/// Busca os contadores do dia uma vez e escuta `historicoMudou` pra manter
/// em dia — o Rust so emite em transicao (sessao concluida, foco pulado
/// pela metade), nunca a 1Hz.
export function useContadoresDoDia(): EstadoDoHistorico | null {
  const [contadores, setContadores] = useState<EstadoDoHistorico | null>(null);

  useEffect(() => {
    let ativo = true;
    commands.obterContadoresDoDia().then((resposta) => {
      if (ativo && resposta.status === "ok") setContadores(resposta.data);
    });
    const inscricao = events.historicoMudou.listen((evento) => setContadores(evento.payload));
    return () => {
      ativo = false;
      inscricao.then((parar) => parar());
    };
  }, []);

  return contadores;
}
