import type { CSSProperties, MouseEvent as EventoDoMouseReact } from "react";
import { BarraDeProgresso } from "../atomos/BarraDeProgresso";
import { PontoPulsante } from "../atomos/PontoPulsante";

// Sem tamanho fixo no design (flex + padding, conteudo define a largura) —
// estimado pra caber dot + relogio + barra com folga, sem medida de pixel
// exata pra portar (sem screenshot). Ver CartaoDoWidget sobre a janela fixa
// que os dois compartilham.
const LARGURA = 138;
const ALTURA = 38;

interface Props {
  acento: string;
  relogio: string;
  progresso: number;
  onPressionar: (evento: EventoDoMouseReact) => void;
  onClicar: () => void;
}

export function PilulaDoWidget({ acento, relogio, progresso, onPressionar, onClicar }: Props) {
  return (
    <button onMouseDown={onPressionar} onClick={onClicar} title="Abrir widget" style={estilos.pilula}>
      <PontoPulsante cor={acento} />
      <span style={estilos.relogio}>{relogio}</span>
      <BarraDeProgresso
        progresso={progresso}
        cor={acento}
        corDeFundo="rgba(255,255,255,0.14)"
        transicao={false}
        estilo={{ width: 26, flexShrink: 0 }}
      />
    </button>
  );
}

const estilos: Record<string, CSSProperties> = {
  pilula: {
    boxSizing: "border-box",
    width: LARGURA,
    height: ALTURA,
    display: "flex",
    alignItems: "center",
    gap: 10,
    padding: "9px 14px 9px 11px",
    borderRadius: 999,
    background: "#000000",
    border: "1px solid rgba(255,255,255,0.12)",
    color: "#f3f2ef",
    fontFamily: "Geist, sans-serif",
    cursor: "pointer",
    userSelect: "none",
  },
  relogio: {
    fontWeight: 450,
    fontSize: 17,
    letterSpacing: "-0.01em",
    fontVariantNumeric: "tabular-nums",
    color: "#f6f5f1",
    whiteSpace: "nowrap",
  },
};
