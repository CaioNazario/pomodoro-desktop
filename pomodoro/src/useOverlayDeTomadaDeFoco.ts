import { useEffect, useRef, useState } from "react";
import { events } from "./bindings";

const DURACAO_MS = 850;

/// PRD §8: overlay preto "trazendo a janela para frente…" por ~850ms.
/// Reage ao evento `tomadaDeFocoOcorreu`, nunca inferindo da transicao de
/// etapa — o Rust ja decide as duas condicoes (fim de Foco automatico,
/// nunca skip manual; so com URL aplicavel) e so emite quando elas batem.
export function useOverlayDeTomadaDeFoco(): boolean {
  const [ativo, setAtivo] = useState(false);
  const temporizador = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const inscricao = events.tomadaDeFocoOcorreu.listen(() => {
      if (temporizador.current) clearTimeout(temporizador.current);
      setAtivo(true);
      temporizador.current = setTimeout(() => setAtivo(false), DURACAO_MS);
    });
    return () => {
      inscricao.then((parar) => parar());
      if (temporizador.current) clearTimeout(temporizador.current);
    };
  }, []);

  return ativo;
}
