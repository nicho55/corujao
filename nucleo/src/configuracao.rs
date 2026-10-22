//! Preferências do usuário em `<raiz>/configuracao.toml` (D-10: TOML, escrita atômica, viaja
//! com a pasta). Hoje só o tema (RF-06); o que for preferência entra aqui, não em código.
//!
//! ```toml
//! tema = "sistema"   # sistema | claro | escuro
//! ```
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::raiz::Raiz;

/// Como a interface escolhe a aparência (RF-06). `Sistema` segue o sistema operacional.
/// O padrão é `Escuro`: é a cara de Steam com que o launcher nasceu; quem quiser troca.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tema {
    Sistema,
    Claro,
    #[default]
    Escuro,
}

impl Tema {
    pub const TODOS: [Tema; 3] = [Tema::Sistema, Tema::Claro, Tema::Escuro];

    /// Identificador estável, o mesmo que vai no TOML e atravessa a ponte para o Flutter.
    pub fn id(self) -> &'static str {
        match self {
            Tema::Sistema => "sistema",
            Tema::Claro => "claro",
            Tema::Escuro => "escuro",
        }
    }

    pub fn de_id(id: &str) -> Option<Tema> {
        Tema::TODOS.into_iter().find(|t| t.id() == id.trim().to_ascii_lowercase())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Configuracao {
    #[serde(default)]
    pub tema: Tema,
}

impl Configuracao {
    /// Sem arquivo é a configuração padrão; arquivo mal formado é erro (não se perde a
    /// configuração de ninguém silenciosamente).
    pub fn ler(raiz: &Raiz) -> Result<Configuracao> {
        let arquivo = raiz.configuracao();
        if !arquivo.is_file() {
            return Ok(Configuracao::default());
        }
        let texto = std::fs::read_to_string(&arquivo)
            .with_context(|| format!("não consegui ler {}", arquivo.display()))?;
        toml::from_str(&texto).with_context(|| format!("{} mal formado", arquivo.display()))
    }

    /// Escrita atômica: grava num arquivo ao lado e renomeia por cima, para o app morrer no
    /// meio sem deixar o TOML pela metade (o motivo de não usarmos um JSON único, D-10).
    pub fn gravar(&self, raiz: &Raiz) -> Result<()> {
        raiz.garantir()?;
        let arquivo = raiz.configuracao();
        let temporario = arquivo.with_extension("toml.tmp");
        let texto = format!(
            "# Preferências do POG. Editável à mão; o app reescreve ao mudar algo na interface.\n{}",
            toml::to_string(self).context("não consegui serializar a configuração")?
        );
        std::fs::write(&temporario, texto)
            .with_context(|| format!("não consegui escrever {}", temporario.display()))?;
        std::fs::rename(&temporario, &arquivo)
            .with_context(|| format!("não consegui mover para {}", arquivo.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::PathBuf;

    fn raiz_temporaria(nome: &str) -> Raiz {
        let caminho = std::env::temp_dir().join(format!("pog-teste-config-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&caminho);
        Raiz { caminho }
    }

    #[test]
    fn ids_estaveis() {
        for t in Tema::TODOS {
            assert_eq!(Tema::de_id(t.id()), Some(t));
        }
        assert_eq!(Tema::de_id(" Escuro "), Some(Tema::Escuro));
        assert_eq!(Tema::de_id("neon"), None);
    }

    #[test]
    fn sem_arquivo_e_padrao() {
        let raiz = raiz_temporaria("padrao");
        assert_eq!(Configuracao::ler(&raiz).unwrap(), Configuracao::default());
    }

    #[test]
    fn grava_e_le_de_volta() {
        let raiz = raiz_temporaria("ida-e-volta");
        let c = Configuracao { tema: Tema::Claro };
        c.gravar(&raiz).unwrap();
        assert_eq!(Configuracao::ler(&raiz).unwrap(), c);
        assert!(!raiz.configuracao().with_extension("toml.tmp").exists(), "o temporário some");
        let texto = std::fs::read_to_string(raiz.configuracao()).unwrap();
        assert!(texto.contains("tema = \"claro\""), "{texto}");
        let _ = std::fs::remove_dir_all(PathBuf::from(&raiz.caminho));
    }

    #[test]
    fn mal_formado_e_erro() {
        let raiz = raiz_temporaria("mal-formado");
        raiz.garantir().unwrap();
        std::fs::write(raiz.configuracao(), "tema = 42").unwrap();
        assert!(Configuracao::ler(&raiz).is_err());
        let _ = std::fs::remove_dir_all(&raiz.caminho);
    }
}
