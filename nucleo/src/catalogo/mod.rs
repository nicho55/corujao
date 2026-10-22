//! O catálogo: o que existe para explorar, com o que cada repositório É. A aba Explorar
//! responde três perguntas por entrada: que jogo, de que console, e o que esse repositório faz
//! (recompilação? descompilação? porte?). E, pelo RF-05, qual cópia do jogo ele cobre.
//!
//! Fontes (`trait Fonte`): hoje o cache local do QuiverLauncher (`quiver_cache.rs`), sem rede.
//! Uma fonte baixada da API do Quiver e o nosso catálogo em git (D-10, §5.3) entram como outras
//! implementações do mesmo trait; `Catalogo::enriquecer` junta o que cada uma sabe, por slug.
pub mod quiver_api;
pub mod quiver_cache;

use std::collections::BTreeMap;
use std::time::SystemTime;

use anyhow::Result;

use crate::disco::Disco;

/// O que um repositório faz com o jogo. Inferido pelo nome quando a fonte não diz
/// (`Entrada::tipo_inferido`), e a interface avisa que foi inferido.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDeProjeto {
    Recompilacao,
    Descompilacao,
    Porte,
    Reimplementacao,
    Desconhecido,
}

impl TipoDeProjeto {
    /// Chuta pelo nome do projeto e pelas tags. É heurística: `recomp` no nome é quase certeza,
    /// `port` e `native` são fortes, `Open*` costuma ser motor reimplementado.
    pub fn inferir(projeto: &str, tags: &[String]) -> TipoDeProjeto {
        let nome = projeto.to_ascii_lowercase();
        let tem = |p: &str| nome.contains(p) || tags.iter().any(|t| t.to_ascii_lowercase().contains(p));
        if tem("recomp") {
            TipoDeProjeto::Recompilacao
        } else if tem("decomp") {
            TipoDeProjeto::Descompilacao
        } else if tem("port") || tem("native") || tem("droid") || nome.ends_with("pc") || nome.ends_with("hd") {
            TipoDeProjeto::Porte
        } else if nome.starts_with("open") || tem("engine") || tem("reimpl") || tem("rs") && nome.ends_with("rs") {
            TipoDeProjeto::Reimplementacao
        } else {
            TipoDeProjeto::Desconhecido
        }
    }

    /// Identificador estável, para filtro e para a interface escolher ícone.
    pub fn id(self) -> &'static str {
        match self {
            TipoDeProjeto::Recompilacao => "recompilacao",
            TipoDeProjeto::Descompilacao => "descompilacao",
            TipoDeProjeto::Porte => "porte",
            TipoDeProjeto::Reimplementacao => "reimplementacao",
            TipoDeProjeto::Desconhecido => "desconhecido",
        }
    }

    pub fn rotulo(self) -> &'static str {
        match self {
            TipoDeProjeto::Recompilacao => "Recompilação",
            TipoDeProjeto::Descompilacao => "Descompilação",
            TipoDeProjeto::Porte => "Porte nativo",
            TipoDeProjeto::Reimplementacao => "Motor reimplementado",
            TipoDeProjeto::Desconhecido => "Tipo desconhecido",
        }
    }

    /// A explicação que a aba Explorar mostra quando a fonte não traz descrição.
    pub fn explicacao(self) -> &'static str {
        match self {
            TipoDeProjeto::Recompilacao => {
                "Recompilação estática: o código do jogo é traduzido do binário original para C e compilado \
                 nativo. Precisa da SUA cópia do jogo (ROM ou imagem de disco); o repositório traz só a \
                 ferramenta, nunca o jogo."
            }
            TipoDeProjeto::Descompilacao => {
                "Descompilação: o código-fonte foi reconstruído à mão a partir do binário e compila de novo. \
                 Precisa dos dados da sua cópia do jogo (texturas, sons, mapas)."
            }
            TipoDeProjeto::Porte => {
                "Porte nativo: o jogo adaptado para rodar direto no PC ou no celular, em geral sobre uma \
                 descompilação. Precisa dos dados da sua cópia do jogo."
            }
            TipoDeProjeto::Reimplementacao => {
                "Motor reimplementado: um motor novo, escrito do zero, que lê os dados do jogo original. \
                 Precisa dos dados da sua cópia do jogo."
            }
            TipoDeProjeto::Desconhecido => {
                "Não deu para saber pelo nome o que este repositório faz. Abra o repositório para ver."
            }
        }
    }
}

