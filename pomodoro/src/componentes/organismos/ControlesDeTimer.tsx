import type { CSSProperties } from "react";
import type { Etapa } from "../../bindings";
import { BotaoIcone } from "../atomos/BotaoIcone";
import { IconePausa } from "../atomos/icones/IconePausa";
import { IconePlay } from "../atomos/icones/IconePlay";
import { IconeReset } from "../atomos/icones/IconeReset";
import { IconeSkip } from "../atomos/icones/IconeSkip";
import { acentoDaEtapa } from "../tema";

interface Props {
  etapa: Etapa;
  rodando: boolean;
  onAlternarExecucao: () => void;
  onReiniciar: () => void;
  onPular: () => void;
}

export function ControlesDeTimer({ etapa, rodando, onAlternarExecucao, onReiniciar, onPular }: Props) {
  const acento = acentoDaEtapa(etapa);
  return (
    <div style={estilos.linha}>
      <BotaoIcone tamanho={54} titulo="Reiniciar (R)" estilo={estilos.secundario} onClick={onReiniciar}>
        <IconeReset />
      </BotaoIcone>
      <BotaoIcone
        tamanho={76}
        titulo="Iniciar / pausar (Espaço)"
        estilo={{ ...estilos.primario, background: acento }}
        onClick={onAlternarExecucao}
      >
        {rodando ? <IconePausa /> : <IconePlay />}
      </BotaoIcone>
      <BotaoIcone tamanho={54} titulo="Pular sessão (S)" estilo={estilos.secundario} onClick={onPular}>
        <IconeSkip />
      </BotaoIcone>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  linha: { display: "flex", alignItems: "center", gap: 20 },
  secundario: {
    border: "1px solid rgba(255,255,255,0.1)",
    background: "rgba(255,255,255,0.025)",
    color: "#bfbeb9",
  },
  primario: {
    border: "none",
    color: "#100b05",
    boxShadow: "0 10px 34px rgba(0,0,0,0.55)",
  },
};
