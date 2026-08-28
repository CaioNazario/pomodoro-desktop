import { useState, type CSSProperties } from "react";
import { commands, type AtividadeDoPlano, type EstadoDoPlano } from "../../bindings";

interface Props {
  plano: EstadoDoPlano;
}

export function AtividadesNasPausas({ plano }: Props) {
  return (
    <div>
      <div style={estilos.cabecalho}>
        <span style={estilos.titulo}>Atividades nas pausas</span>
        <span style={estilos.opcional}>opcional</span>
      </div>
      <div style={estilos.lista}>
        {plano.modo === "global" ? (
          <CampoDeAtividade
            rotulo="Todas as pausas"
            atividade={plano.atividadeGlobal}
            onSalvar={(nome, url) => void commands.alterarAtividadeGlobal(nome, url)}
          />
        ) : (
          plano.planoIndividual.map((sessao, indice) => (
            <CampoDeAtividade
              key={indice}
              rotulo={`Pausa ${indice + 1}`}
              atividade={sessao.atividade}
              onSalvar={(nome, url) => void commands.alterarAtividadeIndividual(indice + 1, nome, url)}
            />
          ))
        )}
      </div>
    </div>
  );
}

interface CampoProps {
  rotulo: string;
  atividade: AtividadeDoPlano;
  onSalvar: (nome: string | null, url: string | null) => void;
}

function vazioParaNulo(valor: string): string | null {
  const aparado = valor.trim();
  return aparado.length > 0 ? aparado : null;
}

function CampoDeAtividade({ rotulo, atividade, onSalvar }: CampoProps) {
  const [nome, setNome] = useState(atividade.nome ?? "");
  const [url, setUrl] = useState(atividade.url ?? "");

  return (
    <div style={estilos.cartao}>
      <div style={estilos.linhaNome}>
        <span style={estilos.rotuloPausa}>{rotulo}</span>
        <input
          type="text"
          value={nome}
          placeholder="Atividade"
          style={estilos.inputNome}
          onChange={(evento) => setNome(evento.target.value)}
          onBlur={() => onSalvar(vazioParaNulo(nome), vazioParaNulo(url))}
        />
      </div>
      <input
        type="text"
        value={url}
        placeholder="https://…"
        style={estilos.inputUrl}
        onChange={(evento) => setUrl(evento.target.value)}
        onBlur={() => onSalvar(vazioParaNulo(nome), vazioParaNulo(url))}
      />
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
  lista: {
    maxHeight: 214,
    overflowY: "auto",
    padding: "0 6px 4px",
    display: "flex",
    flexDirection: "column",
    gap: 6,
  },
  cartao: {
    padding: 8,
    borderRadius: 10,
    border: "1px solid rgba(255,255,255,0.07)",
    display: "flex",
    flexDirection: "column",
    gap: 6,
  },
  linhaNome: { display: "flex", alignItems: "center", gap: 8 },
  rotuloPausa: {
    fontFamily: "'Geist Mono', monospace",
    fontSize: 10,
    letterSpacing: ".16em",
    textTransform: "uppercase",
    color: "#62626a",
    whiteSpace: "nowrap",
  },
  inputNome: {
    flex: 1,
    minWidth: 0,
    height: 24,
    padding: "0 8px",
    borderRadius: 7,
    border: "1px solid rgba(255,255,255,0.09)",
    background: "rgba(255,255,255,0.03)",
    color: "#ecebe6",
    font: "400 12px Geist, sans-serif",
    outline: "none",
  },
  inputUrl: {
    width: "100%",
    boxSizing: "border-box",
    height: 24,
    padding: "0 8px",
    borderRadius: 7,
    border: "1px solid rgba(255,255,255,0.09)",
    background: "rgba(255,255,255,0.03)",
    color: "#c9c8c3",
    font: "400 11.5px 'Geist Mono', monospace",
    outline: "none",
  },
};