/// Uma entrada do catálogo, já com o que dá para explicar.
#[derive(Clone, Debug, PartialEq)]
pub struct Entrada {
    pub slug: String,
    /// Nome do jogo em texto; `jogo_inferido` quando saiu do nome da pasta, não de uma fonte.
    pub jogo: String,
    pub jogo_inferido: bool,
    /// Nome do projeto que recompila/porta (ex.: `BlueWake`, `ZeldaWWHDRecomp`).
    pub projeto: String,
    pub tipo: TipoDeProjeto,
    pub tipo_inferido: bool,
    /// `dono/repo` no provedor.
    pub repositorio: String,
    /// `github` ou `gitlab`.
    pub provedor: String,
    pub url: String,
    pub versao_verificada: Option<String>,
    pub versao_upstream: Option<String>,
    pub descricao: Option<String>,
    pub consoles: Vec<String>,
    pub tags: Vec<String>,
    pub disco: Disco,
    /// Pasta que o Quiver usa para instalar (`Jogo-Projeto`).
    pub pasta: Option<String>,
}

impl Entrada {
    /// Há versão upstream mais nova que a verificada.
    pub fn tem_atualizacao(&self) -> bool {
        match (&self.versao_verificada, &self.versao_upstream) {
            (Some(v), Some(u)) => v != u,
            _ => false,
        }
    }

