import type { CSSProperties } from "react";

interface Props {
  ativo: boolean;
}

export function OverlayDeTomadaDeFoco({ ativo }: Props) {
  return (
    <div style={{ ...estilos.overlay, opacity: ativo ? 1 : 0 }} aria-hidden="true">
      <span style={estilos.rotulo}>trazendo a janela para frente…</span>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  overlay: {
    position: "absolute",
    inset: 0,
    zIndex: 30,
    background: "#000",
    pointerEvents: "none",
    display: "grid",
    placeItems: "center",
    transition: "opacity .35s ease",
  },
  rotulo: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 11.5,
    letterSpacing: ".28em",
    textTransform: "uppercase",
    color: "#8d8c92",
  },
};
