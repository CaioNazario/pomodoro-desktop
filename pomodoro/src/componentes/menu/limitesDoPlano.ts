/// Espelha os limites fixos no dominio (PRD §6, decisao 9) — validacao real
/// e sempre no Rust; aqui e so pra desabilitar os botoes +/- na borda, sem
/// esperar o erro de comando pra dar feedback.
export const SESSOES_MIN = 1;
export const SESSOES_MAX = 24;
export const FOCO_MIN_MIN = 5;
export const FOCO_MIN_MAX = 180;
export const FOCO_PASSO_MIN = 5;
export const PAUSA_MIN_MIN = 1;
export const PAUSA_MIN_MAX = 60;
export const PAUSA_PASSO_MIN = 1;

export function emMinutos(ms: number): number {
  return Math.round(ms / 60_000);
}
