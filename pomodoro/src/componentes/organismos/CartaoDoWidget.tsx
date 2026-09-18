import type { CSSProperties } from "react";
import { BarraDeProgresso } from "../atomos/BarraDeProgresso";
import { BotaoIcone } from "../atomos/BotaoIcone";
import { IconeMinimizar } from "../atomos/icones/IconeMinimizar";
import { IconePausa } from "../atomos/icones/IconePausa";
import { IconePlay } from "../atomos/icones/IconePlay";
import { IconePontos } from "../atomos/icones/IconePontos";
import { IconeReset } from "../atomos/icones/IconeReset";

// 262x150: medida do design canvas (`Pomodoro Desktop.dc.html`, linha 227:
// `const w = this._drag.w || 262, h = this._drag.h || 150`). A janela real
// (criada em `src-tauri/src/widget/mod.rs`) NAO redimensiona entre cartao e
// pilula — webkit2gtk impoe um piso de ~200px por dimensao que ignora
// inner_size/min_inner_size, entao os dois sao desenhados com seu proprio
// tamanho dentro da mesma janela fixa (262x200).
const LARGURA = 262;
const ALTURA = 150;

interface Props {
  acento: string;
  modeLabel: string;
  cycleLabel: string;
  relogio: string;
  progresso: number;
  rodando: boolean;
  onMinimizar: () => void;
  onFechar: () => void;
  onReiniciar: () => void;
  onAlternarExecucao: () => void;
}

export function CartaoDoWidget({
  acento,
  modeLabel,
  cycleLabel,
  relogio,
  progresso,
  rodando,
  onMinimizar,
  onFechar,
  onReiniciar,
  onAlternarExecucao,
}: Props) {
  return (
    <div style={estilos.cartao}>
      <div data-tauri-drag-region="deep" style={estilos.cabecalho}>
        <span style={estilos.rotuloCabecalho}>
          <IconePontos />
          {modeLabel} · {cycleLabel}
        </span>
        <div style={estilos.botoesCabecalho}>
          <BotaoIcone tamanho={20} titulo="Minimizar" estilo={estilos.botaoCirculo} onClick={onMinimizar}>
            <IconeMinimizar />
          </BotaoIcone>
          <BotaoIcone tamanho={20} titulo="Fechar" estilo={estilos.botaoCirculo} onClick={onFechar}>
            ×
          </BotaoIcone>
        </div>
      </div>
      <div style={estilos.linhaRelogio}>
        <span style={estilos.relogioGrande}>{relogio}</span>
        <div style={estilos.botoesControle}>
          <BotaoIcone tamanho={34} titulo="Reiniciar" estilo={estilos.botaoReset} onClick={onReiniciar}>
            <IconeReset tamanho={13} />
          </BotaoIcone>
          <BotaoIcone
            tamanho={44}
            titulo="Iniciar / pausar"
            estilo={{ ...estilos.botaoPlay, background: acento }}
            onClick={onAlternarExecucao}
          >
            {rodando ? <IconePausa tamanho={14} /> : <IconePlay tamanho={14} />}
          </BotaoIcone>
        </div>
      </div>
      <BarraDeProgresso progresso={progresso} cor={acento} estilo={{ marginTop: 14 }} />
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  cartao: {
    boxSizing: "border-box",
    width: LARGURA,
    height: ALTURA,
    padding: "15px 17px 14px",
    borderRadius: 20,
    background: "#000000",
    border: "1px solid rgba(255,255,255,0.12)",
    color: "#f3f2ef",
    fontFamily: "Geist, 'Helvetica Neue', Helvetica, sans-serif",
    userSelect: "none",
  },
  cabecalho: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    margin: "-15px -17px 6px",
    padding: "13px 17px 7px",
    cursor: "grab",
  },
  rotuloCabecalho: {
    display: "flex",
    alignItems: "center",
    gap: 8,
    fontFamily: "'Geist Mono', monospace",
    fontSize: 10.5,
    letterSpacing: ".22em",
    textTransform: "uppercase",
    color: "#6e6e75",
    whiteSpace: "nowrap",
  },
  botoesCabecalho: { display: "flex", alignItems: "center", gap: 6 },
  botaoCirculo: {
    border: "none",
    background: "rgba(255,255,255,0.08)",
    color: "#9a9a9f",
    fontSize: 11,
    lineHeight: 1,
  },
  linhaRelogio: { display: "flex", alignItems: "center", justifyContent: "space-between", gap: 14 },
  relogioGrande: {
    fontFamily: "Geist, sans-serif",
    fontWeight: 300,
    fontSize: 44,
    lineHeight: 1,
    letterSpacing: "-0.04em",
    fontVariantNumeric: "tabular-nums",
    color: "#ffffff",
  },
  botoesControle: { display: "flex", alignItems: "center", gap: 8 },
  botaoReset: {
    border: "1px solid rgba(255,255,255,0.14)",
    background: "transparent",
    color: "#b9b8b2",
  },
  botaoPlay: {
    border: "none",
    color: "#100b05",
  },
};
