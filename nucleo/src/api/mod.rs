//! O que a interface Flutter enxerga. Funções finas; a lógica mora nos outros módulos.
//! Structs simples, não enums com dados: a ponte exigiria `freezed` no lado Dart só para isso.
use anyhow::Result;

use crate::biblioteca;
use crate::catalogo::{self, Fonte};
use crate::configuracao::{Configuracao, Tema};
use crate::execucao;
use crate::maquina::{self, Maquina};
use crate::nome::{NOME_COMPLETO, NOME_DO_APP};
use crate::raiz::Raiz;

#[flutter_rust_bridge::frb(init)]
pub fn iniciar() {
    flutter_rust_bridge::setup_default_user_utils();
    let _ = Raiz::descobrir().garantir();
}

#[flutter_rust_bridge::frb(sync)]
pub fn nome_do_app() -> String {
    NOME_DO_APP.to_string()
}

/// Frase que acompanha o nome (subtítulo, tooltip).
#[flutter_rust_bridge::frb(sync)]
pub fn nome_completo() -> String {
    NOME_COMPLETO.to_string()
}

#[flutter_rust_bridge::frb(sync)]
pub fn raiz() -> String {
    Raiz::descobrir().caminho.to_string_lossy().to_string()
}

// ---- biblioteca ----

pub struct JogoInfo {
    pub id: String,
    pub nome: String,
    pub projeto: Option<String>,
    pub console: Option<String>,
    pub descricao: Option<String>,
    pub tags: Vec<String>,
    pub capa: Option<String>,
    pub executavel: String,
    pub executavel_existe: bool,
    pub receita: Option<String>,
    pub receita_existe: bool,
    pub saves: Option<String>,
    pub pasta: String,
    /// Qual cópia do jogo esta recomp cobre, em texto (RF-05); "não informado" quando falta.
    pub disco: String,
    pub disco_informado: bool,
}

/// Entradas da biblioteca; a segunda lista traz o que não deu para ler, com o motivo.
pub struct Biblioteca {
    pub jogos: Vec<JogoInfo>,
    pub problemas: Vec<String>,
}

/// Situação de um jogo: `situacao` é "parado", "rodando" ou "encerrado".
pub struct Situacao {
    pub situacao: String,
    pub pid: Option<u32>,
    pub segundos: Option<u32>,
    pub codigo: Option<i32>,
}

#[flutter_rust_bridge::frb(sync)]
pub fn biblioteca() -> Biblioteca {
    let raiz = Raiz::descobrir();
    let maquina = Maquina::ler(&raiz).unwrap_or_default();
    let mut jogos = Vec::new();
    let mut problemas = Vec::new();
    for item in biblioteca::listar(&raiz) {
        match item {
            Ok(j) => {
                let receita = j.toml.execucao.receita.clone();
                let receita_existe = receita.as_deref().map(|r| maquina.receita(r).is_some()).unwrap_or(true);
                jogos.push(JogoInfo {
                    id: j.id.clone(),
                    nome: j.toml.nome.clone(),
                    projeto: j.toml.projeto.clone(),
                    console: j.toml.console.clone(),
                    descricao: j.toml.descricao.clone(),
                    tags: j.toml.tags.clone(),
                    capa: j.capa.as_ref().map(|c| c.to_string_lossy().to_string()),
                    executavel: j.executavel().to_string_lossy().to_string(),
                    executavel_existe: j.executavel().is_file(),
                    receita,
                    receita_existe,
                    saves: j.toml.dados.saves.clone(),
                    pasta: j.pasta.to_string_lossy().to_string(),
                    disco: j.toml.disco.descrever(j.toml.console.as_deref()),
                    disco_informado: j.toml.disco.informado(),
                });
            }
            Err(e) => problemas.push(format!("{e:#}")),
        }
    }
    Biblioteca { jogos, problemas }
}

/// Abre o jogo com a receita da máquina. Devolve o PID do processo lançado.
pub fn jogar(id: String) -> Result<u32> {
    let raiz = Raiz::descobrir();
    raiz.garantir()?;
    let jogo = biblioteca::achar(&raiz, &id)?;
    let maquina = Maquina::ler(&raiz)?;
    let receita = match jogo.toml.execucao.receita.as_deref() {
        Some(nome) => Some(maquina.receita(nome).ok_or_else(|| {
            anyhow::anyhow!("a receita '{nome}' não existe em {}", raiz.maquina().display())
        })?),
        None => None,
    };
    execucao::lancar(&jogo, receita, &raiz.registros())
}

