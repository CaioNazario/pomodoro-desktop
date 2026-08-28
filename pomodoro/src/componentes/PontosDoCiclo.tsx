import type { CSSProperties } from "react";
import type { Etapa } from "../bindings";
import { acentoDaEtapa } from "./tema";

interface Props {
  etapa: Etapa;
  sessaoAtual: number;
  totalSessoes: number;
}

export function PontosDoCiclo({ etapa, sessaoAtual, totalSessoes }: Props) {
  const acento = acentoDaEtapa(etapa);
  const pontos = Array.from({ length: totalSessoes }, (_, indice) => {
    const numero = indice + 1;
    const ativo = numero === sessaoAtual;
    const cor = ativo ? acento : numero < sessaoAtual ? "rgba(255,255,255,0.32)" : "rgba(255,255,255,0.12)";
    return { numero, largura: ativo ? 22 : 6, cor };
  });
  return (
    <div style={estilos.linha}>
      {pontos.map((p) => (
        <span key={p.numero} style={{ ...estilos.ponto, width: p.largura, background: p.cor }} />
      ))}
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  linha: { display: "flex", alignItems: "center", gap: 10 },
  ponto: { height: 6, borderRadius: 999 },
};
