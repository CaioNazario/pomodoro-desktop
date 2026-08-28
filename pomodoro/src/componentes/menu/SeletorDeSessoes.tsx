import { commands } from "../../bindings";
import { estilosDoMenu } from "./estilosDoMenu";
import { SESSOES_MAX, SESSOES_MIN } from "./limitesDoPlano";

interface Props {
  totalSessoes: number;
}

function redimensionar(nova: number): void {
  void commands.redimensionar(nova);
}

export function SeletorDeSessoes({ totalSessoes }: Props) {
  return (
    <div style={estilosDoMenu.linha}>
      <span style={estilosDoMenu.rotulo}>Sessões no ciclo</span>
      <div style={estilosDoMenu.passo}>
        <button
          style={estilosDoMenu.botaoPasso}
          disabled={totalSessoes <= SESSOES_MIN}
          onClick={() => redimensionar(totalSessoes - 1)}
        >
          –
        </button>
        <span style={{ ...estilosDoMenu.valor, minWidth: 42 }}>{totalSessoes}</span>
        <button
          style={estilosDoMenu.botaoPasso}
          disabled={totalSessoes >= SESSOES_MAX}
          onClick={() => redimensionar(totalSessoes + 1)}
        >
          +
        </button>
      </div>
    </div>
  );
}
