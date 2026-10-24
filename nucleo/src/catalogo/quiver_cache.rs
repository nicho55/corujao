//! Fonte de catálogo: o cache local do QuiverLauncher, lido do disco, sem rede.
//!
//! O Quiver guarda em `~/.local/share/QuiverLauncher/Cache/` dois arquivos que nos servem:
//! - `catalog-release-status.json`: lista com `slug`, `provider`, `repository`, versão
//!   verificada e versão upstream;
//! - `catalog-release-links.json`: por slug, o `folderName` no formato `Jogo-Projeto`
//!   (`TheLegendOfZeldaTheWindWakerHD-ZeldaWWHDRecomp`), que é o que separa o jogo do projeto.
//!
//! O que o cache NÃO tem, e esta fonte deixa vazio em vez de inventar: descrição, console,
//! tipo oficial do projeto e o disco que a recomp cobre (RF-05). O tipo é inferido pelo nome
//! e marcado como inferido.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use super::{separar_pascal_case, Catalogo, Entrada, Fonte, TipoDeProjeto};
use crate::disco::Disco;
use crate::raiz::casa;

const ARQUIVO_STATUS: &str = "catalog-release-status.json";
const ARQUIVO_LINKS: &str = "catalog-release-links.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Versao {
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    slug: String,
    provider: String,
    repository: String,
    #[serde(default)]
    verified: Option<Versao>,
    #[serde(default)]
    latest_upstream: Option<Versao>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Link {
    #[serde(default)]
    folder_name: Option<String>,
}

pub struct CacheDoQuiver {
    pub pasta: PathBuf,
}

impl CacheDoQuiver {
    /// A pasta de cache do Quiver nesta máquina: `$XDG_DATA_HOME/QuiverLauncher/Cache` ou
    /// `~/.local/share/QuiverLauncher/Cache` no Linux; no Windows, `%LOCALAPPDATA%\\QuiverLauncher\\Cache`
    /// (o código C# do Quiver usa `SpecialFolder.LocalApplicationData`; não testado em Windows).
    pub fn desta_maquina() -> CacheDoQuiver {
        let dados = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| if cfg!(windows) { std::env::var_os("LOCALAPPDATA").map(PathBuf::from) } else { None })
            .unwrap_or_else(|| casa().join(".local").join("share"));
        CacheDoQuiver { pasta: dados.join("QuiverLauncher").join("Cache") }
    }

    pub fn existe(&self) -> bool {
        self.pasta.join(ARQUIVO_STATUS).is_file()
    }

    fn ler_json<T: serde::de::DeserializeOwned>(&self, nome: &str) -> Result<T> {
        let arquivo = self.pasta.join(nome);
        let texto = std::fs::read_to_string(&arquivo)
            .with_context(|| format!("não consegui ler {}", arquivo.display()))?;
        serde_json::from_str(&texto).with_context(|| format!("{} mal formado", arquivo.display()))
    }
}

/// Endereço do repositório no provedor. Provedor desconhecido devolve o próprio `dono/repo`,
/// que a interface mostra como texto e não como link.
pub fn url_do_repositorio(provedor: &str, repositorio: &str) -> String {
    match provedor.to_ascii_lowercase().as_str() {
        "github" => format!("https://github.com/{repositorio}"),
        "gitlab" => format!("https://gitlab.com/{repositorio}"),
        _ => repositorio.to_string(),
    }
}

/// Converte um registro do cache numa entrada. `pasta` é o `folderName` (`Jogo-Projeto`);
/// sem ele, o jogo e o projeto saem do nome do repositório, marcados como inferidos.
fn entrada_do_cache(status: &Status, pasta: Option<&str>) -> Entrada {
    let (jogo, projeto) = match pasta.and_then(|p| p.split_once('-')) {
        Some((jogo, projeto)) => (separar_pascal_case(jogo), projeto.to_string()),
        None => {
            let projeto = status.repository.rsplit('/').next().unwrap_or(&status.repository).to_string();
            (pasta.map(separar_pascal_case).unwrap_or_else(|| projeto.clone()), projeto)
        }
    };
    let tipo = TipoDeProjeto::inferir(&projeto, &[]);
    Entrada {
        slug: status.slug.clone(),
        jogo,
        jogo_inferido: true,
        projeto,
        tipo,
        tipo_inferido: true,
        repositorio: status.repository.clone(),
        provedor: status.provider.clone(),
        url: url_do_repositorio(&status.provider, &status.repository),
        versao_verificada: status.verified.as_ref().map(|v| v.version.clone()),
        versao_upstream: status.latest_upstream.as_ref().map(|v| v.version.clone()),
        descricao: None,
        consoles: Vec::new(),
        tags: Vec::new(),
        disco: Disco::default(),
        pasta: pasta.map(String::from),
    }
}

fn modificado_em(arquivo: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(arquivo).and_then(|m| m.modified()).ok()
}

impl Fonte for CacheDoQuiver {
    fn nome(&self) -> String {
        "cache local do QuiverLauncher".to_string()
    }

