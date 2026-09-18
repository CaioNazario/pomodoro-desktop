import type { CSSProperties } from "react";

interface Props {
  rotuloEstado: string;
  restanteMs: number;
}

function formatarRelogio(ms: number): string {
  const segundos = Math.floor(ms / 1000);
  const mm = String(Math.floor(segundos / 60)).padStart(2, "0");
  const ss = String(segundos % 60).padStart(2, "0");
  return `${mm}:${ss}`;
}

function formatarTerminaEm(ms: number): string {
  const fim = new Date(Date.now() + ms);
  const hh = String(fim.getHours()).padStart(2, "0");
  const min = String(fim.getMinutes()).padStart(2, "0");
  return `${hh}:${min}`;
}

export function Relogio({ rotuloEstado, restanteMs }: Props) {
  return (
    <div style={estilos.overlay}>
      <span style={estilos.rotulo}>{rotuloEstado}</span>
      <span style={estilos.clock}>{formatarRelogio(restanteMs)}</span>
      <span style={estilos.termina}>termina {formatarTerminaEm(restanteMs)}</span>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  overlay: {
    position: "absolute",
    inset: 0,
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: 10,
  },
  rotulo: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 11,
    letterSpacing: ".3em",
    textTransform: "uppercase",
    color: "#5f5f66",
  },
  clock: {
    fontFamily: "Geist, sans-serif",
    fontWeight: 300,
    fontSize: "min(11vh, 104px)",
    lineHeight: 0.92,
    letterSpacing: "-0.035em",
    fontVariantNumeric: "tabular-nums",
    color: "#fbfaf7",
  },
  termina: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 12,
    letterSpacing: ".1em",
    color: "#6c6c73",
  },
};
