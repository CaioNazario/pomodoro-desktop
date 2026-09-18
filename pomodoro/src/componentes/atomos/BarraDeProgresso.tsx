import type { CSSProperties } from "react";

interface Props {
  progresso: number;
  cor: string;
  altura?: number;
  corDeFundo?: string;
  transicao?: boolean;
  estilo?: CSSProperties;
}

export function BarraDeProgresso({
  progresso,
  cor,
  altura = 3,
  corDeFundo = "rgba(255,255,255,0.1)",
  transicao = true,
  estilo,
}: Props) {
  return (
    <div style={{ height: altura, borderRadius: 999, background: corDeFundo, overflow: "hidden", ...estilo }}>
      <div
        style={{
          height: "100%",
          width: `${progresso * 100}%`,
          background: cor,
          ...(transicao ? { transition: "width .2s linear" } : {}),
        }}
      />
    </div>
  );
}
