# Requisitos

Formato: `id`, `status`, `revisado em`. Status: `aberto`, `em análise`, `aceito`, `descartado`.
Quem decide é o dono; a gerente mantém este arquivo.

---

## RF-01 — Receitas de execução por máquina

- **status:** aceito
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### O que é

Um jogo não roda igual em duas máquinas. O que faz ele abrir direito é um conjunto de ajustes
que depende do **computador**, não do jogo: variáveis de ambiente, caminhos de biblioteca,
qual binário chamar, argumentos, diretório de trabalho, e passos antes e depois.

O launcher precisa tratar isso como dado de primeira classe — uma **receita** — e não como
constante no código.

### Por que, com o caso real que já temos

Este laptop é Optimus: tem uma placa Intel integrada e uma GTX 1060. O BlueWake aberto sem
preparo sai pela Intel e fica lento. Para sair pela Nvidia são necessárias três variáveis
(`__NV_PRIME_RENDER_OFFLOAD`, `__VK_LAYER_NV_optimus`, `__GLX_VENDOR_LIBRARY_NAME`).

Num desktop com uma placa só, essas variáveis não fazem sentido. No Windows, o mecanismo é
outro. No celular, não existe. **É a mesma recomp, com receitas diferentes.** Gravar as três
variáveis no código seria acertar esta máquina e errar todas as outras.

Outros exemplos que já conhecemos: a saída do jogo precisa de `stdbuf` para não ficar presa no
buffer; o `settings.ini` do jogador vence as variáveis de ambiente, então forçar configuração
exige um `XDG_CONFIG_HOME` próprio; e o turbo da CPU desligado muda muito o desempenho, o que o
launcher deve **avisar**, não consertar.

### O que se espera do desenho

- a receita é **dados**, editável pelo usuário, não código;
- separar o que é **do jogo** (binário, argumentos, onde ficam os saves) do que é **da
  máquina** (placa de vídeo, caminhos, sistema). A mesma recomp em duas máquinas tem a mesma
  parte de jogo e partes de máquina diferentes;
- **detecção automática** onde der: reconhecer Optimus e propor a receita pronta, em vez de
  exigir que o usuário saiba o nome das variáveis;
- a receita deve poder ser **compartilhada** entre amigos, já que cada um compila o seu jogo e
  vai esbarrar nos mesmos ajustes. Compartilhar receita é diferente de compartilhar binário:
  receita é texto, não tem código do jogo dentro;
- quando o jogo não abrir, o erro tem que apontar para a receita, não para um log cru.

### Em aberto, para investigar depois

Como representar isso sem virar um mini sistema de build. Ponto de partida sugerido: perfil de
máquina detectado na primeira execução, mais sobreposições por jogo. **Não resolver agora** —
o dono pediu para registrar e investigar depois.

---

## RF-02 — O produto se apresenta como um *harness* de recompilação

- **status:** aceito
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

A identidade do produto é **neutra e de ferramenta**: ele se apresenta como um *harness* para
recompilar e rodar jogos, não como "a Steam de fulano" nem com cara de um jogo específico.

Consequências de desenho:
- nome, ícone e vocabulário da interface **não** puxam para uma franquia;
- **temas vêm depois**, como camada: o usuário escolhe a cara, o programa não nasce com uma;
- a biblioteca e o catálogo falam de "recomp", "receita", "estado", não de personagens.

Isto não é decoração: define o nome do projeto e o tom de toda a interface.

---

## RF-03 — Pipeline de recompilação com apoio de IA para resolver erros

- **status:** aberto, a investigar
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### A ideia

Compilar uma recomp é longo e quebra por motivos obscuros. O dono quer poder **ligar IAs
gratuitas** ao harness para ajudar a diagnosticar e resolver esses erros.

### Por que isso exige um pipeline bem definido antes

Não dá para jogar um log de 1 GB numa IA e esperar resposta útil. Para a ideia funcionar, a
recompilação precisa deixar de ser "um script que roda 1h45 e falha" e virar uma sequência de
**etapas nomeadas, com entradas, saídas e critério de sucesso** — que é o que o dono chamou de
pipeline. Com isso:

- a falha é localizada numa etapa, não no build inteiro;
- o contexto que vai para a IA é pequeno e relevante: a etapa, o comando, o trecho de log que
  falhou, as versões envolvidas;
