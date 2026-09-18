import type { CSSProperties } from "react";
import type { EstadoDaTela } from "../bindings";
import { BotaoIcone } from "./atomos/BotaoIcone";
import { IconeMenu } from "./atomos/icones/IconeMenu";
import { PontoPulsante } from "./atomos/PontoPulsante";
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
        <BotaoIcone tamanho={38} formato="arredondado" titulo="Menu (M)" estilo={estilos.botaoIcone} onClick={onAbrirMenu}>
          <IconeMenu />
        </BotaoIcone>
        <div style={estilos.badge}>
          <PontoPulsante cor={acento} />
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
    border: "1px solid rgba(255,255,255,0.09)",
    background: "rgba(255,255,255,0.03)",
    color: "#cfcec9",
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
