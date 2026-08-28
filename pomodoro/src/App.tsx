import "./App.css";
import { useEffect, useState, type CSSProperties } from "react";
import { commands } from "./bindings";
import { AnelDeProgresso } from "./componentes/AnelDeProgresso";
import { Cabecalho } from "./componentes/Cabecalho";
import { ControlesDeTimer } from "./componentes/ControlesDeTimer";
import { MenuLateral } from "./componentes/MenuLateral";
import { OverlayDeTomadaDeFoco } from "./componentes/OverlayDeTomadaDeFoco";
import { PontosDoCiclo } from "./componentes/PontosDoCiclo";
import { Relogio } from "./componentes/Relogio";
import { Rodape } from "./componentes/Rodape";
import { useBloqueioAtivo } from "./useBloqueioAtivo";
import { useEstadoDoTimer } from "./useEstadoDoTimer";
import { useOverlayDeTomadaDeFoco } from "./useOverlayDeTomadaDeFoco";

// Fatia 1: 4 sessoes fixas, 25/5 global — mesmos valores hardcoded em
// `ciclo_inicial()` no src-tauri. Vira configuravel na fatia 2.
const DURACAO_FOCO_MIN = 25;
const DURACAO_PAUSA_MIN = 5;

function rotuloDoEstado(rodando: boolean, foco: boolean, decorridoMs: number): string {
  if (rodando) return foco ? "em foco" : "em pausa";
  return decorridoMs > 0 ? "pausado" : "pronto";
}

// Atalhos do app inertes durante o bloqueio (PRD §7.3): "Espaco"/"R"/"S" nao
// existem ainda (divida pre-existente do PRD §11), entao so M/Esc precisam
// ser desativados aqui.
function useMenuLateral(emBloqueio: boolean) {
  const [aberto, setAberto] = useState(false);

  useEffect(() => {
    function aoTeclar(evento: KeyboardEvent) {
      if (emBloqueio) return;
      if (evento.key === "Escape") {
        setAberto(false);
        return;
      }
      if (evento.key === "m" || evento.key === "M") setAberto((valor) => !valor);
    }
    window.addEventListener("keydown", aoTeclar);
    return () => window.removeEventListener("keydown", aoTeclar);
  }, [emBloqueio]);

  return { aberto, abrir: () => setAberto(true), fechar: () => setAberto(false) };
}

function App() {
  const { estado, restanteMs, alternarExecucao, reiniciarEtapa, pularEtapa, alterarIniciarAutomaticamente } =
    useEstadoDoTimer();
  const emBloqueio = useBloqueioAtivo();
  const menu = useMenuLateral(emBloqueio);
  const overlayDeTomadaDeFoco = useOverlayDeTomadaDeFoco();

  if (!estado) {
    return <div style={estilos.raiz} />;
  }

  const decorridoMs = estado.duracaoTotalMs - restanteMs;
  const progresso = estado.duracaoTotalMs > 0 ? Math.min(1, Math.max(0, decorridoMs / estado.duracaoTotalMs)) : 0;
  const rotuloEstado = rotuloDoEstado(estado.rodando, estado.etapa === "foco", decorridoMs);

  return (
    <div style={estilos.raiz}>
      <div style={estilos.fundo} />
      <Cabecalho estado={estado} onAbrirMenu={menu.abrir} onAlternarWidget={() => commands.alternarWidget()} />
      <main style={estilos.main}>
        <div style={estilos.coluna}>
          <div style={estilos.anelWrap}>
            <AnelDeProgresso etapa={estado.etapa} progresso={progresso} />
            <Relogio rotuloEstado={rotuloEstado} restanteMs={restanteMs} />
          </div>
          <ControlesDeTimer
            etapa={estado.etapa}
            rodando={estado.rodando}
            onAlternarExecucao={alternarExecucao}
            onReiniciar={reiniciarEtapa}
            onPular={pularEtapa}
          />
          <PontosDoCiclo etapa={estado.etapa} sessaoAtual={estado.numeroSessao} totalSessoes={estado.totalSessoes} />
        </div>
      </main>
      <Rodape estado={estado} duracaoFocoMin={DURACAO_FOCO_MIN} duracaoPausaMin={DURACAO_PAUSA_MIN} />
      <MenuLateral
        aberto={menu.aberto}
        onFechar={menu.fechar}
        iniciarAutomaticamente={estado.iniciarAutomaticamente}
        onAlterarIniciarAutomaticamente={alterarIniciarAutomaticamente}
      />
      <OverlayDeTomadaDeFoco ativo={overlayDeTomadaDeFoco} />
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  raiz: {
    position: "relative",
    width: "100%",
    height: "100vh",
    minHeight: 480,
    background: "#060607",
    color: "#f3f2ef",
    fontFamily: "Geist, 'Helvetica Neue', Helvetica, sans-serif",
    overflow: "hidden",
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    userSelect: "none",
  },
  fundo: {
    position: "absolute",
    inset: 0,
    background: "radial-gradient(90% 70% at 50% 42%, rgba(255,255,255,0.035), rgba(0,0,0,0) 70%)",
    pointerEvents: "none",
  },
  main: {
    position: "relative",
    zIndex: 2,
    display: "grid",
    placeItems: "center",
    padding: "8px 32px",
  },
  coluna: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "clamp(18px, 4.4vh, 46px)",
  },
  anelWrap: {
    position: "relative",
    width: "min(44vh, 38vw, 460px)",
    aspectRatio: "1",
  },
};

export default App;
