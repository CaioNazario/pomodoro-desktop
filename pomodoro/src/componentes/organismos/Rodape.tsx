import type { CSSProperties } from "react";
import type { EstadoDaTela } from "../../bindings";
import { useContadoresDoDia } from "../../useContadoresDoDia";
import { usePlanoDoCiclo } from "../../usePlanoDoCiclo";
import { emMinutos } from "../limitesDoPlano";

interface Props {
  estado: EstadoDaTela;
}

function formatarHorasEMinutos(ms: number): string {
  const minutosTotais = Math.floor(ms / 60_000);
  const horas = Math.floor(minutosTotais / 60);
  const minutos = minutosTotais % 60;
  return `${horas}h ${minutos}m`;
}

export function Rodape({ estado }: Props) {
  const contadores = useContadoresDoDia();
  const plano = usePlanoDoCiclo();
  const sessoesHoje = contadores?.sessoesConcluidasHoje ?? 0;
  const tempoDeFocoHoje = contadores?.tempoDeFocoHojeMs ?? 0;

  const sessaoAtual = plano?.planoIndividual[estado.numeroSessao - 1];
  const focoMs = plano?.modo === "global" ? plano.duracaoGlobalFocoMs : (sessaoAtual?.focoMs ?? 0);
  const pausaMs = plano?.modo === "global" ? plano.duracaoGlobalPausaMs : (sessaoAtual?.pausaMs ?? 0);

  return (
    <footer style={estilos.footer}>
      <div style={estilos.grupo}>
        <span>Hoje · {sessoesHoje} sessões</span>
        <span>Acumulado · {formatarHorasEMinutos(tempoDeFocoHoje)}</span>
      </div>
      <div style={estilos.grupo}>
        <span>
          {emMinutos(focoMs)} / {emMinutos(pausaMs)} min
        </span>
        <span style={estilos.destaque}>
          Ciclo {estado.numeroSessao}/{estado.totalSessoes} · {plano?.modo ?? "global"}
        </span>
      </div>
    </footer>
  );
}

const estilos: Record<string, CSSProperties> = {
  footer: {
    position: "relative",
    zIndex: 2,
    display: "flex",
    alignItems: "flex-end",
    justifyContent: "space-between",
    padding: "24px 28px 26px",
    fontFamily: "'Geist Mono', monospace",
    fontSize: 13.5,
    fontWeight: 500,
    letterSpacing: ".1em",
    textTransform: "uppercase",
    color: "#b6b5b0",
  },
  grupo: { display: "flex", gap: 26, whiteSpace: "nowrap" },
  destaque: { color: "#dedcd6", whiteSpace: "nowrap" },
};
