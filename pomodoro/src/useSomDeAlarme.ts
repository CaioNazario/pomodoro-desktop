import { useEffect, useRef, useState } from "react";
import { commands, events, type SomDeAlarme } from "./bindings";

export const ARQUIVO_DO_SOM: Record<SomDeAlarme, string> = {
  classico: "/sons/classico.mp3",
  suave: "/sons/suave.mp3",
  urgente: "/sons/urgente.mp3",
};

export interface EstadoDoAlarme {
  som: SomDeAlarme | null;
  alterarSom: (som: SomDeAlarme) => void;
  tocarPreview: (som: SomDeAlarme) => void;
}

function tocar(som: SomDeAlarme): void {
  void new Audio(ARQUIVO_DO_SOM[som]).play().catch(() => {});
}

/// Busca a preferencia uma vez e escuta `somDeAlarmeMudou` pra manter em dia
/// (mesmo padrao de `usePlanoDoCiclo`), alem de escutar `tocarAlarme` — o
/// sinal do Rust de que uma transicao de etapa sem URL aplicavel aconteceu
/// agora — e tocar o som selecionado no momento (via ref, pra nao recriar a
/// inscricao a cada troca de preferencia).
export function useSomDeAlarme(): EstadoDoAlarme {
  const [som, setSom] = useState<SomDeAlarme | null>(null);
  const somRef = useRef<SomDeAlarme | null>(null);
  somRef.current = som;

  useEffect(() => {
    let ativo = true;
    commands.obterSomDeAlarme().then((resposta) => {
      if (ativo && resposta.status === "ok") setSom(resposta.data);
    });
    const inscricaoMudou = events.somDeAlarmeMudou.listen((evento) => setSom(evento.payload));
    const inscricaoTocar = events.tocarAlarme.listen(() => {
      if (somRef.current) tocar(somRef.current);
    });
    return () => {
      ativo = false;
      inscricaoMudou.then((parar) => parar());
      inscricaoTocar.then((parar) => parar());
    };
  }, []);

  return {
    som,
    alterarSom: (novo) => void commands.alterarSomDeAlarme(novo),
    tocarPreview: tocar,
  };
}
