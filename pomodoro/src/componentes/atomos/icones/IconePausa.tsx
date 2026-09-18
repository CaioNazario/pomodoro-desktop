interface Props {
  tamanho?: number;
}

export function IconePausa({ tamanho = 22 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 24 24" aria-hidden="true">
      <rect x="7" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
      <rect x="13.6" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
    </svg>
  );
}