    /// Sem o Quiver instalado o catálogo sai vazio com o motivo em `problemas`; não é erro.
    fn ler(&self) -> Result<Catalogo> {
        let mut catalogo = Catalogo { fonte: self.nome(), ..Default::default() };
        if !self.existe() {
            catalogo.problemas.push(format!(
                "cache do QuiverLauncher não encontrado em {} (abra o Quiver uma vez, ou atualize o catálogo pela rede quando isso existir)",
                self.pasta.display()
            ));
            return Ok(catalogo);
        }
        let status: Vec<Status> = self.ler_json(ARQUIVO_STATUS)?;
        // Os links são complemento: sem eles o catálogo ainda sai, só com nomes piores.
        let links: BTreeMap<String, Link> = match self.ler_json(ARQUIVO_LINKS) {
            Ok(l) => l,
            Err(e) => {
                catalogo.problemas.push(format!("{e:#}"));
                BTreeMap::new()
            }
        };
        catalogo.atualizado_em = modificado_em(&self.pasta.join(ARQUIVO_STATUS));
        catalogo.entradas = status
            .iter()
            .map(|s| entrada_do_cache(s, links.get(&s.slug).and_then(|l| l.folder_name.as_deref())))
            .collect();
        catalogo.ordenar_por_jogo();
        Ok(catalogo)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const STATUS: &str = r#"[
      {"id":"k1","slug":"dr-mario-drmarionesrecomp","provider":"github","repository":"mstan/DrMarioNesRecomp",
       "verified":{"version":"v1.3.1","rolling":false},"latestUpstream":{"version":"v1.3.1","rolling":false}},
      {"id":"k2","slug":"super-smash-bros-melee-melee-unlocked","provider":"github","repository":"Hero88go/melee-unlocked",
       "verified":{"version":"v0.9.1","rolling":false},"latestUpstream":{"version":"v0.9.16","rolling":false}},
      {"id":"k3","slug":"sem-links","provider":"gitlab","repository":"grupo/jogo-port"}
    ]"#;
    const LINKS: &str = r#"{
      "dr-mario-drmarionesrecomp": {"folderName": "DrMario-DrMarioNesRecomp", "releaseAssetFilter": null},
      "super-smash-bros-melee-melee-unlocked": {"folderName": "SuperSmashBrosMelee-MeleeUnlocked", "releaseAssetFilter": null}
    }"#;

    fn pasta_de_teste(nome: &str, com_links: bool) -> CacheDoQuiver {
        let pasta = std::env::temp_dir().join(format!("corujao-teste-quiver-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).unwrap();
        std::fs::write(pasta.join(ARQUIVO_STATUS), STATUS).unwrap();
        if com_links {
            std::fs::write(pasta.join(ARQUIVO_LINKS), LINKS).unwrap();
        }
        CacheDoQuiver { pasta }
    }

    #[test]
    fn le_o_cache_e_separa_jogo_de_projeto() {
        let fonte = pasta_de_teste("completo", true);
        let c = fonte.ler().unwrap();
        assert!(c.problemas.is_empty(), "{:?}", c.problemas);
        assert_eq!(c.entradas.len(), 3);
        let dr = c.entradas.iter().find(|e| e.slug == "dr-mario-drmarionesrecomp").unwrap();
        assert_eq!(dr.jogo, "Dr Mario");
        assert_eq!(dr.projeto, "DrMarioNesRecomp");
        assert_eq!(dr.tipo, TipoDeProjeto::Recompilacao);
        assert!(dr.tipo_inferido && dr.jogo_inferido);
        assert_eq!(dr.url, "https://github.com/mstan/DrMarioNesRecomp");
        assert!(!dr.tem_atualizacao());
        let melee = c.entradas.iter().find(|e| e.projeto == "MeleeUnlocked").unwrap();
        assert!(melee.tem_atualizacao());
        assert!(c.atualizado_em.is_some());
        let _ = std::fs::remove_dir_all(&fonte.pasta);
    }

    #[test]
    fn sem_links_cai_no_nome_do_repositorio() {
        let fonte = pasta_de_teste("sem-links", false);
        let c = fonte.ler().unwrap();
        assert_eq!(c.problemas.len(), 1, "avisa que os links faltaram");
        let e = c.entradas.iter().find(|e| e.slug == "sem-links").unwrap();
        assert_eq!(e.projeto, "jogo-port");
        assert_eq!(e.jogo, "jogo-port");
        assert_eq!(e.tipo, TipoDeProjeto::Porte);
        assert_eq!(e.url, "https://gitlab.com/grupo/jogo-port");
        assert_eq!(e.versao_verificada, None);
        let _ = std::fs::remove_dir_all(&fonte.pasta);
    }

    #[test]
    fn sem_quiver_e_vazio_com_motivo() {
        let fonte = CacheDoQuiver { pasta: PathBuf::from("/nao/existe/QuiverLauncher/Cache") };
        let c = fonte.ler().unwrap();
        assert!(c.entradas.is_empty());
        assert_eq!(c.problemas.len(), 1);
    }
}
