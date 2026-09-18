import type { CSSProperties } from "react";

interface Props {
  ativo: boolean;
}

export function Interruptor({ ativo }: Props) {
  return (
    <span style={{ ...estilos.trilho, background: ativo ? "#e0dfda" : "rgba(255,255,255,0.14)" }}>
      <span style={{ ...estilos.bolinha, transform: ativo ? "translateX(13px)" : "translateX(0)" }} />
    </span>
  );
}

const estilos: Record<string, CSSProperties> = {
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
