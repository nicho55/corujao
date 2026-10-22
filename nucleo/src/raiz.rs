//! A raiz de dados: UMA pasta com tudo do launcher (jogos, capas, receitas da máquina,
//! registros). Copiar essa pasta leva o launcher inteiro para outra máquina.
//!
//! ```text
//! <raiz>/
//!   jogos/<id>/jogo.toml   metadados de um jogo (ver biblioteca.rs)
//!   jogos/<id>/capa.png    capa em 2:3
//!   maquina.toml           receitas de execução DESTA máquina (ver maquina.rs)
//!   configuracao.toml      preferências do usuário (ver configuracao.rs)
//!   catalogo/              cópias locais de catálogos baixados (ver catalogo/)
//!   registros/             saída de cada execução, um arquivo por vez
//! ```
//!
//! Caminho padrão: `~/jogos/<NOME_DA_PASTA>`; a variável `VARIAVEL_RAIZ` troca.
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::nome::{NOME_DA_PASTA, VARIAVEL_RAIZ};

#[derive(Clone, Debug)]
pub struct Raiz {
    pub caminho: PathBuf,
}

/// A pasta do usuário: `HOME` no Unix, `USERPROFILE` no Windows.
pub fn casa() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Expande `~` no começo de um caminho.
pub fn expandir(caminho: &str) -> PathBuf {
    match caminho.strip_prefix("~/") {
        Some(resto) => casa().join(resto),
        None if caminho == "~" => casa(),
        None => PathBuf::from(caminho),
    }
}

impl Raiz {
    pub fn descobrir() -> Raiz {
        let caminho = match std::env::var(VARIAVEL_RAIZ) {
            Ok(v) if !v.trim().is_empty() => expandir(v.trim()),
            _ => casa().join("jogos").join(NOME_DA_PASTA),
        };
        Raiz { caminho }
    }

    pub fn garantir(&self) -> Result<()> {
        for sub in ["jogos", "registros"] {
            let dir = self.caminho.join(sub);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("não consegui criar {}", dir.display()))?;
        }
        Ok(())
    }

    pub fn jogos(&self) -> PathBuf {
        self.caminho.join("jogos")
    }
    pub fn registros(&self) -> PathBuf {
        self.caminho.join("registros")
    }
    pub fn maquina(&self) -> PathBuf {
        self.caminho.join("maquina.toml")
    }
    pub fn configuracao(&self) -> PathBuf {
        self.caminho.join("configuracao.toml")
    }
    pub fn catalogo(&self) -> PathBuf {
        self.caminho.join("catalogo")
    }
    pub fn pasta_do_jogo(&self, id: &str) -> PathBuf {
        self.jogos().join(id)
    }
}

impl AsRef<Path> for Raiz {
    fn as_ref(&self) -> &Path {
        &self.caminho
    }
}
