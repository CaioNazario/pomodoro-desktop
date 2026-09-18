interface Props {
  tamanho?: number;
}

export function IconeMenu({ tamanho = 16 }: Props) {
  return (
    <svg width={tamanho} height={tamanho} viewBox="0 0 16 16" aria-hidden="true">
      <rect x="2" y="4" width="12" height="1.4" fill="currentColor" />
      <rect x="2" y="7.3" width="12" height="1.4" fill="currentColor" />
      <rect x="2" y="10.6" width="12" height="1.4" fill="currentColor" />
    </svg>
  );
}
