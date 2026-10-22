//! Receitas de execução DESTA máquina (RF-01, `documentacao/03-requisitos.md`).
//!
//! O que faz um jogo abrir direito depende do computador, não do jogo: neste laptop Optimus
//! são as três variáveis de offload da Nvidia; num desktop de placa única não é nada. Por isso
//! isso é DADO, em `<raiz>/maquina.toml`, nunca constante no código:
//!
//! ```toml
//! [receitas.nvidia-optimus]
//! descricao = "Laptop Optimus: manda o jogo para a GTX pela Nvidia (Vulkan)"
//! prefixo = ["taskset", "-c", "0-3"]          # comando que embrulha o jogo (opcional)
//! [receitas.nvidia-optimus.ambiente]           # variáveis de ambiente
//! __NV_PRIME_RENDER_OFFLOAD = "1"
//! __VK_LAYER_NV_optimus = "NVIDIA_only"
//! __GLX_VENDOR_LIBRARY_NAME = "nvidia"
//! ```
//! Na v1 o arquivo é escrito à mão e tem um perfil só. Detecção automática, receitas
//! compartilhadas entre amigos e erro que aponta para a receita vêm depois (RF-01).
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::raiz::Raiz;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Receita {
    #[serde(default)]
    pub descricao: Option<String>,
    #[serde(default)]
    pub prefixo: Vec<String>,
    #[serde(default)]
    pub ambiente: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Maquina {
    #[serde(default)]
    pub receitas: BTreeMap<String, Receita>,
}

impl Maquina {
    /// Sem `maquina.toml` a máquina é "sem receitas"; isso não é erro.
    pub fn ler(raiz: &Raiz) -> Result<Maquina> {
        let arquivo = raiz.maquina();
        if !arquivo.is_file() {
            return Ok(Maquina::default());
        }
        let texto = std::fs::read_to_string(&arquivo)
            .with_context(|| format!("não consegui ler {}", arquivo.display()))?;
        toml::from_str(&texto).with_context(|| format!("{} mal formado", arquivo.display()))
    }

    pub fn receita(&self, nome: &str) -> Option<&Receita> {
        self.receitas.get(nome)
    }
}

/// Avisos sobre o estado da máquina que o launcher só pode avisar, não consertar.
pub fn avisos() -> Vec<String> {
    let mut lista = Vec::new();
    if let Ok(v) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        if v.trim() == "1" {
            lista.push(
                "Turbo da CPU desligado (intel_pstate/no_turbo=1): o jogo roda bem mais devagar. Ligar exige \
                 root: `echo 0 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo`."
                    .to_string(),
            );
        }
    }
    lista
}
