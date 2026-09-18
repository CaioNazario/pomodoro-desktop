interface Props {
  tamanho?: number;
}

export function IconePlay({ tamanho = 22 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 5.2 18.4 12 8 18.8z" fill="currentColor" />
    </svg>
  );
}
