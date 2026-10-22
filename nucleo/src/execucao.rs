//! Abrir e fechar jogos. Regras que custaram caro a alguém (documentacao/02-rodar-o-bluewake.md):
//! guardar o PID e matar por PID, nunca por nome; `stdbuf -oL -eL` para a saída não ficar presa
//! no buffer; pedido de fechar com espera, porque saída suja deixa o jogo em modo seguro na
//! próxima vez; grupo de processos próprio para o jogo não morrer junto com o terminal.
//!
//! O que é de sistema operacional (sinais, grupos, `taskkill`) fica em `plataforma`, uma
//! versão por sistema; o resto é igual nos cinco alvos.
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{LazyLock, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};

use crate::biblioteca::Jogo;
use crate::maquina::Receita;

pub struct Processo {
    pub pid: u32,
    child: Child,
    pub inicio: Instant,
    pub registro: PathBuf,
    pub saida: Option<i32>,
}

#[derive(Clone, Debug)]
pub enum Estado {
    Parado,
    Rodando { pid: u32, segundos: u64 },
    Encerrado { codigo: i32 },
}

static PROCESSOS: LazyLock<Mutex<HashMap<String, Processo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn carimbo() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{s}")
}

fn existe_no_path(programa: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(programa).is_file()))
        .unwrap_or(false)
}

/// Monta a linha de comando: [stdbuf] + prefixo da receita + executável + argumentos.
pub fn linha_de_comando(jogo: &Jogo, receita: Option<&Receita>) -> Vec<String> {
    let mut linha = Vec::new();
    if existe_no_path("stdbuf") {
        linha.extend(["stdbuf", "-oL", "-eL"].map(String::from));
    }
    if let Some(r) = receita {
        linha.extend(r.prefixo.iter().cloned());
    }
    linha.push(jogo.executavel().to_string_lossy().to_string());
    linha.extend(jogo.toml.execucao.argumentos.iter().cloned());
    linha
}

pub fn lancar(jogo: &Jogo, receita: Option<&Receita>, registros: &Path) -> Result<u32> {
    let mut mapa = PROCESSOS.lock().unwrap();
    if let Some(p) = mapa.get_mut(&jogo.id) {
        if vivo(p) {
            bail!("{} já está aberto (pid {})", jogo.toml.nome, p.pid);
        }
    }
    let exe = jogo.executavel();
    if !exe.is_file() {
        bail!("executável não encontrado: {}", exe.display());
    }
    let linha = linha_de_comando(jogo, receita);
    std::fs::create_dir_all(registros)?;
    let registro = registros.join(format!("{}-{}.log", jogo.id, carimbo()));
    let arquivo = File::create(&registro)
        .with_context(|| format!("não consegui criar o registro {}", registro.display()))?;

    let mut cmd = Command::new(&linha[0]);
    cmd.args(&linha[1..]);
    if let Some(r) = receita {
        cmd.envs(r.ambiente.iter());
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::from(arquivo.try_clone()?))
        .stderr(Stdio::from(arquivo))
        .current_dir(crate::raiz::casa());
    plataforma::isolar(&mut cmd);
    let child = cmd
        .spawn()
        .with_context(|| format!("não consegui executar {}", linha.join(" ")))?;
    let pid = child.id();
    mapa.insert(
        jogo.id.clone(),
        Processo { pid, child, inicio: Instant::now(), registro, saida: None },
    );
    Ok(pid)
}

fn vivo(p: &mut Processo) -> bool {
    if p.saida.is_some() {
        return false;
    }
    match p.child.try_wait() {
        Ok(Some(status)) => {
            p.saida = Some(status.code().unwrap_or(-1));
            false
        }
        Ok(None) => true,
        Err(_) => false,
    }
}

pub fn estado(id: &str) -> Estado {
    let mut mapa = PROCESSOS.lock().unwrap();
    match mapa.get_mut(id) {
        None => Estado::Parado,
        Some(p) => {
            if vivo(p) {
                Estado::Rodando { pid: p.pid, segundos: p.inicio.elapsed().as_secs() }
            } else {
                Estado::Encerrado { codigo: p.saida.unwrap_or(-1) }
            }
        }
    }
}

pub fn registro_atual(id: &str) -> Option<PathBuf> {
    PROCESSOS.lock().unwrap().get(id).map(|p| p.registro.clone())
}

/// Pede ao jogo para fechar (saída limpa): no Unix, SIGTERM ao processo DO JOGO (o nome em
/// `execucao.processo`, procurado entre os descendentes do que lançamos — numa AppImage o
/// filho direto é o runtime, não o jogo); o SDL transforma o sinal em pedido de saída.
/// No Windows, `taskkill /PID`, que manda fechar a janela.
pub fn fechar(id: &str, nome_processo: Option<&str>) -> Result<String> {
    let mut mapa = PROCESSOS.lock().unwrap();
    let p = mapa.get_mut(id).ok_or_else(|| anyhow!("esse jogo não foi aberto por aqui"))?;
    if !vivo(p) {
        bail!("o jogo já não está rodando");
    }
    let alvo = nome_processo.and_then(|n| descendente_por_nome(p.pid, n)).unwrap_or(p.pid);
    let feito = plataforma::pedir_para_fechar(alvo)?;
    Ok(format!("{feito}; aguardando o jogo encerrar"))
}

