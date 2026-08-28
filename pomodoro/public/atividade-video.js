// So https://www.youtube.com/embed/... passa — mesmo formato que
// video_incorporado.rs gera; qualquer outra coisa em `?src=` e ignorada.
const src = new URLSearchParams(location.search).get("src");
if (src && /^https:\/\/www\.youtube\.com\/embed\//.test(src)) {
  document.getElementById("video").src = src;
}
