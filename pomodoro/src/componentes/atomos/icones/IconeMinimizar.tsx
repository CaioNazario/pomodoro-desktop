interface Props {
  tamanho?: number;
}

export function IconeMinimizar({ tamanho = 9 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 10 10" aria-hidden="true">
      <rect x="1" y="4.4" width="8" height="1.3" rx="0.65" fill="currentColor" />
    </svg>
  );
}
