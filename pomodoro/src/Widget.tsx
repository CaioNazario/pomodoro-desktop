import "./Widget.css";
import { useCallback, useRef, useState, type CSSProperties, type MouseEvent as EventoDoMouseReact } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
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
        <span style={{ ...estilos.pontoPulsante, background: acento }} />
        <span style={estilos.relogioPilula}>{relogio}</span>
        <span style={estilos.barraPilula}>
          <span style={{ ...estilos.progressoPilula, width: `${progresso * 100}%`, background: acento }} />
        </span>
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
          <button title="Minimizar" style={estilos.botaoCirculo} onClick={() => setMinimizado(true)}>
            <IconeMinimizar />
          </button>
          <button title="Fechar" style={estilos.botaoCirculo} onClick={() => getCurrentWindow().close()}>
            ×
          </button>
        </div>
      </div>
      <div style={estilos.linhaRelogio}>
        <span style={estilos.relogioGrande}>{relogio}</span>
        <div style={estilos.botoesControle}>
          <button title="Reiniciar" style={estilos.botaoReset} onClick={reiniciarEtapa}>
            <IconeReset />
          </button>
          <button
            title="Iniciar / pausar"
            style={{ ...estilos.botaoPlay, background: acento }}
            onClick={alternarExecucao}
          >
            {estado.rodando ? <IconePausaPequena /> : <IconePlayPequeno />}
          </button>
        </div>
      </div>
      <div style={estilos.barra}>
        <div style={{ ...estilos.progresso, width: `${progresso * 100}%`, background: acento }} />
      </div>
    </div>
  );
}

function IconePontos() {
  return (
    <svg width="9" height="12" viewBox="0 0 9 12" aria-hidden="true" style={{ opacity: 0.55 }}>
      <circle cx="2" cy="2" r="1" fill="currentColor" />
      <circle cx="7" cy="2" r="1" fill="currentColor" />
      <circle cx="2" cy="6" r="1" fill="currentColor" />
      <circle cx="7" cy="6" r="1" fill="currentColor" />
      <circle cx="2" cy="10" r="1" fill="currentColor" />
      <circle cx="7" cy="10" r="1" fill="currentColor" />
    </svg>
  );
}

function IconeMinimizar() {
  return (
    <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
      <rect x="1" y="4.4" width="8" height="1.3" rx="0.65" fill="currentColor" />
    </svg>
  );
}

function IconeReset() {
  return (
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M3 8a5 5 0 1 0 1.9-3.9" stroke="currentColor" strokeWidth={1.4} strokeLinecap="round" />
      <path
        d="M2.7 2.8v2.8h2.8"
        stroke="currentColor"
        strokeWidth={1.4}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function IconePlayPequeno() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 5.2 18.4 12 8 18.8z" fill="currentColor" />
    </svg>
  );
}

function IconePausaPequena() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true">
      <rect x="7" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
      <rect x="13.6" y="5" width="3.4" height="14" rx="1.2" fill="currentColor" />
    </svg>
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
    width: 20,
    height: 20,
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "none",
    background: "rgba(255,255,255,0.08)",
    color: "#9a9a9f",
    fontSize: 11,
    lineHeight: 1,
    cursor: "pointer",
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
    width: 34,
    height: 34,
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "1px solid rgba(255,255,255,0.14)",
    background: "transparent",
    color: "#b9b8b2",
    cursor: "pointer",
  },
  botaoPlay: {
    width: 44,
    height: 44,
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "none",
    color: "#100b05",
    cursor: "pointer",
  },
  barra: {
    marginTop: 14,
    height: 3,
    borderRadius: 999,
    background: "rgba(255,255,255,0.1)",
    overflow: "hidden",
  },
  progresso: { height: "100%", transition: "width .2s linear" },
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
  pontoPulsante: {
    width: 7,
    height: 7,
    borderRadius: "50%",
    animation: "breathe 3.4s ease-in-out infinite",
    flexShrink: 0,
  },
  relogioPilula: {
    fontWeight: 400,
    fontSize: 17,
    letterSpacing: "-0.01em",
    fontVariantNumeric: "tabular-nums",
    color: "#f6f5f1",
    whiteSpace: "nowrap",
  },
  barraPilula: {
    width: 26,
    height: 3,
    borderRadius: 999,
    background: "rgba(255,255,255,0.14)",
    overflow: "hidden",
    flexShrink: 0,
  },
  progressoPilula: { display: "block", height: "100%" },
};
