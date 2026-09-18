import type { CSSProperties, ReactNode } from "react";

interface Props {
  rotulo: string;
  cor?: string;
  onClick?: () => void;
  desabilitado?: boolean;
  filho?: ReactNode;
}

export function ItemDeMenu({ rotulo, cor = "#eceae5", onClick, desabilitado = false, filho }: Props) {
  return (
    <button
      style={{ ...estilos.item, color: cor, cursor: desabilitado ? "default" : "pointer" }}
      onClick={onClick}
      tabIndex={desabilitado ? -1 : undefined}
      aria-disabled={desabilitado || undefined}
    >
      <span style={estilos.rotulo}>{rotulo}</span>
      {filho}
    </button>
  );
}

const estilos: Record<string, CSSProperties> = {
  item: {
    width: "100%",
    display: "flex",
    alignItems: "center",
    gap: 11,
    padding: "9px 12px",
    border: "none",
    borderRadius: 9,
    background: "transparent",
    font: "400 13.5px Geist, sans-serif",
    textAlign: "left",
  },
  rotulo: { flex: 1 },
};
