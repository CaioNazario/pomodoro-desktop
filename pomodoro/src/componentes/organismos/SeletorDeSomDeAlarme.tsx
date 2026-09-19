import type { CSSProperties } from "react";
import type { SomDeAlarme } from "../../bindings";
import { useSomDeAlarme } from "../../useSomDeAlarme";
import { BotaoIcone } from "../atomos/BotaoIcone";

const OPCOES: Array<{ valor: SomDeAlarme; rotulo: string }> = [
  { valor: "classico", rotulo: "Clássico" },
  { valor: "suave", rotulo: "Suave" },
  { valor: "urgente", rotulo: "Urgente" },
];

function estiloDaAba(ativa: boolean): CSSProperties {
  return {
    flex: 1,
    padding: "6px 0",
    border: "none",
    borderRadius: 7,
    background: ativa ? "rgba(255,255,255,0.1)" : "transparent",
    color: ativa ? "#eceae5" : "#87868c",
    font: "500 11.5px 'Geist Mono', monospace",
    letterSpacing: ".08em",
    textTransform: "uppercase",
    cursor: "pointer",
  };
}

/// Preferencia de som de alarme (PRD §7): toca nas transicoes de etapa de
/// pausas sem URL configurada, ja que essas nao tem o bloqueio em video
/// como sinal. Cada opcao tem um botao de previa (▶) pra ouvir antes de
/// escolher.
export function SeletorDeSomDeAlarme() {
  const { som, alterarSom, tocarPreview } = useSomDeAlarme();

  return (
    <div>
      <div style={estilos.cabecalho}>
        <span style={estilos.titulo}>Som de alarme</span>
        <span style={estilos.opcional}>sem atividade</span>
      </div>
      <div style={estilos.trilho}>
        {OPCOES.map((opcao) => (
          <div key={opcao.valor} style={estilos.item}>
            <button style={estiloDaAba(som === opcao.valor)} onClick={() => alterarSom(opcao.valor)}>
              {opcao.rotulo}
            </button>
            <BotaoIcone
              tamanho={20}
              titulo={`Ouvir ${opcao.rotulo}`}
              estilo={estilos.preview}
              onClick={() => tocarPreview(opcao.valor)}
            >
              ▶
            </BotaoIcone>
          </div>
        ))}
      </div>
    </div>
  );
}

const estilos: Record<string, CSSProperties> = {
  cabecalho: {
    display: "flex",
    alignItems: "baseline",
    justifyContent: "space-between",
    padding: "8px 12px 6px",
  },
  titulo: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 10.5,
    letterSpacing: ".22em",
    textTransform: "uppercase",
    color: "#56565d",
  },
  opcional: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 9.5,
    letterSpacing: ".1em",
    textTransform: "uppercase",
    color: "#45454b",
  },
  trilho: {
    display: "flex",
    gap: 4,
    margin: "0 12px 8px",
    padding: 3,
    borderRadius: 9,
    background: "rgba(255,255,255,0.045)",
  },
  item: {
    flex: 1,
    display: "flex",
    alignItems: "center",
    gap: 2,
  },
  preview: {
    color: "#87868c",
    flexShrink: 0,
  },
};
