interface Props {
  tamanho?: number;
}

export function IconeReset({ tamanho = 17 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M4 10a6 6 0 1 0 2.2-4.65" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" />
      <path
        d="M3.6 3.6v3.4h3.4"
        stroke="currentColor"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
