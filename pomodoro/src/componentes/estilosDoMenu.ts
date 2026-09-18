import type { CSSProperties } from "react";

/// Tokens compartilhados pelos blocos do corpo do menu lateral — paridade de
/// valor computado com `Pomodoro Desktop.dc.html` (linhas ~120-200).
export const estilosDoMenu: Record<string, CSSProperties> = {
  linha: {
    display: "flex",
    alignItems: "center",
    gap: 11,
    padding: "8px 12px",
    color: "#eceae5",
    fontSize: 13.5,
  },
  rotulo: { flex: 1, whiteSpace: "nowrap" },
  passo: { display: "flex", alignItems: "center", gap: 2 },
  botaoPasso: {
    width: 22,
    height: 22,
    borderRadius: 7,
    border: "1px solid rgba(255,255,255,0.1)",
    background: "transparent",
    color: "#cfcec9",
    cursor: "pointer",
    lineHeight: 1,
  },
  valor: {
    minWidth: 54,
    textAlign: "center",
    fontFamily: "'Geist Mono', monospace",
    fontSize: 12,
    color: "#b9b8b2",
  },
  divisor: { height: 1, margin: "7px 10px", background: "rgba(255,255,255,0.08)" },
  cabecalhoSecao: {
    display: "flex",
    alignItems: "baseline",
    justifyContent: "space-between",
    padding: "8px 12px 6px",
  },
  tituloSecao: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 10.5,
    letterSpacing: ".22em",
    textTransform: "uppercase",
    color: "#56565d",
  },
};
