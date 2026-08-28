import type { CSSProperties } from "react";
import type { Etapa } from "../bindings";
import { acentoDaEtapa } from "./tema";

interface Props {
  etapa: Etapa;
  rodando: boolean;
  onAlternarExecucao: () => void;
  onReiniciar: () => void;
  onPular: () => void;
}

export function ControlesDeTimer({ etapa, rodando, onAlternarExecucao, onReiniciar, onPular }: Props) {
  const acento = acentoDaEtapa(etapa);
  return (
    <div style={estilos.linha}>
      <button title="Reiniciar (R)" style={estilos.secundario} onClick={onReiniciar}>
        <IconeReset />
      </button>
      <button
        title="Iniciar / pausar (Espaço)"
        style={{ ...estilos.primario, background: acento }}
        onClick={onAlternarExecucao}
      >
        {rodando ? <IconePausa /> : <IconePlay />}
      </button>
      <button title="Pular sessão (S)" style={estilos.secundario} onClick={onPular}>
        <IconeSkip />
      </button>
    </div>
  );
}

function IconeReset() {
  return (
    <svg width="17" height="17" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M4 10a6 6 0 1 0 2.2-4.65" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" />
      <path
        d="M3.6 3.6v3.4h3.4"
        stroke="currentColor"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function IconePlay() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 5.2 18.4 12 8 18.8z" fill="currentColor" />
    </svg>
  );
}

function IconePausa() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" aria-hidden="true">
      <rect x="7" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
      <rect x="13.6" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
    </svg>
  );
}

function IconeSkip() {
  return (
    <svg width="17" height="17" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M5 4.5 13 10l-8 5.5z" fill="currentColor" />
      <rect x="14.2" y="4.5" width="1.6" height="11" fill="currentColor" />
    </svg>
  );
}

const estilos: Record<string, CSSProperties> = {
  linha: { display: "flex", alignItems: "center", gap: 20 },
  secundario: {
    width: 54,
    height: 54,
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "1px solid rgba(255,255,255,0.1)",
    background: "rgba(255,255,255,0.025)",
    color: "#bfbeb9",
    cursor: "pointer",
  },
  primario: {
    width: 76,
    height: 76,
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "none",
    color: "#100b05",
    cursor: "pointer",
    boxShadow: "0 10px 34px rgba(0,0,0,0.55)",
  },
};