/// A linha de comando que `jogar` usaria, para a interface mostrar.
#[flutter_rust_bridge::frb(sync)]
pub fn linha_de_comando(id: String) -> Result<String> {
    let raiz = Raiz::descobrir();
    let jogo = biblioteca::achar(&raiz, &id)?;
    let maquina = Maquina::ler(&raiz)?;
    let receita = jogo.toml.execucao.receita.as_deref().and_then(|n| maquina.receita(n));
    let mut partes: Vec<String> = receita
        .map(|r| r.ambiente.iter().map(|(k, v)| format!("{k}={v}")).collect())
        .unwrap_or_default();
    partes.extend(execucao::linha_de_comando(&jogo, receita));
    Ok(partes.join(" "))
}

#[flutter_rust_bridge::frb(sync)]
pub fn estado(id: String) -> Situacao {
    match execucao::estado(&id) {
        execucao::Estado::Parado => {
            Situacao { situacao: "parado".into(), pid: None, segundos: None, codigo: None }
        }
        execucao::Estado::Rodando { pid, segundos } => Situacao {
            situacao: "rodando".into(),
            pid: Some(pid),
            segundos: Some(segundos.min(u32::MAX as u64) as u32),
            codigo: None,
        },
        execucao::Estado::Encerrado { codigo } => {
            Situacao { situacao: "encerrado".into(), pid: None, segundos: None, codigo: Some(codigo) }
        }
    }
}

/// SIGTERM ao processo do jogo (saída limpa). Devolve o que foi feito.
pub fn fechar(id: String) -> Result<String> {
    let raiz = Raiz::descobrir();
    let jogo = biblioteca::achar(&raiz, &id)?;
    execucao::fechar(&id, jogo.toml.execucao.processo.as_deref())
}

/// SIGKILL no jogo e nos grupos. Último recurso; a interface só oferece depois de esperar o SIGTERM.
pub fn forcar(id: String) -> Result<String> {
    let raiz = Raiz::descobrir();
    let jogo = biblioteca::achar(&raiz, &id)?;
    execucao::forcar(&id, jogo.toml.execucao.processo.as_deref())
}

/// Fim do registro da execução atual (ou da última).
#[flutter_rust_bridge::frb(sync)]
pub fn registro(id: String, linhas: u32) -> String {
    match execucao::registro_atual(&id) {
        Some(arquivo) => execucao::cauda(&arquivo, linhas as usize),
        None => String::new(),
    }
}

#[flutter_rust_bridge::frb(sync)]
pub fn avisos() -> Vec<String> {
    maquina::avisos()
}

// ---- catálogo (aba Explorar) ----

/// Uma entrada do catálogo já explicada: que jogo, de que console, o que o repositório faz.
pub struct EntradaDoCatalogo {
    pub slug: String,
    pub jogo: String,
    /// O nome do jogo saiu do nome da pasta, não de uma fonte que o conhece.
    pub jogo_inferido: bool,
    pub projeto: String,
    /// `recompilacao`, `descompilacao`, `porte`, `reimplementacao` ou `desconhecido`.
    pub tipo: String,
    pub tipo_rotulo: String,
    pub tipo_explicacao: String,
    /// O tipo foi chutado pelo nome do projeto: a interface marca como inferido.
    pub tipo_inferido: bool,
    pub repositorio: String,
    pub provedor: String,
    pub url: String,
    pub versao_verificada: Option<String>,
    pub versao_upstream: Option<String>,
    pub tem_atualizacao: bool,
    pub descricao: Option<String>,
    pub consoles: Vec<String>,
    pub tags: Vec<String>,
    /// Qual cópia do jogo esta recomp cobre (RF-05); "não informado" quando a fonte não diz.
    pub disco: String,
    pub disco_informado: bool,
    pub pasta: Option<String>,
}

