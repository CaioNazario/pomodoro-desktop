import { exit } from "@tauri-apps/plugin-process";
import { useCallback, type CSSProperties } from "react";
import { usePlanoDoCiclo } from "../usePlanoDoCiclo";
import { AtividadesNasPausas } from "./menu/AtividadesNasPausas";
import { DuracoesDoPlano } from "./menu/DuracoesDoPlano";
import { estilosDoMenu } from "./menu/estilosDoMenu";
import { SeletorDeModo } from "./menu/SeletorDeModo";
import { SeletorDeSessoes } from "./menu/SeletorDeSessoes";
import { ToggleAutoStart } from "./menu/ToggleAutoStart";

interface Props {
  aberto: boolean;
  onFechar: () => void;
  iniciarAutomaticamente: boolean;
  onAlterarIniciarAutomaticamente: (valor: boolean) => void;
}

async function sair(): Promise<void> {
  await exit(0);
}

export function MenuLateral({ aberto, onFechar, iniciarAutomaticamente, onAlterarIniciarAutomaticamente }: Props) {
  const plano = usePlanoDoCiclo();
  const aoClicarSair = useCallback(() => {
    void sair();
  }, []);

  return (
    <>
      <div
        style={{ ...estilos.scrim, opacity: aberto ? 1 : 0, pointerEvents: aberto ? "auto" : "none" }}
        onClick={onFechar}
        aria-hidden="true"
      />
      <nav
        aria-label="Menu"
        style={{
          ...estilos.painel,
          opacity: aberto ? 1 : 0,
          transform: aberto ? "translateY(0) scale(1)" : "translateY(-8px) scale(.97)",
          pointerEvents: aberto ? "auto" : "none",
        }}
      >
        {plano && (
          <>
            <SeletorDeSessoes totalSessoes={plano.totalSessoes} />
            <SeletorDeModo modo={plano.modo} />
            <DuracoesDoPlano plano={plano} />
            <div style={estilosDoMenu.divisor} />
            <ToggleAutoStart ativo={iniciarAutomaticamente} onAlterar={onAlterarIniciarAutomaticamente} />
            <div style={estilosDoMenu.divisor} />
            <AtividadesNasPausas plano={plano} />
            <div style={estilosDoMenu.divisor} />
          </>
        )}
        <button style={estilos.itemInerte} tabIndex={-1} aria-disabled="true">
          <span style={estilos.rotulo}>Estatísticas</span>
        </button>
        <button style={estilos.itemInerte} tabIndex={-1} aria-disabled="true">
          <span style={estilos.rotulo}>Configurações</span>
        </button>
        <div style={estilos.divisor} />
        <button style={estilos.itemSair} onClick={aoClicarSair}>
          <span style={estilos.rotulo}>Sair</span>
        </button>
      </nav>
    </>
  );
}

const estilos: Record<string, CSSProperties> = {
  scrim: {
    position: "absolute",
    inset: 0,
    zIndex: 8,
    background: "rgba(4,4,5,0.55)",
    backdropFilter: "blur(2px)",
    transition: "opacity .18s ease",
  },
  painel: {
    position: "absolute",
    top: 68,
    left: 24,
    zIndex: 9,
    boxSizing: "border-box",
    width: 300,
    maxHeight: "calc(100vh - 96px)",
    overflowY: "auto",
    padding: 8,
    borderRadius: 14,
    border: "1px solid rgba(255,255,255,0.1)",
    background: "#0d0d10",
    boxShadow: "0 24px 60px rgba(0,0,0,0.7)",
    transition: "opacity .16s ease, transform .18s cubic-bezier(.2,.8,.2,1)",
  },
  itemInerte: {
    width: "100%",
    display: "flex",
    alignItems: "center",
    gap: 11,
    padding: "9px 12px",
    border: "none",
    borderRadius: 9,
    background: "transparent",
    color: "#5f5f66",
    font: "400 13.5px Geist, sans-serif",
    textAlign: "left",
    cursor: "default",
  },
  itemSair: {
    width: "100%",
    display: "flex",
    alignItems: "center",
    gap: 11,
    padding: "9px 12px",
    border: "none",
    borderRadius: 9,
    background: "transparent",
    color: "#a9a8a3",
    font: "400 13.5px Geist, sans-serif",
    textAlign: "left",
    cursor: "pointer",
  },
  rotulo: { flex: 1 },
  divisor: { height: 1, margin: "7px 10px", background: "rgba(255,255,255,0.08)" },
};
