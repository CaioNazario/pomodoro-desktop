import { Interruptor } from "../atomos/Interruptor";
import { ItemDeMenu } from "./ItemDeMenu";

interface Props {
  ativo: boolean;
  onAlterar: (valor: boolean) => void;
}

export function ToggleAutoStart({ ativo, onAlterar }: Props) {
  return (
    <ItemDeMenu
      rotulo="Iniciar automaticamente"
      onClick={() => onAlterar(!ativo)}
      filho={<Interruptor ativo={ativo} />}
    />
  );
}
