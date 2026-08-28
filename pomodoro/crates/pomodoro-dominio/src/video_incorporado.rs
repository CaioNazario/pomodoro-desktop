use url::Url;

/// Se `url` aponta pra um vídeo do YouTube ou YouTube Music, a URL de embed
/// equivalente (PRD §7.2: pausa com vídeo mostra só o player, não o site
/// inteiro). `None` pra qualquer outra URL — inclusive links desses mesmos
/// hosts sem ID de vídeo identificável (playlist, home), que carregam como
/// estão.
pub(crate) fn url_incorporada(url: &Url) -> Option<Url> {
    let host_bruto = url.host_str()?;
    let host = host_bruto.strip_prefix("www.").unwrap_or(host_bruto);
    match host {
        "youtu.be" => embed_youtube(url.path_segments()?.next()?),
        "youtube.com" | "m.youtube.com" => embed_youtube(&id_do_youtube(url)?),
        "music.youtube.com" => embed_youtube(&id_da_query_v(url)?),
        _ => None,
    }
}

fn id_do_youtube(url: &Url) -> Option<String> {
    let mut segmentos = url.path_segments()?;
    match segmentos.next()? {
        "watch" => id_da_query_v(url),
        "shorts" => segmentos.next().map(str::to_string),
        _ => None,
    }
}

fn id_da_query_v(url: &Url) -> Option<String> {
    url.query_pairs()
        .find(|(chave, _)| chave == "v")
        .map(|(_, valor)| valor.into_owned())
        .filter(|valor| !valor.is_empty())
}

fn embed_youtube(id: &str) -> Option<Url> {
    if id.is_empty() {
        return None;
    }
    Url::parse(&format!("https://www.youtube.com/embed/{id}")).ok()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn embed(bruta: &str) -> Option<String> {
        url_incorporada(&Url::parse(bruta).expect("teste")).map(|u| u.to_string())
    }

    #[test]
    fn youtube_watch_vira_embed() {
        assert_eq!(
            embed("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtube_watch_sem_www_vira_embed() {
        assert_eq!(
            embed("https://youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtube_mobile_vira_embed() {
        assert_eq!(
            embed("https://m.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtu_be_vira_embed() {
        assert_eq!(
            embed("https://youtu.be/dQw4w9WgXcQ"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtube_shorts_vira_embed() {
        assert_eq!(
            embed("https://www.youtube.com/shorts/dQw4w9WgXcQ"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtube_music_watch_vira_embed() {
        assert_eq!(
            embed("https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=xyz"),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtube_sem_id_de_video_fica_none() {
        assert_eq!(embed("https://www.youtube.com/playlist?list=abc"), None);
        assert_eq!(embed("https://www.youtube.com/"), None);
    }

    #[test]
    fn vimeo_nao_e_reconhecido() {
        assert_eq!(embed("https://vimeo.com/76979871"), None);
    }

    #[test]
    fn site_de_leitura_fica_none() {
        assert_eq!(embed("https://akitaonrails.com/algum-post"), None);
    }
}