- dá para repetir só a etapa que falhou, em vez de recomeçar;
- dá para acumular um catálogo de falhas conhecidas com a solução, e consultar antes de gastar
  uma chamada de IA.

Temos caso real para desenhar em cima: o build do BlueWake já é por etapas numeradas
(`scripts/linux/build.py`, com logs separados por etapa em `build/linux/logs/`), e as falhas
conhecidas dele já estão catalogadas em `~/.claude/projects/-home-neps/memory/projeto-recomps.md`
e em `~/projetos/RECOMPS/notas/`. Isso é o rascunho de um pipeline e de uma base de falhas.

### Em aberto

Quais IAs, com que limites de uso, o que pode sair da máquina (log de build de código traduzido
é material sensível — ver RF-01 sobre receita ser texto e binário não), e se a IA sugere ou
aplica correção. **Não resolver agora**, a pedido do dono.

---

## RF-04 — Instalação no PC em um comando, a partir do repositório

- **status:** aceito com ressalva (ver "o problema" abaixo)
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### O que o dono quer

A ergonomia: **clonar o repositório, rodar um comando, está funcionando.** Ele citou
`cargo install` como a forma. O valor está na experiência, não na ferramenta específica.

### O problema, dito antes de prometer

`cargo install` **não constrói este app**, e vale entender por quê antes de desenhar em cima:

- `cargo install` compila um alvo Rust e copia **um binário solto** para `~/.cargo/bin`;
- este app é Flutter na interface. O produto final não é um binário solto: é um pacote com o
  executável, a casca GTK, os assets, os dados de ICU e as bibliotecas nativas ao lado. Quem
  monta isso é `flutter build linux`, não o cargo;
- o cargo não sabe invocar o Flutter nem carregar os assets junto. O binário instalado abriria
  sem interface.

Ou seja: do jeito literal, não fecha. Mas a **ergonomia pedida fecha**, e é o que importa.

### O desenho proposto

Manter exatamente o que o dono pediu — clone mais um comando do cargo — usando o padrão
`cargo xtask`: um subprojeto Rust no repositório que **orquestra** a instalação.

```
git clone <repo> && cd <repo>
cargo run --release -p instalador
```

Esse instalador em Rust: confere as dependências (Flutter, GTK, cmake, ninja), chama
`flutter build linux`, compila o núcleo Rust, monta a pasta única do RF-01 e deixa o atalho
pronto. Uma linha para o usuário, verdade técnica por baixo.

Alternativa a considerar quando houver release: `cargo binstall`, que baixa binário pronto em
vez de compilar.

### Dois públicos, dois caminhos

| quem | caminho | exige |
|---|---|---|
| quem desenvolve ou quer a versão do repositório | clone + um comando cargo | SDK do Flutter, GTK, cmake, ninja |
| **o amigo que só quer jogar** | AppImage ou pacote pronto, baixado | nada |

Isto importa: exigir SDK do Flutter de um amigo para ele entrar numa partida mata o projeto.
O caminho do repositório é para quem desenvolve; o jogador comum recebe pacote pronto.
Nas outras plataformas o caminho do repositório nem existe (iOS não instala nada sem Xcode).

---

## RF-05 — O catálogo diz qual versão do jogo cada recomp cobre

- **status:** aceito
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### O que é

Na aba Explorar, cada recomp precisa declarar **explicitamente** qual versão do jogo ela cobre.
Não é detalhe de catálogo: é o que decide se a sua cópia serve ou não.

### Por que importa de verdade

Uma recomp é feita a partir de uma imagem de disco específica. Trocar a região ou a revisão
muda os endereços e a recompilação simplesmente não aceita. Quem tem disco PAL e tenta usar uma
recomp NTSC descobre isso depois de baixar a ferramenta, preparar o ambiente e falhar.

Já temos o modelo pronto no nosso próprio registro, `recomps.d/bluewake.conf`:

```
DISCO_ID="GZLE01"
DISCO_REV="0"
```

`GZLE01` carrega jogo, **região** (a letra `E` é NTSC-U) e editora; `0` é a revisão da
prensagem. É exatamente a informação que falta no catálogo do Quiver.

### O que se espera na interface