/// Último recurso: mata o jogo e o que estiver pendurado nele. Deixa o jogo em modo seguro na
/// próxima abertura. Mata o filho que lançamos, o processo com o nome do jogo (se for um
/// descendente, como acontece com scripts embrulhando o binário) e o grupo de cada um.
/// Medido na AppImage do BlueWake: o runtime monta o squashfs num filho e faz `exec` do jogo no
/// próprio processo, então o PID lançado vira o PID do `bluewake`.
pub fn forcar(id: &str, nome_processo: Option<&str>) -> Result<String> {
    let mut mapa = PROCESSOS.lock().unwrap();
    let p = mapa.get_mut(id).ok_or_else(|| anyhow!("esse jogo não foi aberto por aqui"))?;
    if !vivo(p) {
        bail!("o jogo já não está rodando");
    }
    let mut alvos = vec![p.pid];
    if let Some(filho) = nome_processo.and_then(|n| descendente_por_nome(p.pid, n)) {
        alvos.insert(0, filho);
    }
    let feitos: Vec<String> = alvos.into_iter().map(plataforma::matar).collect();
    Ok(format!("{} em {}", plataforma::NOME_DO_GOLPE, feitos.join(", ")))
}

/// Procura, entre os descendentes de `raiz`, um processo cujo `comm` seja `nome`.
/// Só Linux (lê /proc); em outro sistema devolve None e o chamador usa o filho direto.
#[cfg(target_os = "linux")]
fn descendente_por_nome(raiz: u32, nome: &str) -> Option<u32> {
    let mut filhos: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut nomes: HashMap<u32, String> = HashMap::new();
    for entrada in std::fs::read_dir("/proc").ok()?.flatten() {
        let Ok(pid) = entrada.file_name().to_string_lossy().parse::<u32>() else { continue };
        let Ok(stat) = std::fs::read_to_string(entrada.path().join("stat")) else { continue };
        // "<pid> (<comm>) <estado> <ppid> ..." — comm pode ter espaços, por isso o rparen.
        let (Some(a), Some(b)) = (stat.find('('), stat.rfind(')')) else { continue };
        let comm = stat[a + 1..b].to_string();
        let ppid = stat[b + 1..].split_whitespace().nth(1).and_then(|s| s.parse::<u32>().ok());
        if let Some(ppid) = ppid {
            filhos.entry(ppid).or_default().push(pid);
        }
        nomes.insert(pid, comm);
    }
    let mut fila = vec![raiz];
    while let Some(p) = fila.pop() {
        if p != raiz && nomes.get(&p).map(|n| n == nome).unwrap_or(false) {
            return Some(p);
        }
        if let Some(f) = filhos.get(&p) {
            fila.extend(f.iter().copied());
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn descendente_por_nome(_raiz: u32, _nome: &str) -> Option<u32> {
    None
}

/// Últimas `linhas` de um arquivo de registro.
pub fn cauda(arquivo: &Path, linhas: usize) -> String {
    let Ok(texto) = std::fs::read_to_string(arquivo) else { return String::new() };
    let todas: Vec<&str> = texto.lines().collect();
    let inicio = todas.len().saturating_sub(linhas);
    todas[inicio..].join("\n")
}

/// Sinais e grupos de processo no Unix (Linux, macOS, Android).
#[cfg(unix)]
mod plataforma {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use anyhow::{bail, Result};

    pub const NOME_DO_GOLPE: &str = "SIGKILL";

    /// Grupo de processos próprio: o jogo não morre junto com quem o lançou.
    pub fn isolar(cmd: &mut Command) {
        cmd.process_group(0);
    }

    pub fn pedir_para_fechar(pid: u32) -> Result<String> {
        sinal(pid, libc::SIGTERM)?;
        Ok(format!("SIGTERM enviado ao pid {pid}"))
    }

    /// SIGKILL no processo e no grupo dele. Devolve o que foi feito, para o registro.
    pub fn matar(pid: u32) -> String {
        let grupo = unsafe { libc::getpgid(pid as libc::pid_t) };
        if grupo > 0 {
            unsafe { libc::kill(-grupo, libc::SIGKILL) };
        }
        unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
        format!("{pid} (grupo {grupo})")
    }

    fn sinal(pid: u32, sinal: libc::c_int) -> Result<()> {
        let r = unsafe { libc::kill(pid as libc::pid_t, sinal) };
        if r != 0 {
            bail!("não consegui sinalizar o pid {pid}: {}", std::io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Windows não tem sinais: o `taskkill` do sistema pede o fechamento (sem `/F`, manda
/// WM_CLOSE à janela) ou derruba a árvore (`/F /T`). NÃO TESTADO em máquina Windows ainda;
/// é o caminho documentado pela Microsoft e o que o CI vai exercitar.
#[cfg(windows)]
mod plataforma {
    use std::process::Command;

    use anyhow::{bail, Context, Result};

    pub const NOME_DO_GOLPE: &str = "taskkill /F /T";

    /// Sem grupo de processos no Windows; o jogo já nasce em processo próprio.
    pub fn isolar(_cmd: &mut Command) {}

    pub fn pedir_para_fechar(pid: u32) -> Result<String> {
        let saida = Command::new("taskkill").args(["/PID", &pid.to_string()]).output().context("não consegui rodar taskkill")?;
        if !saida.status.success() {
            bail!("taskkill recusou o pid {pid}: {}", String::from_utf8_lossy(&saida.stderr).trim());
        }
        Ok(format!("pedido de fechar (taskkill) enviado ao pid {pid}"))
    }

    pub fn matar(pid: u32) -> String {
        let _ = Command::new("taskkill").args(["/F", "/T", "/PID", &pid.to_string()]).output();
        format!("{pid} (com a árvore)")
    }
}
