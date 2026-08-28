import type { CSSProperties } from "react";

interface Props {
  ativo: boolean;
  onAlterar: (valor: boolean) => void;
}

export function ToggleAutoStart({ ativo, onAlterar }: Props) {
  return (
    <button style={estilos.botao} onClick={() => onAlterar(!ativo)}>
      <span style={estilos.rotulo}>Iniciar automaticamente</span>
      <span style={{ ...estilos.trilho, background: ativo ? "#e0dfda" : "rgba(255,255,255,0.14)" }}>
        <span style={{ ...estilos.bolinha, transform: ativo ? "translateX(13px)" : "translateX(0)" }} />
      </span>
    </button>
  );
}

const estilos: Record<string, CSSProperties> = {
  botao: {
    width: "100%",
    display: "flex",
    alignItems: "center",
    gap: 11,
    padding: "9px 12px",
    border: "none",
    borderRadius: 9,
    background: "transparent",
    color: "#eceae5",
    font: "400 13.5px Geist, sans-serif",
    textAlign: "left",
    cursor: "pointer",
  },
  rotulo: { flex: 1 },
  trilho: {
    width: 30,
    height: 17,
    borderRadius: 999,
    display: "flex",
    alignItems: "center",
    padding: 2,
    transition: "background .18s ease",
  },
  bolinha: {
    width: 13,
    height: 13,
    borderRadius: "50%",
    background: "#0c0c0e",
    transition: "transform .18s ease",
  },
};
