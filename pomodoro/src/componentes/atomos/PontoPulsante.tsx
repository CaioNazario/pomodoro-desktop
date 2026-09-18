interface Props {
  cor: string;
  tamanho?: number;
}

export function PontoPulsante({ cor, tamanho = 7 }: Props) {
  return (
    <span
      style={{
        width: tamanho,
        height: tamanho,
        borderRadius: "50%",
        background: cor,
        animation: "breathe 3.4s ease-in-out infinite",
        flexShrink: 0,
      }}
    />
  );
}
