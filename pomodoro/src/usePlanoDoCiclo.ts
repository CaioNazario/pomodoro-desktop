import { useEffect, useState } from "react";
import { commands, events, type EstadoDoPlano } from "./bindings";

/// Busca o plano uma vez e escuta `planoMudou` pra manter em dia — mesmo
/// padrao de `useContadoresDoDia`/`useEstadoDoTimer`: o Rust so emite em
/// mutacao real, nunca a 1Hz.
export function usePlanoDoCiclo(): EstadoDoPlano | null {
  const [plano, setPlano] = useState<EstadoDoPlano | null>(null);

  useEffect(() => {
    let ativo = true;
    commands.obterPlano().then((resposta) => {
      if (ativo && resposta.status === "ok") setPlano(resposta.data);
    });
    const inscricao = events.planoMudou.listen((evento) => setPlano(evento.payload));
    return () => {
      ativo = false;
      inscricao.then((parar) => parar());
    };
  }, []);

  return plano;
}