impl From<&catalogo::Entrada> for EntradaDoCatalogo {
    fn from(e: &catalogo::Entrada) -> Self {
        let console = e.consoles.first().map(String::as_str);
        EntradaDoCatalogo {
            slug: e.slug.clone(),
            jogo: e.jogo.clone(),
            jogo_inferido: e.jogo_inferido,
            projeto: e.projeto.clone(),
            tipo: e.tipo.id().to_string(),
            tipo_rotulo: e.tipo.rotulo().to_string(),
            tipo_explicacao: e.tipo.explicacao().to_string(),
            tipo_inferido: e.tipo_inferido,
            repositorio: e.repositorio.clone(),
            provedor: e.provedor.clone(),
            url: e.url.clone(),
            versao_verificada: e.versao_verificada.clone(),
            versao_upstream: e.versao_upstream.clone(),
            tem_atualizacao: e.tem_atualizacao(),
            descricao: e.descricao.clone(),
            consoles: e.consoles.clone(),
            tags: e.tags.clone(),
            disco: e.disco.descrever(console),
            disco_informado: e.disco.informado(),
            pasta: e.pasta.clone(),
        }
    }
}

pub struct CatalogoInfo {
    pub entradas: Vec<EntradaDoCatalogo>,
    /// De onde veio, para a interface dizer.
    pub fonte: String,
    /// Segundos desde 1970 da fonte mais antiga; a interface formata.
    pub atualizado_em: Option<u64>,
    pub problemas: Vec<String>,
    /// O arquivo que "Atualizar pela rede" deixou na raiz, se já houve uma atualização.
    pub arquivo_baixado: Option<String>,
}

/// O catálogo, lido do disco: o cache do QuiverLauncher desta máquina mais, se existir, o
/// catálogo baixado pela rede numa atualização anterior. Nunca bate em rede.
#[flutter_rust_bridge::frb(sync)]
pub fn catalogo() -> CatalogoInfo {
    let raiz = Raiz::descobrir();
    let mut cat = catalogo::quiver_cache::CacheDoQuiver::desta_maquina().ler().unwrap_or_else(|e| catalogo::Catalogo {
        problemas: vec![format!("{e:#}")],
        ..Default::default()
    });
    let baixado = catalogo::quiver_api::CatalogoBaixadoDoQuiver::na_raiz(&raiz);
    let arquivo_baixado = baixado.existe().then(|| baixado.arquivo.to_string_lossy().to_string());
    if baixado.existe() {
        match baixado.ler() {
            Ok(outro) => cat.enriquecer(outro),
            Err(e) => cat.problemas.push(format!("{e:#}")),
        }
    }
    cat.ordenar_por_jogo();
    CatalogoInfo {
        entradas: cat.entradas.iter().map(EntradaDoCatalogo::from).collect(),
        fonte: cat.fonte,
        atualizado_em: cat.atualizado_em.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
        problemas: cat.problemas,
        arquivo_baixado,
    }
}

/// Baixa o catálogo de quiverlauncher.com para `<raiz>/catalogo/` (a única função que usa
/// rede, e só quando o usuário clica). Devolve o que foi feito.
pub fn atualizar_catalogo() -> Result<String> {
    let raiz = Raiz::descobrir();
    let (arquivo, n) = catalogo::quiver_api::baixar(&raiz)?;
    Ok(format!("{n} entradas baixadas de quiverlauncher.com para {}", arquivo.display()))
}

// ---- configuração (tema, RF-06) ----

pub struct ConfiguracaoInfo {
    /// `sistema`, `claro` ou `escuro`.
    pub tema: String,
    /// O que não deu para ler, se houve (a interface segue com o padrão).
    pub problema: Option<String>,
}

#[flutter_rust_bridge::frb(sync)]
pub fn configuracao() -> ConfiguracaoInfo {
    match Configuracao::ler(&Raiz::descobrir()) {
        Ok(c) => ConfiguracaoInfo { tema: c.tema.id().to_string(), problema: None },
        Err(e) => ConfiguracaoInfo { tema: Tema::default().id().to_string(), problema: Some(format!("{e:#}")) },
    }
}

/// Guarda o tema escolhido em `<raiz>/configuracao.toml`.
pub fn definir_tema(tema: String) -> Result<()> {
    let raiz = Raiz::descobrir();
    let escolhido = Tema::de_id(&tema).ok_or_else(|| anyhow::anyhow!("tema desconhecido: {tema}"))?;
    let mut c = Configuracao::ler(&raiz).unwrap_or_default();
    c.tema = escolhido;
    c.gravar(&raiz)
}