- mostrar jogo, console, **identificador do disco**, **região** e **revisão**, em texto que uma
  pessoa entenda — "GameCube, NTSC-U, revisão 0", não só o código cru;
- deixar claro quando a informação **não existir** para aquele recomp, em vez de omitir. "Não
  informado" é um dado útil; silêncio não é;
- quando o dono já tiver o disco, a comparação é óbvia e deve ser feita por nós, não por ele.

### De onde a informação vem (investigado em 22/10/2026 pelo implementador)

- **Cache local do QuiverLauncher**: só slug, repositório e versões. Nada de disco.
- **API do Quiver** (`/api/v1/apps`, confirmado com uma chamada): traz jogo, console, tipo de
  projeto e descrição, mas **nenhum campo de disco, região ou revisão**. Por isso a aba Explorar
  mostra "não informado" em todo o catálogo hoje, e a biblioteca mostra o que o `jogo.toml` traz
  na seção `[disco]` (`id`, `revisao`), decodificado por `nucleo/src/disco.rs`.
- **Onde pode vir a existir**: (1) no nosso catálogo em git (D-10), um `[disco]` por entrada,
  preenchido por quem compilou — a receita já viaja como texto, o disco vai junto; (2) o README
  de cada recomp costuma dizer ("requires the USA version"), o que dá uma colheita
  semi-automática a confirmar à mão; (3) a comparação local: o launcher lê o cabeçalho da imagem
  que a pessoa tem (ID e revisão, como o `recomps.sh` já faz) e compara com o `[disco]` da
  entrada. É o item (3) que faz a comparação por nós.

---

## RF-06 — Seletor de cores e temas da interface

- **status:** aceito
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

O usuário escolhe a aparência do POG. É a primeira parcela do RF-02, que já dizia que o produto
nasce neutro e os temas entram **como camada**.

Consequência de desenho, e ela é a parte que importa: **nenhuma cor pode estar escrita no meio
dos componentes.** Toda a interface lê de um tema, e o seletor troca o tema inteiro. Se as cores
nascerem espalhadas, cada tema novo vira caçada a literal.

Mínimo da primeira versão: claro, escuro, e seguir o sistema. Depois, paletas nomeadas e tema
definido pelo usuário. Acessibilidade entra aqui: contraste suficiente é requisito, não enfeite.

---

## RF-07 — Instalar ou atualizar no Windows com dois cliques

- **status:** aceito
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### O que é

O amigo do dono não entende de tecnologia. Em vez de colar comando no `cmd`, ele recebe **um
arquivo só**, `POG-instalar-ou-atualizar.bat`, anexado na release junto com o zip, que ele clica
duas vezes e resolve: instala se não houver POG; se houver, compara a versão instalada com a última
publicada e atualiza só quando precisa, dizendo o que fez.

### O que se espera

- sem senha de administrador: instala em `%LOCALAPPDATA%\pog`, atalho na Área de Trabalho;
- console com título, passos numerados e mensagens em português de gente; nada de saída crua
  de `curl`;
- erros em português útil, cada um com o que fazer: sem internet, GitHub fora, Windows velho sem
  `curl`/`tar`, antivírus barrando;
- `pause` no fim, sempre, inclusive nos erros;
- a versão instalada fica em `versao.txt` ao lado do executável, para a comparação na próxima vez.

### Como foi feito (22/10/2026, implementador)

`instalador/POG-instalar-ou-atualizar.bat`, anexado à release pelo workflow `Pacotes`. Descobre a
última versão pelo redirecionamento de `github.com/<repo>/releases/latest` (sem JSON), baixa o
`pog-windows-x64.zip` com `curl -# -f -L`, extrai com `tar`, copia com `robocopy /E`, cria o atalho
por PowerShell (`WScript.Shell`) na pasta que o Windows diz ser a Área de Trabalho (cobre OneDrive).
Conferido aqui: labels e `goto` casam, CRLF em todas as linhas, sem BOM, lógica de versão simulada
em shell. **Não testado em Windows de verdade** (não há Windows nesta máquina); hipóteses marcadas
no cabeçalho do arquivo: Windows 10 de 2018 em diante ou 11, acentos via `chcp 65001`.

---

## RF-08 — Steam Deck: o POG e a recomp rodam lá?

