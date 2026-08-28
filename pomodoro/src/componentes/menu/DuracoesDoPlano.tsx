import type { CSSProperties } from "react";
import { commands, type EstadoDoPlano, type SessaoDoPlano } from "../../bindings";
import { estilosDoMenu } from "./estilosDoMenu";
import { emMinutos, FOCO_MIN_MAX, FOCO_MIN_MIN, FOCO_PASSO_MIN, PAUSA_MIN_MAX, PAUSA_MIN_MIN, PAUSA_PASSO_MIN } from "./limitesDoPlano";

interface Props {
  plano: EstadoDoPlano;
}

export function DuracoesDoPlano({ plano }: Props) {
  if (plano.modo === "global") return <DuracoesGlobais plano={plano} />;
  return <DuracoesIndividuais planoIndividual={plano.planoIndividual} />;
}

function DuracoesGlobais({ plano }: { plano: EstadoDoPlano }) {
  return (
    <div>
      <LinhaDeMinutos
        rotulo="Todas as sessões (foco)"
        minutos={emMinutos(plano.duracaoGlobalFocoMs)}
        passo={FOCO_PASSO_MIN}
        min={FOCO_MIN_MIN}
        max={FOCO_MIN_MAX}
        onAlterar={(m) => void commands.alterarDuracaoGlobalFoco(m)}
      />
      <LinhaDeMinutos
        rotulo="Todas as pausas"
        minutos={emMinutos(plano.duracaoGlobalPausaMs)}
        passo={PAUSA_PASSO_MIN}
        min={PAUSA_MIN_MIN}
        max={PAUSA_MIN_MAX}
        onAlterar={(m) => void commands.alterarDuracaoGlobalPausa(m)}
      />
    </div>
  );
}

function DuracoesIndividuais({ planoIndividual }: { planoIndividual: SessaoDoPlano[] }) {
  return (
    <div style={estilos.listaIndividual}>
      <div style={estilos.cabecalhoColunas}>
        <span style={{ flex: 1 }}>Sessão</span>
        <span style={estilos.colunaLargura}>Foco</span>
        <span style={estilos.colunaLargura}>Pausa</span>
      </div>
      {planoIndividual.map((sessao, indice) => (
        <LinhaIndividual key={indice} numeroSessao={indice + 1} sessao={sessao} />
      ))}
    </div>
  );
}

function LinhaIndividual({ numeroSessao, sessao }: { numeroSessao: number; sessao: SessaoDoPlano }) {
  return (
    <div style={estilos.linhaIndividual}>
      <span style={estilos.numero}>{numeroSessao}</span>
      <ContadorCompacto
        minutos={emMinutos(sessao.focoMs)}
        passo={FOCO_PASSO_MIN}
        min={FOCO_MIN_MIN}
        max={FOCO_MIN_MAX}
        onAlterar={(m) => void commands.alterarDuracaoIndividualFoco(numeroSessao, m)}
      />
      <ContadorCompacto
        minutos={emMinutos(sessao.pausaMs)}
        passo={PAUSA_PASSO_MIN}
        min={PAUSA_MIN_MIN}
        max={PAUSA_MIN_MAX}
        onAlterar={(m) => void commands.alterarDuracaoIndividualPausa(numeroSessao, m)}
      />
    </div>
  );
}

interface LinhaDeMinutosProps {
  rotulo: string;
  minutos: number;
  passo: number;
  min: number;
  max: number;
  onAlterar: (minutos: number) => void;
}

function LinhaDeMinutos({ rotulo, minutos, passo, min, max, onAlterar }: LinhaDeMinutosProps) {
  return (
    <div style={estilosDoMenu.linha}>
      <span style={estilosDoMenu.rotulo}>{rotulo}</span>
      <div style={estilosDoMenu.passo}>
        <button
          style={estilosDoMenu.botaoPasso}
          disabled={minutos <= min}
          onClick={() => onAlterar(Math.max(min, minutos - passo))}
        >
          –
        </button>
        <span style={estilosDoMenu.valor}>{minutos} min</span>
        <button
          style={estilosDoMenu.botaoPasso}
          disabled={minutos >= max}
          onClick={() => onAlterar(Math.min(max, minutos + passo))}
        >
          +
        </button>
      </div>
    </div>
  );
}

function ContadorCompacto({ minutos, passo, min, max, onAlterar }: Omit<LinhaDeMinutosProps, "rotulo">) {
  return (
    <div style={estilos.colunaLargura}>
      <button
        style={estilos.botaoCompacto}
        disabled={minutos <= min}
        onClick={() => onAlterar(Math.max(min, minutos - passo))}
      >
        –
      </button>
      <span style={estilos.valorCompacto}>{minutos}</span>
      <button
        style={estilos.botaoCompacto}
        disabled={minutos >= max}
        onClick={() => onAlterar(Math.min(max, minutos + passo))}
      >
        +
      </button>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  listaIndividual: { maxHeight: 224, overflowY: "auto", margin: "0 4px", padding: "0 2px" },
  cabecalhoColunas: {
    display: "flex",
    alignItems: "center",
    padding: "0 8px 6px",
    fontFamily: "'Geist Mono', monospace",
    fontSize: 10,
    letterSpacing: ".16em",
    textTransform: "uppercase",
    color: "#4f4f56",
  },
  colunaLargura: {
    width: 68,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: 1,
    textAlign: "center",
  },
  linhaIndividual: {
    display: "flex",
    alignItems: "center",
    gap: 8,
    padding: "5px 8px",
    borderRadius: 9,
  },
  numero: {
    flex: 1,
    fontSize: 12.5,
    color: "#d9d8d3",
    whiteSpace: "nowrap",
  },
  botaoCompacto: {
    width: 19,
    height: 19,
    borderRadius: 6,
    border: "1px solid rgba(255,255,255,0.09)",
    background: "transparent",
    color: "#b6b5b0",
    fontSize: 11,
    cursor: "pointer",
    lineHeight: 1,
  },
  valorCompacto: {
    minWidth: 34,
    textAlign: "center",
    fontFamily: "'Geist Mono', monospace",
    fontSize: 11.5,
    color: "#ecebe6",
  },
};
