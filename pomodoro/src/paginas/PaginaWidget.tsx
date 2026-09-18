import "./PaginaWidget.css";
import { useCallback, useRef, useState, type MouseEvent as EventoDoMouseReact } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CartaoDoWidget } from "../componentes/organismos/CartaoDoWidget";
import { PilulaDoWidget } from "../componentes/organismos/PilulaDoWidget";
import { acentoDaEtapa } from "../componentes/tema";
import { useEstadoDoTimer } from "../useEstadoDoTimer";

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

export function PaginaWidget() {
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
      <PilulaDoWidget
        acento={acento}
        relogio={relogio}
        progresso={progresso}
        onPressionar={aoPressionar}
        onClicar={aoClicarBotao}
      />
    );
  }

  return (
    <CartaoDoWidget
      acento={acento}
      modeLabel={modeLabel}
      cycleLabel={cycleLabel}
      relogio={relogio}
      progresso={progresso}
      rodando={estado.rodando}
      onMinimizar={() => setMinimizado(true)}
      onFechar={() => getCurrentWindow().close()}
      onReiniciar={reiniciarEtapa}
      onAlternarExecucao={alternarExecucao}
    />
  );
}
