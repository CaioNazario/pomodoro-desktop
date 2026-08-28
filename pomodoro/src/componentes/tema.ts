import type { Etapa } from "../bindings";

/// Tokens do contrato visual do PRD (12.1). Paridade de valor computado com
/// `Pomodoro Desktop.dc.html`, nao classe/cascata.
export const ACENTO_FOCO = "oklch(0.79 0.135 62)";
export const ACENTO_PAUSA = "oklch(0.80 0.09 178)";

export function acentoDaEtapa(etapa: Etapa): string {
  return etapa === "foco" ? ACENTO_FOCO : ACENTO_PAUSA;
}
