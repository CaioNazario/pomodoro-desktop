interface Props {
  tamanho?: number;
}

export function IconeSkip({ tamanho = 17 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M5 4.5 13 10l-8 5.5z" fill="currentColor" />
      <rect x="14.2" y="4.5" width="1.6" height="11" fill="currentColor" />
    </svg>
  );
}
