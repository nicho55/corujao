//! Fonte de catálogo: a API pública do QuiverLauncher, baixada UMA vez por ação explícita do
//! usuário (botão "Atualizar pela rede") e guardada em `<raiz>/catalogo/quiver-apps.json`.
//! A leitura é sempre do arquivo local; o app nunca bate em rede ao abrir.
//!
//! Formato confirmado em 22/10/2026 (`GET /api/v1/apps?limit=N&cursor=`): `{items, nextCursor,
//! isDone}`; cada item traz `name` (o JOGO), `projectName`, `projectType`, `description`,
//! `consoles` (ids: `n64`, `ps2`, `x360`...), `tags`, `games[{slug,title}]`, `verified.version`,
//! `lastReleaseVersion`, `launcher.folderName`. NÃO traz repositório (só no detalhe por slug)
//! nem identificador de disco, região ou revisão (RF-05): isso continua "não informado".
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{Catalogo, Entrada, Fonte, TipoDeProjeto};
use crate::disco::Disco;
use crate::raiz::Raiz;

pub const ENDERECO: &str = "https://api.quiverlauncher.com/api/v1";
/// O Quiver cai para este quando o principal não responde; fazemos igual.
pub const ENDERECO_RESERVA: &str = "https://famous-wildebeest-660.convex.site/api/v1";
const ARQUIVO: &str = "quiver-apps.json";
const POR_PAGINA: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagina {
    items: Vec<serde_json::Value>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    is_done: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JogoDaApi {
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Verificada {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Lancador {
    #[serde(default)]
    folder_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    slug: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    project_name: String,
    #[serde(default)]
    project_type: String,
    #[serde(default)]
    games: Vec<JogoDaApi>,
    #[serde(default)]
    consoles: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    verified: Option<Verificada>,
    #[serde(default)]
    last_release_version: Option<String>,
    #[serde(default)]
    launcher: Option<Lancador>,
}

/// Os ids de console da API em nomes que uma pessoa lê. Id desconhecido sai em maiúsculas,
/// que é melhor que sumir.
pub fn nome_do_console(id: &str) -> String {
    match id.to_ascii_lowercase().as_str() {
        "nes" => "NES",
        "snes" => "Super Nintendo",
        "n64" => "Nintendo 64",
        "gc" | "gcn" | "gamecube" => "GameCube",
        "wii" => "Wii",
        "wiiu" => "Wii U",
        "switch" => "Switch",
        "gb" => "Game Boy",
        "gbc" => "Game Boy Color",
        "gba" => "Game Boy Advance",
        "ds" | "nds" => "Nintendo DS",
        "3ds" => "Nintendo 3DS",
        "ps1" | "psx" => "PlayStation",
        "ps2" => "PlayStation 2",
        "ps3" => "PlayStation 3",
        "psp" => "PSP",
        "vita" | "psvita" => "PS Vita",
        "xbox" => "Xbox",
        "x360" | "xbox360" => "Xbox 360",
        "dreamcast" | "dc" => "Dreamcast",
        "saturn" => "Saturn",
        "genesis" | "megadrive" | "md" => "Mega Drive",
        "pc" | "windows" | "dos" => "PC",
        "arcade" => "Arcade",
        "android" => "Android",
        outro => return outro.to_ascii_uppercase(),
    }
    .to_string()
}

/// O `projectType` da API no nosso tipo. Valor que não conhecemos vira inferência pelo nome.
fn tipo_da_api(project_type: &str, projeto: &str, tags: &[String]) -> (TipoDeProjeto, bool) {
    match project_type.to_ascii_lowercase().as_str() {
        "recomp" | "recompilation" => (TipoDeProjeto::Recompilacao, false),
        "decomp" | "decompilation" => (TipoDeProjeto::Descompilacao, false),
        "port" => (TipoDeProjeto::Porte, false),
        "engine" | "reimplementation" => (TipoDeProjeto::Reimplementacao, false),
        _ => (TipoDeProjeto::inferir(projeto, tags), true),
    }
}

pub fn entrada_da_api(app: &App) -> Entrada {
    let jogo = app
        .games
        .iter()
        .find_map(|g| g.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| app.name.clone());
    let projeto = if app.project_name.trim().is_empty() { app.name.clone() } else { app.project_name.clone() };
    let (tipo, tipo_inferido) = tipo_da_api(&app.project_type, &projeto, &app.tags);
    let descricao = Some(app.description.trim().to_string()).filter(|d| !d.is_empty());
    Entrada {
        slug: app.slug.clone(),
        jogo,
        jogo_inferido: false,
        projeto,
        tipo,
        tipo_inferido,
        repositorio: String::new(),
        provedor: String::new(),
        url: String::new(),
        versao_verificada: app.verified.as_ref().and_then(|v| v.version.clone()),
        versao_upstream: app.last_release_version.clone(),
        descricao,
        consoles: app.consoles.iter().map(|c| nome_do_console(c)).collect(),
        tags: app.tags.clone(),
        disco: Disco::default(),
        pasta: app.launcher.as_ref().and_then(|l| l.folder_name.clone()),
    }
}

fn agente() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .user_agent(concat!("Corujão/", env!("CARGO_PKG_VERSION")))
        .build();
    ureq::Agent::new_with_config(config)
}

fn pagina(agente: &ureq::Agent, base: &str, cursor: Option<&str>) -> Result<Pagina> {
    let mut url = format!("{base}/apps?limit={POR_PAGINA}");
    if let Some(c) = cursor {
        url.push_str("&cursor=");
        url.push_str(&urlencoding(c));
    }
    let texto = agente
        .get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .read_to_string()
        .with_context(|| format!("lendo a resposta de {url}"))?;
    serde_json::from_str(&texto).with_context(|| format!("resposta de {url} fora do formato esperado"))
}

/// Só o que um cursor precisa: o resto da URL é nosso.
fn urlencoding(s: &str) -> String {
    let mut saida = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => saida.push(b as char),
            _ => saida.push_str(&format!("%{b:02X}")),
        }
    }
    saida
}