    /// Completa o que falta com o que `outra` sabe. A fonte mais rica vence onde tem dado;
    /// o que já era conhecido (não inferido) não é sobrescrito por inferência.
    pub fn enriquecer(&mut self, outra: &Entrada) {
        if self.jogo_inferido && !outra.jogo_inferido {
            self.jogo = outra.jogo.clone();
            self.jogo_inferido = false;
        }
        if self.tipo_inferido && !outra.tipo_inferido {
            self.tipo = outra.tipo;
            self.tipo_inferido = false;
        }
        if self.descricao.is_none() {
            self.descricao = outra.descricao.clone();
        }
        if self.consoles.is_empty() {
            self.consoles = outra.consoles.clone();
        }
        if self.tags.is_empty() {
            self.tags = outra.tags.clone();
        }
        if !self.disco.informado() && outra.disco.informado() {
            self.disco = outra.disco.clone();
        }
        if self.versao_verificada.is_none() {
            self.versao_verificada = outra.versao_verificada.clone();
        }
        if self.versao_upstream.is_none() {
            self.versao_upstream = outra.versao_upstream.clone();
        }
        if self.pasta.is_none() {
            self.pasta = outra.pasta.clone();
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Catalogo {
    pub entradas: Vec<Entrada>,
    /// De onde veio (para a interface dizer; ex.: "cache local do QuiverLauncher").
    pub fonte: String,
    pub atualizado_em: Option<SystemTime>,
    /// O que não deu para ler, com o motivo. Não derruba o resto.
    pub problemas: Vec<String>,
}

impl Catalogo {
    /// Junta outro catálogo a este, por slug: entradas novas entram, repetidas se enriquecem.
    pub fn enriquecer(&mut self, outro: Catalogo) {
        let mut por_slug: BTreeMap<String, usize> =
            self.entradas.iter().enumerate().map(|(i, e)| (e.slug.clone(), i)).collect();
        for e in outro.entradas {
            match por_slug.get(&e.slug) {
                Some(&i) => self.entradas[i].enriquecer(&e),
                None => {
                    por_slug.insert(e.slug.clone(), self.entradas.len());
                    self.entradas.push(e);
                }
            }
        }
        self.problemas.extend(outro.problemas);
        if self.atualizado_em.is_none() {
            self.atualizado_em = outro.atualizado_em;
        }
        if !outro.fonte.is_empty() {
            self.fonte = if self.fonte.is_empty() { outro.fonte } else { format!("{} + {}", self.fonte, outro.fonte) };
        }
    }

    pub fn ordenar_por_jogo(&mut self) {
        self.entradas.sort_by(|a, b| a.jogo.to_lowercase().cmp(&b.jogo.to_lowercase()).then_with(|| a.projeto.cmp(&b.projeto)));
    }
}

/// De onde um catálogo pode vir. Cada implementação lê um lugar; nenhuma bate em rede sem o
/// usuário pedir (a atualização pela rede é uma ação explícita, nunca a leitura).
pub trait Fonte {
    fn nome(&self) -> String;
    fn ler(&self) -> Result<Catalogo>;
}

/// `TheLegendOfZeldaTheWindWakerHD` → `The Legend Of Zelda The Wind Waker HD`;
/// `Bomberman64` → `Bomberman 64`; `LEGOIsland` → `LEGO Island`.
pub fn separar_pascal_case(texto: &str) -> String {
    let letras: Vec<char> = texto.chars().collect();
    let mut saida = String::with_capacity(texto.len() + 8);
    for (i, &c) in letras.iter().enumerate() {
        if i > 0 {
            let ant = letras[i - 1];
            let prox = letras.get(i + 1).copied();
            // depois de dígito, só começa palavra se vier minúscula ("3DPinball" é "3D Pinball")
            let depois_de_numero = ant.is_ascii_digit() && prox.map(|p| p.is_lowercase()).unwrap_or(false);
            let inicio_de_palavra = c.is_uppercase() && (ant.is_lowercase() || depois_de_numero);
            let fim_de_sigla = c.is_uppercase() && ant.is_uppercase() && prox.map(|p| p.is_lowercase()).unwrap_or(false);
            let inicio_de_numero = c.is_ascii_digit() && ant.is_alphabetic();
            if inicio_de_palavra || fim_de_sigla || inicio_de_numero {
                saida.push(' ');
            }
        }
        saida.push(c);
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tags(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn separa_nomes_de_pasta() {
        assert_eq!(separar_pascal_case("TheLegendOfZeldaTheWindWakerHD"), "The Legend Of Zelda The Wind Waker HD");
        assert_eq!(separar_pascal_case("Bomberman64"), "Bomberman 64");
        assert_eq!(separar_pascal_case("LEGOIsland"), "LEGO Island");
        assert_eq!(separar_pascal_case("XCOMApocalypse"), "XCOM Apocalypse");
        assert_eq!(separar_pascal_case("DrMario"), "Dr Mario");
        assert_eq!(separar_pascal_case("OpenRCT2"), "Open RCT 2");
        assert_eq!(separar_pascal_case("3DPinballSpaceCadet"), "3D Pinball Space Cadet");
        assert_eq!(separar_pascal_case("AceCombat04ShatteredSkiesPCPort"), "Ace Combat 04 Shattered Skies PC Port");
    }

    #[test]
    fn infere_tipo_pelo_nome() {
        assert_eq!(TipoDeProjeto::inferir("ZeldaWWHDRecomp", &[]), TipoDeProjeto::Recompilacao);
        assert_eq!(TipoDeProjeto::inferir("BanjoRecompiled", &[]), TipoDeProjeto::Recompilacao);
        assert_eq!(TipoDeProjeto::inferir("IslePortable", &[]), TipoDeProjeto::Porte);
        assert_eq!(TipoDeProjeto::inferir("OpenRCT2", &[]), TipoDeProjeto::Reimplementacao);
        assert_eq!(TipoDeProjeto::inferir("sm64decomp", &[]), TipoDeProjeto::Descompilacao);
        assert_eq!(TipoDeProjeto::inferir("Dethrace", &[]), TipoDeProjeto::Desconhecido);
        assert_eq!(TipoDeProjeto::inferir("Dethrace", &tags(&["decompilation"])), TipoDeProjeto::Descompilacao);
    }

    fn entrada(slug: &str) -> Entrada {
        Entrada {
            slug: slug.into(),
            jogo: separar_pascal_case("DrMario"),
            jogo_inferido: true,
            projeto: "DrMarioNesRecomp".into(),
            tipo: TipoDeProjeto::Recompilacao,
            tipo_inferido: true,
            repositorio: "mstan/DrMarioNesRecomp".into(),
            provedor: "github".into(),
            url: "https://github.com/mstan/DrMarioNesRecomp".into(),
            versao_verificada: Some("v1.3.1".into()),
            versao_upstream: Some("v1.3.1".into()),
            descricao: None,
            consoles: vec![],
            tags: vec![],
            disco: Disco::default(),
            pasta: Some("DrMario-DrMarioNesRecomp".into()),
        }
    }

    #[test]
    fn enriquecer_preenche_sem_sobrescrever_o_conhecido() {
        let mut base = Catalogo { entradas: vec![entrada("dr-mario")], fonte: "cache".into(), ..Default::default() };
        let mut rica = entrada("dr-mario");
        rica.jogo = "Dr. Mario".into();
        rica.jogo_inferido = false;
        rica.descricao = Some("Recompilação do Dr. Mario do NES".into());
        rica.consoles = tags(&["NES"]);
        rica.versao_upstream = Some("v9.9.9".into());
        let mut nova = entrada("outro-jogo");
        nova.slug = "outro-jogo".into();
        base.enriquecer(Catalogo { entradas: vec![rica, nova], fonte: "api".into(), ..Default::default() });
        assert_eq!(base.entradas.len(), 2);
        let e = &base.entradas[0];
        assert_eq!(e.jogo, "Dr. Mario");
        assert!(!e.jogo_inferido);
        assert_eq!(e.consoles, tags(&["NES"]));
        assert_eq!(e.versao_upstream.as_deref(), Some("v1.3.1"), "o que já era conhecido fica");
        assert_eq!(base.fonte, "cache + api");
    }

    #[test]
    fn atualizacao_quando_upstream_difere() {
        let mut e = entrada("x");
        assert!(!e.tem_atualizacao());
        e.versao_upstream = Some("v2".into());
        assert!(e.tem_atualizacao());
        e.versao_verificada = None;
        assert!(!e.tem_atualizacao());
    }
}
