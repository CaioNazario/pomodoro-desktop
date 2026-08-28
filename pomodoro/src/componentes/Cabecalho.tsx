import type { CSSProperties } from "react";
import type { EstadoDaTela } from "../bindings";
import { acentoDaEtapa } from "./tema";

interface Props {
  estado: EstadoDaTela;
  onAbrirMenu: () => void;
  onAlternarWidget: () => void;
}

const rotuloModo = { foco: "Foco", pausa: "Pausa" } as const;

export function Cabecalho({ estado, onAbrirMenu, onAlternarWidget }: Props) {
  const acento = acentoDaEtapa(estado.etapa);
  return (
    <header style={estilos.header}>
      <div style={estilos.grupoEsquerda}>
        <button title="Menu (M)" style={estilos.botaoIcone} onClick={onAbrirMenu}>
          <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
            <rect x="2" y="4" width="12" height="1.4" fill="currentColor" />
            <rect x="2" y="7.3" width="12" height="1.4" fill="currentColor" />
            <rect x="2" y="10.6" width="12" height="1.4" fill="currentColor" />
          </svg>
        </button>
        <div style={estilos.badge}>
          <span style={{ ...estilos.pontoPulsante, background: acento }} />
          <span style={estilos.modoTexto}>{rotuloModo[estado.etapa]}</span>
          <span style={estilos.separador} />
          <span style={estilos.sessaoTexto}>
            Sessão {estado.numeroSessao} de {estado.totalSessoes}
          </span>
        </div>
      </div>
      <div style={estilos.grupoDireita}>
        <button style={estilos.botaoWidget} onClick={onAlternarWidget}>
          Widget
        </button>
      </div>
    </header>
  );
}

const estilos: Record<string, CSSProperties> = {
  header: {
    position: "relative",
    zIndex: 3,
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "22px 28px",
    gap: 16,
  },
  grupoEsquerda: { display: "flex", alignItems: "center", gap: 14 },
  botaoIcone: {
    width: 38,
    height: 38,
    display: "grid",
    placeItems: "center",
    borderRadius: 11,
    border: "1px solid rgba(255,255,255,0.09)",
    background: "rgba(255,255,255,0.03)",
    color: "#cfcec9",
    cursor: "pointer",
  },
  badge: {
    display: "flex",
    alignItems: "center",
    gap: 10,
    padding: "8px 14px 8px 12px",
    borderRadius: 999,
    border: "1px solid rgba(255,255,255,0.07)",
    background: "rgba(255,255,255,0.02)",
  },
  pontoPulsante: {
    width: 7,
    height: 7,
    borderRadius: "50%",
    animation: "breathe 3.4s ease-in-out infinite",
  },
  modoTexto: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 13.5,
    letterSpacing: ".16em",
    textTransform: "uppercase",
    color: "#dedcd6",
  },
  separador: { width: 1, height: 12, background: "rgba(255,255,255,0.12)" },
  sessaoTexto: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 13.5,
    letterSpacing: ".08em",
    color: "#b6b5b0",
    whiteSpace: "nowrap",
  },
  grupoDireita: {
    display: "flex",
    alignItems: "center",
    gap: 22,
    fontFamily: "'Geist Mono', monospace",
    fontSize: 13,
    letterSpacing: ".12em",
    color: "#b6b5b0",
    textTransform: "uppercase",
  },
  botaoWidget: {
    font: "inherit",
    letterSpacing: ".12em",
    textTransform: "uppercase",
    padding: "7px 12px",
    borderRadius: 9,
    border: "1px solid rgba(255,255,255,0.09)",
    background: "rgba(255,255,255,0.02)",
    color: "#dedcd6",
    cursor: "pointer",
  },
};
