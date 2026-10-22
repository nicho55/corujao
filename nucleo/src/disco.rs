//! Qual cópia do jogo uma recomp cobre (RF-05, `documentacao/03-requisitos.md`).
//!
//! Uma recompilação é feita de uma imagem de disco específica: outra região ou outra revisão
//! muda os endereços e a recompilação não aceita. Por isso o identificador do disco é dado de
//! primeira classe, tanto na biblioteca (`jogo.toml`, seção `[disco]`) quanto no catálogo.
//!
//! ```toml
//! [disco]
//! id = "GZLE01"   # GameCube/Wii: sistema + jogo + região + editora
//! revisao = "0"   # revisão da prensagem
//! ```
//!
//! O código é decodificado em texto que uma pessoa entende ("GameCube, NTSC-U, revisão 0") e,
//! quando não existe, a resposta é "não informado": silêncio não é dado.
use serde::{Deserialize, Serialize};

/// Identificação do disco ou cartucho. Tudo opcional: a maior parte dos catálogos não traz.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Disco {
    /// Código do disco/cartucho: `GZLE01` (GameCube/Wii, 6 letras) ou `NZSE` (N64, 4 letras).
    #[serde(default)]
    pub id: Option<String>,
    /// Revisão da prensagem (`0`, `1`, `1.1`...).
    #[serde(default)]
    pub revisao: Option<String>,
}

/// O que dá para ler de dentro de um código de disco.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decodificado {
    pub console: Option<&'static str>,
    pub regiao: Option<&'static str>,
    pub editora: Option<&'static str>,
}

pub const NAO_INFORMADO: &str = "não informado";

impl Disco {
    pub fn informado(&self) -> bool {
        self.id.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
    }

    /// Lê sistema, região e editora do código. Formatos conhecidos: GameCube/Wii (6
    /// caracteres) e Nintendo 64 (4 caracteres). Outro tamanho decodifica só o que der (nada).
    pub fn decodificar(&self) -> Decodificado {
        let Some(id) = self.id.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
            return Decodificado::default();
        };
        let letras: Vec<char> = id.chars().map(|c| c.to_ascii_uppercase()).collect();
        match letras.len() {
            6 => Decodificado {
                console: console_gamecube_wii(letras[0]),
                regiao: regiao(letras[3]),
                editora: editora(&id[4..6]),
            },
            4 if letras[0] == 'N' || letras[0] == 'C' => Decodificado {
                console: Some("Nintendo 64"),
                regiao: regiao(letras[3]),
                editora: None,
            },
            _ => Decodificado::default(),
        }
    }

    /// Texto para a interface. `console_do_jogo` entra quando o código não diz o sistema.
    ///
    /// Exemplos: `GameCube, NTSC-U (América), revisão 0 · GZLE01`; `não informado`.
    pub fn descrever(&self, console_do_jogo: Option<&str>) -> String {
        if !self.informado() {
            return NAO_INFORMADO.to_string();
        }
        let id = self.id.as_deref().unwrap_or_default().trim();
        let d = self.decodificar();
        let mut partes: Vec<String> = Vec::new();
        if let Some(c) = d.console.or(console_do_jogo) {
            partes.push(c.to_string());
        }
        if let Some(r) = d.regiao {
            partes.push(r.to_string());
        }
        match self.revisao.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
            Some(r) => partes.push(format!("revisão {r}")),
            None => partes.push("revisão não informada".to_string()),
        }
        if let Some(e) = d.editora {
            partes.push(e.to_string());
        }
        format!("{} · {id}", partes.join(", "))
    }
}

fn console_gamecube_wii(letra: char) -> Option<&'static str> {
    match letra {
        'G' | 'D' => Some("GameCube"),
        'R' | 'S' => Some("Wii"),
        _ => None,
    }
}

/// Letra de região da Nintendo (GameCube, Wii e N64 usam a mesma tabela).
fn regiao(letra: char) -> Option<&'static str> {
    match letra {
        'E' => Some("NTSC-U (América)"),
        'J' => Some("NTSC-J (Japão)"),
        'K' => Some("NTSC-K (Coreia)"),
        'P' => Some("PAL (Europa)"),
        'D' => Some("PAL (Alemanha)"),
        'F' => Some("PAL (França)"),
        'I' => Some("PAL (Itália)"),
        'S' => Some("PAL (Espanha)"),
        'U' => Some("PAL (Austrália)"),
        'X' | 'Y' | 'Z' => Some("PAL (vários idiomas)"),
        _ => None,
    }
}

/// Os códigos de editora mais comuns no GameCube/Wii. Desconhecido vira `None`, não chute.
fn editora(codigo: &str) -> Option<&'static str> {
    match codigo.to_ascii_uppercase().as_str() {
        "01" => Some("Nintendo"),
        "08" => Some("Capcom"),
        "41" => Some("Ubisoft"),
        "4F" => Some("Eidos"),
        "51" => Some("Acclaim"),
        "52" => Some("Activision"),
        "5D" => Some("Midway"),
        "5G" => Some("Hudson"),
        "64" => Some("LucasArts"),
        "69" => Some("Electronic Arts"),
        "6S" => Some("TDK Mediactive"),
        "70" => Some("Atari"),
        "78" => Some("THQ"),
        "7D" => Some("Vivendi"),
        "8P" => Some("Sega"),
        "A4" => Some("Konami"),
        "AF" => Some("Namco"),
        "B2" => Some("Bandai"),
        "E7" => Some("Square Enix"),
        _ => None,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn disco(id: &str, rev: Option<&str>) -> Disco {
        Disco { id: Some(id.into()), revisao: rev.map(String::from) }
    }

    #[test]
    fn wind_waker_americano() {
        let d = disco("GZLE01", Some("0"));
        assert_eq!(d.decodificar(), Decodificado { console: Some("GameCube"), regiao: Some("NTSC-U (América)"), editora: Some("Nintendo") });
        assert_eq!(d.descrever(None), "GameCube, NTSC-U (América), revisão 0, Nintendo · GZLE01");
    }

    #[test]
    fn sem_revisao_diz_que_nao_tem() {
        assert_eq!(disco("GZLP01", None).descrever(None), "GameCube, PAL (Europa), revisão não informada, Nintendo · GZLP01");
    }

    #[test]
    fn nintendo_64() {
        let d = disco("NZSE", Some("1.1"));
        assert_eq!(d.decodificar().console, Some("Nintendo 64"));
        assert_eq!(d.descrever(None), "Nintendo 64, NTSC-U (América), revisão 1.1 · NZSE");
    }

    #[test]
    fn codigo_desconhecido_usa_o_console_do_jogo() {
        assert_eq!(disco("SLUS-20228", Some("1")).descrever(Some("PlayStation 2")), "PlayStation 2, revisão 1 · SLUS-20228");
    }

    #[test]
    fn vazio_e_nao_informado() {
        assert_eq!(Disco::default().descrever(Some("GameCube")), NAO_INFORMADO);
        assert_eq!(disco("  ", Some("0")).descrever(None), NAO_INFORMADO);
        assert!(!disco("", None).informado());
    }
}
