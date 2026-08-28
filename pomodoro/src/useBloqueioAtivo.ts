import { useEffect, useState } from "react";
import { events } from "./bindings";

/// Se o modo de bloqueio (PRD §7) esta ativo agora — so muda em transicao,
/// nunca a 1Hz. Usado pra desativar os atalhos do app (M/Esc) enquanto o
/// bloqueio segura a tela; o estado inicial e sempre `false` porque o app
/// nunca abre ja em bloqueio (nada persiste isso entre execucoes).
export function useBloqueioAtivo(): boolean {
  const [ativo, setAtivo] = useState(false);

  useEffect(() => {
    const inscricao = events.bloqueioMudou.listen((evento) => setAtivo(evento.payload.ativo));
    return () => {
      inscricao.then((parar) => parar());
    };
  }, []);

  return ativo;
}