fn baixar_tudo(agente: &ureq::Agent, base: &str) -> Result<Vec<serde_json::Value>> {
    let mut itens = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..200 {
        let p = pagina(agente, base, cursor.as_deref())?;
        itens.extend(p.items);
        if p.is_done || p.next_cursor.is_none() {
            return Ok(itens);
        }
        cursor = p.next_cursor;
    }
    Err(anyhow!("a paginação de {base} não terminou em 200 páginas; desisti"))
}

/// Baixa o catálogo inteiro e grava em `<raiz>/catalogo/quiver-apps.json` (escrita atômica).
/// Tenta o endereço principal e, se ele falhar, o reserva. Devolve o arquivo e quantas entradas.
pub fn baixar(raiz: &Raiz) -> Result<(PathBuf, usize)> {
    let agente = agente();
    let itens = match baixar_tudo(&agente, ENDERECO) {
        Ok(i) => i,
        Err(principal) => baixar_tudo(&agente, ENDERECO_RESERVA)
            .map_err(|reserva| anyhow!("quiverlauncher.com não respondeu ({principal:#}); o endereço reserva também não ({reserva:#})"))?,
    };
    let pasta = raiz.catalogo();
    std::fs::create_dir_all(&pasta).with_context(|| format!("não consegui criar {}", pasta.display()))?;
    let arquivo = pasta.join(ARQUIVO);
    let temporario = arquivo.with_extension("json.tmp");
    std::fs::write(&temporario, serde_json::to_vec(&itens)?)
        .with_context(|| format!("não consegui escrever {}", temporario.display()))?;
    std::fs::rename(&temporario, &arquivo)
        .with_context(|| format!("não consegui mover para {}", arquivo.display()))?;
    Ok((arquivo, itens.len()))
}

/// O catálogo que `baixar` deixou no disco.
pub struct CatalogoBaixadoDoQuiver {
    pub arquivo: PathBuf,
}

impl CatalogoBaixadoDoQuiver {
    pub fn na_raiz(raiz: &Raiz) -> CatalogoBaixadoDoQuiver {
        CatalogoBaixadoDoQuiver { arquivo: raiz.catalogo().join(ARQUIVO) }
    }

    pub fn existe(&self) -> bool {
        self.arquivo.is_file()
    }
}

impl Fonte for CatalogoBaixadoDoQuiver {
    fn nome(&self) -> String {
        "catálogo baixado de quiverlauncher.com".to_string()
    }

