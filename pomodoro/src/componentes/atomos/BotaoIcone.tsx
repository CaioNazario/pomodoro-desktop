import type { CSSProperties, ReactNode } from "react";

interface Props {
  tamanho: number;
  formato?: "circulo" | "arredondado";
  titulo?: string;
  estilo?: CSSProperties;
  onClick?: () => void;
  children: ReactNode;
}

export function BotaoIcone({ tamanho, formato = "circulo", titulo, estilo, onClick, children }: Props) {
  return (
    <button
      title={titulo}
      onClick={onClick}
      style={{
        width: tamanho,
        height: tamanho,
        display: "grid",
        placeItems: "center",
        borderRadius: formato === "circulo" ? "50%" : 11,
        cursor: "pointer",
        ...estilo,
      }}
    >
      {children}
    </button>
  );
}
