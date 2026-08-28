import { memo, useMemo, type CSSProperties } from "react";
import type { Etapa } from "../bindings";
import { acentoDaEtapa } from "./tema";

interface Props {
  etapa: Etapa;
  progresso: number;
}

const RAIO = 226;
const CIRCUNFERENCIA = 2 * Math.PI * RAIO;

interface Tique {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  largura: number;
  cor: string;
}

function calcularTiques(progresso: number, acento: string): Tique[] {
  return Array.from({ length: 60 }, (_, i) => {
    const angulo = ((i * 6 - 90) * Math.PI) / 180;
    const maior = i % 5 === 0;
    const interno = maior ? 190 : 200;
    const passou = i / 60 <= progresso;
    return {
      x1: 240 + Math.cos(angulo) * interno,
      y1: 240 + Math.sin(angulo) * interno,
      x2: 240 + Math.cos(angulo) * 212,
      y2: 240 + Math.sin(angulo) * 212,
      largura: maior ? 2 : 1,
      cor: passou ? acento : maior ? "rgba(255,255,255,0.30)" : "rgba(255,255,255,0.13)",
    };
  });
}

export const AnelDeProgresso = memo(function AnelDeProgresso({ etapa, progresso }: Props) {
  const acento = acentoDaEtapa(etapa);
  const tiques = useMemo(() => calcularTiques(progresso, acento), [progresso, acento]);
  return (
    <svg viewBox="0 0 480 480" style={estilos.svg}>
      <circle cx={240} cy={240} r={RAIO} fill="none" stroke="rgba(255,255,255,0.07)" strokeWidth={2} />
      <g strokeLinecap="butt">
        {tiques.map((t, i) => (
          <line key={i} x1={t.x1} y1={t.y1} x2={t.x2} y2={t.y2} stroke={t.cor} strokeWidth={t.largura} />
        ))}
      </g>
      <circle
        cx={240}
        cy={240}
        r={RAIO}
        fill="none"
        stroke={acento}
        strokeWidth={3}
        strokeLinecap="round"
        strokeDasharray={CIRCUNFERENCIA}
        strokeDashoffset={CIRCUNFERENCIA * (1 - progresso)}
        transform="rotate(-90 240 240)"
        opacity={0.95}
      />
    </svg>
  );
});

const estilos: Record<string, CSSProperties> = {
  svg: { width: "100%", height: "100%", display: "block", overflow: "visible" },
};