    /// Um item fora do formato vira problema listado, não derruba os outros.
    fn ler(&self) -> Result<Catalogo> {
        let mut catalogo = Catalogo { fonte: self.nome(), ..Default::default() };
        if !self.existe() {
            return Ok(catalogo);
        }
        let texto = std::fs::read_to_string(&self.arquivo)
            .with_context(|| format!("não consegui ler {}", self.arquivo.display()))?;
        let itens: Vec<serde_json::Value> =
            serde_json::from_str(&texto).with_context(|| format!("{} mal formado", self.arquivo.display()))?;
        catalogo.atualizado_em = std::fs::metadata(&self.arquivo).and_then(|m| m.modified()).ok();
        for (i, item) in itens.into_iter().enumerate() {
            match serde_json::from_value::<App>(item) {
                Ok(app) => catalogo.entradas.push(entrada_da_api(&app)),
                Err(e) => catalogo.problemas.push(format!("item {i} do catálogo baixado fora do formato: {e}")),
            }
        }
        catalogo.ordenar_por_jogo();
        Ok(catalogo)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const ITEM: &str = r#"{"slug":"2ship2harkinian","name":"The Legend of Zelda: Majora's Mask","description":"2 Ship 2 Harkinian brings The Legend of Zelda: Majora's Mask to PC.","projectName":"2 Ship 2 Harkinian","projectType":"port","consoles":["n64"],"tags":["decomp","n64"],"games":[{"id":"k9","slug":"the-legend-of-zelda-majoras-mask","title":"The Legend of Zelda: Majora's Mask"}],"verified":{"pinned":true,"version":"5.0.1"},"lastReleaseVersion":"5.0.1","launcher":{"filesToAdd":[],"folderName":"TheLegendOfZeldaMajorasMask-2Ship2Harkinian"}}"#;

    #[test]
    fn item_da_api_vira_entrada_rica() {
        let app: App = serde_json::from_str(ITEM).unwrap();
        let e = entrada_da_api(&app);
        assert_eq!(e.jogo, "The Legend of Zelda: Majora's Mask");
        assert!(!e.jogo_inferido);
        assert_eq!(e.projeto, "2 Ship 2 Harkinian");
        assert_eq!(e.tipo, TipoDeProjeto::Porte);
        assert!(!e.tipo_inferido);
        assert_eq!(e.consoles, vec!["Nintendo 64".to_string()]);
        assert_eq!(e.versao_verificada.as_deref(), Some("5.0.1"));
        assert_eq!(e.pasta.as_deref(), Some("TheLegendOfZeldaMajorasMask-2Ship2Harkinian"));
        assert!(e.repositorio.is_empty(), "a lista da API não traz o repositório");
        assert!(!e.disco.informado(), "a API não diz disco/região/revisão (RF-05)");
    }

    #[test]
    fn tipo_desconhecido_da_api_e_inferido() {
        let (t, inferido) = tipo_da_api("mod", "ZeldaRecomp", &[]);
        assert_eq!((t, inferido), (TipoDeProjeto::Recompilacao, true));
        assert_eq!(tipo_da_api("recomp", "x", &[]), (TipoDeProjeto::Recompilacao, false));
    }

    #[test]
    fn consoles_legiveis() {
        assert_eq!(nome_do_console("x360"), "Xbox 360");
        assert_eq!(nome_do_console("zx81"), "ZX81");
    }

    #[test]
    fn le_arquivo_baixado_tolerando_item_quebrado() {
        let pasta = std::env::temp_dir().join(format!("corujao-teste-quiver-api-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).unwrap();
        let arquivo = pasta.join(ARQUIVO);
        std::fs::write(&arquivo, format!("[{ITEM}, {{\"sem_slug\": true}}]")).unwrap();
        let c = CatalogoBaixadoDoQuiver { arquivo }.ler().unwrap();
        assert_eq!(c.entradas.len(), 1);
        assert_eq!(c.problemas.len(), 1);
        let _ = std::fs::remove_dir_all(&pasta);
    }

    #[test]
    fn cursor_vai_codificado() {
        assert_eq!(urlencoding("cards.WyJu=+/"), "cards.WyJu%3D%2B%2F");
    }
}