- **status:** em análise
- **revisado em:** 22/10/2026
- **pedido por:** o dono, 22/10/2026

### O que o dono perguntou

O amigo tem um Steam Deck. O POG e o recomp rodam nele? Nenhum de nós tem um Deck; o amigo tem.
Não se constrói nada para SteamOS antes de o dono decidir.

### O lado do recomp (levantado pela gerente, 22/10/2026)

A AppImage do BlueWake é x86-64 (mesma arquitetura do Deck); `libc++`, `libc++abi` e `libunwind`
vão dentro dela (o problema antigo de `GLIBCXX` do sistema não existe nesta build); a **única
exigência dura é `glibc >= 2.38`**, medida em todos os binários; o jogo usa Vulkan e o Deck tem AMD
RDNA2 com Vulkan. Provavelmente roda; a dúvida se resume à glibc do SteamOS do amigo.

### O comando que o amigo roda (modo área de trabalho, no Konsole)

```bash
getconf GNU_LIBC_VERSION; grep -m1 PRETTY_NAME /etc/os-release
```

Sai algo como `glibc 2.38` e o nome do SteamOS. **Se o número da glibc for 2.38 ou maior, o jogo
roda.** (Testado aqui: devolve `glibc 2.41` nesta Debian.)

### O lado do POG (avaliação do implementador, 22/10/2026, sem Deck para testar)

- **O POG precisa de mais glibc que o jogo.** Medido no pacote compilado nesta máquina:
  `libnucleo.so` exige `GLIBC_2.39` (o Rust puxa símbolos da glibc onde foi compilado), o runner
  `pog` exige 2.34 e o motor do Flutter só 2.18. O CI de hoje compila em `ubuntu-latest` (24.04,
  glibc 2.39), então o pacote Linux da release também exige 2.39. **Num Deck com glibc 2.38, o
  POG não abre e o jogo abre.** Saída barata, se o dono quiser: compilar o Linux no runner
  `ubuntu-22.04` (glibc 2.35), uma linha no workflow, que cobre o Deck e qualquer distro de 2022
  em diante. Não é build para SteamOS, é só exigir menos; mas fica para o dono decidir.
- **Raiz somente leitura.** Nada de instalar pacote no sistema. O POG já vive inteiro no home
  (`~/jogos/pog` para dados; o `tar.gz` extraído em `~/Applications` ou `~/.local/share/pog`), então
  isso não o atrapalha.
- **Bibliotecas do sistema que o POG usa** (medido com `objdump`): GTK 3, GLib/GIO, Cairo, Pango,
  HarfBuzz, Fontconfig, GdkPixbuf, Epoxy, libstdc++, zlib. O SteamOS traz Plasma e GTK 3 na imagem
  (hipótese forte: o modo área de trabalho tem apps GTK); confere-se com
  `ldconfig -p | grep -c 'libgtk-3\|libepoxy'` (dois ou mais = tem).
- **AppImage é o formato certo para lá?** Para o jogo, sim (já é). Para o POG, o `tar.gz` basta
  enquanto o GTK do sistema servir; AppImage só vale a pena se faltar biblioteca, e aí embrulha o
  bundle do Flutter com `linuxdeploy` + plugin GTK. Não fazer antes de o amigo medir.
- **Modo jogo.** Entra como **jogo não-Steam** apontando para o `pog` (ou um script que o chama).
  O gamescope roda o POG; quando o POG lança o BlueWake como processo filho, a janela do jogo
  aparece na mesma sessão, como fazem Heroic e Lutris. Duas ressalvas: o POG ainda não tem
  navegação por controle (seria um requisito novo; no Deck o touchpad direito vira mouse via Steam
  Input), e o jogo deve ser configurado para tela cheia. **Recomendação:** primeiro no modo área de
  trabalho, para provar que abre; depois jogo não-Steam.
- **Receita de execução no Deck:** nenhuma (placa única, sem Optimus). O `maquina.toml` fica vazio
  ou com a receita `padrao` dos exemplos. Os saves ficam onde o jogo guarda, no home.

### Em aberto (decisão do dono)

Compilar o pacote Linux em `ubuntu-22.04`; navegação por controle no POG; e, só depois de o amigo
rodar o comando, decidir se vale embrulhar o POG em AppImage.
