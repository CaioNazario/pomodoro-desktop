// Moldura do modo de bloqueio (PRD §7.3, §7.6). window.__moldura e injetado
// pelo Rust via initialization_script antes deste arquivo carregar — o
// prazo nao muda durante o bloqueio (reiniciar nao existe aqui), entao um
// unico valor injetado basta: o relogio deriva mm:ss localmente, mesmo
// padrao de useEstadoDoTimer.
//
// Unica chamada de invoke() desta janela: confirmar_urgencia, a unica saida
// real do bloqueio. window.__TAURI__ so existe aqui por causa de
// withGlobalTauri (tauri.conf.json) — este arquivo e servido cru pelo
// Vite, sem bundler, entao nao ha import de @tauri-apps/api possivel. A
// capability que libera o command e escopada so pra "bloqueio-*" (webview
// da propria moldura), nunca alcanca o webview isolado "atividade".
(function () {
  function formatar(ms) {
    var total = Math.max(0, Math.round(ms / 1000));
    var min = Math.floor(total / 60);
    var seg = total % 60;
    return String(min).padStart(2, "0") + ":" + String(seg).padStart(2, "0");
  }

  function ateProximoSegundo(restanteMs) {
    var resto = restanteMs % 1000;
    return resto === 0 ? 1000 : resto;
  }

  // O prazo nao muda durante o bloqueio, entao so o segundo exibido varia —
  // reagenda para o proximo segundo cheio em vez de rodar a 60Hz (rAF) numa
  // janela fullscreen por monitor, sem condicao de parada.
  function passo() {
    var dados = window.__moldura;
    var restante =
      dados.prazoEpocaMs !== null && dados.prazoEpocaMs !== undefined
        ? Math.max(0, dados.prazoEpocaMs - Date.now())
        : dados.duracaoTotalMs;
    document.getElementById("rotulo").textContent = "Pausa · " + dados.rotulo;
    document.getElementById("relogio").textContent = formatar(restante);
    var progresso =
      dados.duracaoTotalMs > 0
        ? Math.min(1, Math.max(0, (dados.duracaoTotalMs - restante) / dados.duracaoTotalMs))
        : 0;
    document.getElementById("progresso").style.width = progresso * 100 + "%";
    setTimeout(passo, ateProximoSegundo(restante));
  }

  // window.__moldura e injetado antes deste arquivo carregar, mas se por
  // algum motivo ainda nao existir no primeiro quadro, espera em passos
  // curtos em vez de girar sem parar.
  function aguardarMoldura() {
    if (window.__moldura) {
      passo();
      return;
    }
    setTimeout(aguardarMoldura, 50);
  }

  aguardarMoldura();

  var modal = document.getElementById("modal-urgencia");
  document.getElementById("botao-urgencia").addEventListener("click", function () {
    modal.hidden = false;
  });
  document.getElementById("continuar").addEventListener("click", function () {
    modal.hidden = true;
  });
  document.getElementById("confirmar").addEventListener("click", function () {
    if (window.__TAURI__ && window.__TAURI__.core) {
      window.__TAURI__.core.invoke("confirmar_urgencia");
    }
  });
})();
