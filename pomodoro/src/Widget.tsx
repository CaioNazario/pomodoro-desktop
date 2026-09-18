import "./Widget.css";
import { useCallback, useRef, useState, type CSSProperties, type MouseEvent as EventoDoMouseReact } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { BarraDeProgresso } from "./componentes/atomos/BarraDeProgresso";
import { BotaoIcone } from "./componentes/atomos/BotaoIcone";
import { IconeMinimizar } from "./componentes/atomos/icones/IconeMinimizar";
import { IconePausa } from "./componentes/atomos/icones/IconePausa";
import { IconePlay } from "./componentes/atomos/icones/IconePlay";
import { IconePontos } from "./componentes/atomos/icones/IconePontos";
import { IconeReset } from "./componentes/atomos/icones/IconeReset";
import { PontoPulsante } from "./componentes/atomos/PontoPulsante";
import { acentoDaEtapa } from "./componentes/tema";
import { useEstadoDoTimer } from "./useEstadoDoTimer";

// Medidas do design canvas (`Pomodoro Desktop.dc.html`, linhas 227 e 381):
// `const w = this._drag.w || 262, h = this._drag.h || 150`. A janela real
// (criada em `src-tauri/src/widget/mod.rs`) NAO redimensiona entre os dois
// estados — achado ao vivo: `webkit2gtk` impoe um piso de ~200px por
// dimensao que ignora `inner_size`/`min_inner_size`, entao o card e a
// pilula sao desenhados com seu proprio tamanho dentro de uma janela fixa
// (262x200), ancorados no topo-esquerda como no design.
const LARGURA_EXPANDIDO = 262;
const ALTURA_EXPANDIDO = 150;
// A pilula minimizada nao tem tamanho fixo no design (flex + padding,
// conteudo define a largura) — estimado pra caber dot + relogio + barra
// com folga, sem medida de pixel exata pra portar (sem screenshot).
const LARGURA_MINIMIZADO = 138;
const ALTURA_MINIMIZADO = 38;

function formatarMMSS(ms: number): string {
  const segundos = Math.floor(ms / 1000);
  const mm = String(Math.floor(segundos / 60)).padStart(2, "0");
  const ss = String(segundos % 60).padStart(2, "0");
  return `${mm}:${ss}`;
}

// A pilula minimizada precisa ser arrastavel E clicavel (pra restaurar o
// widget). O `data-tauri-drag-region` do Tauri nao da pra ter os dois: ele
// decide arrasto vs clique no proprio `mousedown`, antes de saber se o
// usuario vai mover o mouse — dispara `start_dragging()` incondicionalmente
// e o grab nativo do window manager engole o `click` seguinte mesmo parado.
// Aqui a decisao e manual, por limiar de deslocamento: só chama
// `startDragging()` se o mouse realmente se moveu; senao deixa o `onClick`
// normal restaurar o widget.
const LIMIAR_ARRASTO_PX = 4;

function useArrastoDaPilula(aoClicar: () => void) {
  const emArrastoRef = useRef(false);

  const aoPressionar = useCallback((evento: EventoDoMouseReact) => {
    if (evento.button !== 0) return;
    const inicioX = evento.clientX;
    const inicioY = evento.clientY;
    emArrastoRef.current = false;

    function limpar() {
      window.removeEventListener("mousemove", aoMover);
      window.removeEventListener("mouseup", aoSoltar);
    }

    function aoMover(e: MouseEvent) {
      if (Math.hypot(e.clientX - inicioX, e.clientY - inicioY) <= LIMIAR_ARRASTO_PX) return;
      emArrastoRef.current = true;
      limpar();
      void getCurrentWindow().startDragging();
    }

    function aoSoltar() {
      limpar();
    }

    window.addEventListener("mousemove", aoMover);
    window.addEventListener("mouseup", aoSoltar);
  }, []);

  const aoClicarBotao = useCallback(() => {
    if (!emArrastoRef.current) aoClicar();
  }, [aoClicar]);

  return { aoPressionar, aoClicarBotao };
}

export function Widget() {
  const { estado, restanteMs, alternarExecucao, reiniciarEtapa } = useEstadoDoTimer();
  const [minimizado, setMinimizado] = useState(false);
  const { aoPressionar, aoClicarBotao } = useArrastoDaPilula(() => setMinimizado(false));

  if (!estado) {
    return null;
  }

  const acento = acentoDaEtapa(estado.etapa);
  const decorridoMs = estado.duracaoTotalMs - restanteMs;
  const progresso = estado.duracaoTotalMs > 0 ? Math.min(1, Math.max(0, decorridoMs / estado.duracaoTotalMs)) : 0;
  const modeLabel = estado.etapa === "foco" ? "Foco" : "Pausa";
  const cycleLabel = `${estado.numeroSessao}/${estado.totalSessoes}`;
  const relogio = formatarMMSS(restanteMs);

  if (minimizado) {
    return (
      <button onMouseDown={aoPressionar} onClick={aoClicarBotao} title="Abrir widget" style={estilos.pilula}>
        <PontoPulsante cor={acento} />
        <span style={estilos.relogioPilula}>{relogio}</span>
        <BarraDeProgresso
          progresso={progresso}
          cor={acento}
          corDeFundo="rgba(255,255,255,0.14)"
          transicao={false}
          estilo={{ width: 26, flexShrink: 0 }}
        />
      </button>
    );
  }

  return (
    <div style={estilos.cartao}>
      <div data-tauri-drag-region="deep" style={estilos.cabecalho}>
        <span style={estilos.rotuloCabecalho}>
          <IconePontos />
          {modeLabel} · {cycleLabel}
        </span>
        <div style={estilos.botoesCabecalho}>
          <BotaoIcone tamanho={20} titulo="Minimizar" estilo={estilos.botaoCirculo} onClick={() => setMinimizado(true)}>
            <IconeMinimizar />
          </BotaoIcone>
          <BotaoIcone
            tamanho={20}
            titulo="Fechar"
            estilo={estilos.botaoCirculo}
            onClick={() => getCurrentWindow().close()}
          >
            ×
          </BotaoIcone>
        </div>
      </div>
      <div style={estilos.linhaRelogio}>
        <span style={estilos.relogioGrande}>{relogio}</span>
        <div style={estilos.botoesControle}>
          <BotaoIcone tamanho={34} titulo="Reiniciar" estilo={estilos.botaoReset} onClick={reiniciarEtapa}>
            <IconeReset tamanho={13} />
          </BotaoIcone>
          <BotaoIcone
            tamanho={44}
            titulo="Iniciar / pausar"
            estilo={{ ...estilos.botaoPlay, background: acento }}
            onClick={alternarExecucao}
          >
            {estado.rodando ? <IconePausa tamanho={14} /> : <IconePlay tamanho={14} />}
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
    width: LARGURA_EXPANDIDO,
    height: ALTURA_EXPANDIDO,
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
  pilula: {
    boxSizing: "border-box",
    width: LARGURA_MINIMIZADO,
    height: ALTURA_MINIMIZADO,
    display: "flex",
    alignItems: "center",
    gap: 10,
    padding: "9px 14px 9px 11px",
    borderRadius: 999,
    background: "#000000",
    border: "1px solid rgba(255,255,255,0.12)",
    color: "#f3f2ef",
    fontFamily: "Geist, sans-serif",
    cursor: "pointer",
    userSelect: "none",
  },
  relogioPilula: {
    fontWeight: 400,
    fontSize: 17,
    letterSpacing: "-0.01em",
    fontVariantNumeric: "tabular-nums",
    color: "#f6f5f1",
    whiteSpace: "nowrap",
  },
};
