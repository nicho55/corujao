//! A biblioteca: um `jogo.toml` por pasta em `<raiz>/jogos/<id>/`.
//!
//! ```toml
//! nome = "The Legend of Zelda: The Wind Waker"
//! projeto = "BlueWake"            # quem recompilou / porta
//! console = "GameCube"
//! descricao = "..."
//! tags = ["recomp", "zelda"]
//!
//! [execucao]
//! executavel = "~/projetos/RECOMPS/build/linux/BlueWake-x86_64.AppImage"
//! argumentos = []
//! receita = "nvidia-optimus"      # receita de maquina.toml (RF-01); opcional
//! processo = "bluewake"           # nome do processo real (dentro da AppImage), para fechar por PID
//!
//! [dados]
//! saves = "~/.local/share/BlueWake"   # informativo: onde o jogo guarda o que é do jogador
//!
//! [disco]                          # qual cópia do jogo esta recomp cobre (RF-05, ver disco.rs)
//! id = "GZLE01"
//! revisao = "0"
//! ```
//! O que é DO JOGO fica aqui; o que é DA MÁQUINA (placa de vídeo, prefixos) fica em
//! `maquina.toml`. É a separação que o RF-01 pede.
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::disco::Disco;
use crate::raiz::{expandir, Raiz};

#[derive(Clone, Debug, Deserialize)]
pub struct Execucao {
    pub executavel: String,
    #[serde(default)]
    pub argumentos: Vec<String>,
    #[serde(default)]
    pub receita: Option<String>,
    #[serde(default)]
    pub processo: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Dados {
    #[serde(default)]
    pub saves: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JogoToml {
    pub nome: String,
    #[serde(default)]
    pub projeto: Option<String>,
    #[serde(default)]
    pub console: Option<String>,
    #[serde(default)]
    pub descricao: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub execucao: Execucao,
    #[serde(default)]
    pub dados: Dados,
    #[serde(default)]
    pub disco: Disco,
}

#[derive(Clone, Debug)]
pub struct Jogo {
    pub id: String,
    pub pasta: PathBuf,
    pub capa: Option<PathBuf>,
    pub toml: JogoToml,
}

impl Jogo {
    pub fn executavel(&self) -> PathBuf {
        expandir(&self.toml.execucao.executavel)
    }
}

/// Lê todos os `jogos/<id>/jogo.toml`. Uma entrada quebrada vira erro na lista, não derruba o resto.
pub fn listar(raiz: &Raiz) -> Vec<Result<Jogo>> {
    let mut saida = Vec::new();
    let Ok(entradas) = std::fs::read_dir(raiz.jogos()) else {
        return saida;
    };
    let mut pastas: Vec<PathBuf> = entradas.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    pastas.sort();
    for pasta in pastas {
        let arquivo = pasta.join("jogo.toml");
        if !arquivo.is_file() {
            continue;
        }
        saida.push(ler(&pasta, &arquivo));
    }
    saida
}

fn ler(pasta: &std::path::Path, arquivo: &std::path::Path) -> Result<Jogo> {
    let texto = std::fs::read_to_string(arquivo)
        .with_context(|| format!("não consegui ler {}", arquivo.display()))?;
    let toml: JogoToml =
        toml::from_str(&texto).with_context(|| format!("{} mal formado", arquivo.display()))?;
    let id = pasta.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let capa = ["capa.png", "capa.jpg"].iter().map(|n| pasta.join(n)).find(|p| p.is_file());
    Ok(Jogo { id, pasta: pasta.to_path_buf(), capa, toml })
}

pub fn achar(raiz: &Raiz, id: &str) -> Result<Jogo> {
    let pasta = raiz.pasta_do_jogo(id);
    let arquivo = pasta.join("jogo.toml");
    anyhow::ensure!(arquivo.is_file(), "jogo '{id}' não está na biblioteca ({})", arquivo.display());
    ler(&pasta, &arquivo)
}
