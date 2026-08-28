import type { CSSProperties } from "react";
import { commands, type ModoDeDuracao } from "../../bindings";

interface Props {
  modo: ModoDeDuracao;
}

function trocarModo(novo: ModoDeDuracao): void {
  void commands.trocarModo(novo);
}

function estiloDaAba(ativa: boolean): CSSProperties {
  return {
    flex: 1,
    padding: "6px 0",
    border: "none",
    borderRadius: 7,
    background: ativa ? "rgba(255,255,255,0.1)" : "transparent",
    color: ativa ? "#eceae5" : "#87868c",
    font: "500 11.5px 'Geist Mono', monospace",
    letterSpacing: ".1em",
    textTransform: "uppercase",
    cursor: "pointer",
  };
}

export function SeletorDeModo({ modo }: Props) {
  return (
    <div style={estilos.trilho}>
      <button style={estiloDaAba(modo === "global")} onClick={() => trocarModo("global")}>
        Global
      </button>
      <button style={estiloDaAba(modo === "individual")} onClick={() => trocarModo("individual")}>
        Individual
      </button>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  trilho: {
    display: "flex",
    gap: 4,
    margin: "4px 12px 8px",
    padding: 3,
    borderRadius: 9,
    background: "rgba(255,255,255,0.045)",
  },
};
