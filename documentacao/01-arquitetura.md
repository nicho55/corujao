# Arquitetura — rodada 1

**Estado:** rascunho do arquiteto, 22/10/2026. Responde às seis perguntas do pacote de trabalho,
na ordem de importância. Decisões levam `D-NN`; hipóteses que dependem de medição ainda não
feita levam `H-NN` e estão reunidas na seção 9. O que foi medido hoje está na seção 10.

**Premissas combinadas com a gerente enquanto o dono não decide** (mudam a v1, não a arquitetura):
slot de jogador único compartilhado + espectador na v1, contrato já com N slots; licença
**Apache-2.0** para launcher e biblioteca; cada amigo **compila o próprio BlueWake** a partir do
disco dele; no celular a v1 é biblioteca, amigos e convite, sem rodar jogo.

**Provado hoje, nesta máquina (seção 10):** o BlueWake muda ~0,4 MiB de MEM1 por quadro no mar, então
o snapshot incremental cabe num quadro; e duas execuções headless com as mesmas entradas chegam ao
**mesmo estado byte a byte** em todos os chunks após 10 e 600 quadros. O que falta provar é o mesmo
com janela (H-03) e entre duas máquinas (H-11).

---

## 0. Resumo das decisões

| id | decisão | uma linha de motivo |
|---|---|---|
| D-01 | Rollback vive **dentro do processo do jogo**, numa biblioteca Rust com ABI em C que o jogo liga; o launcher nunca carrega o jogo no próprio processo | salvar/restaurar 50 MiB por quadro não atravessa processo; e um jogo GPLv3 dentro do launcher contaminaria a licença |
| D-02 | O contrato do jogo são **quatro callbacks** (salvar, restaurar, liberar, avançar) + um descritor (identidade, quadros/s, tamanho da entrada, slots) + um checksum | é o modelo do GGPO, provado há 15 anos; qualquer coisa a mais é do jogo, não do contrato |
| D-03 | Quadro de netplay = **quadro de jogo** (30 Hz no Wind Waker), fechado no "ponto limpo" que o BlueWake já define (retorno de `GXSetDrawDone`) | o BlueWake já sabe onde o estado é consistente; não inventar outro ponto |
| D-04 | Snapshot **incremental por página de 4 KiB**, em memória, sem compressão; o `.bwstate` em gzip fica só para save manual | medido: cópia integral custa 6,5 ms/quadro, o jogo muda ~0,4 MiB/quadro |
| D-05 | Motor de rollback: **ggrs** (Rust, MIT/Apache-2.0), não escrever o nosso | transport-agnóstico, tem SyncTest para provar determinismo sem rede, e tem lockstep (`max_prediction_window = 0`) como plano B |
| D-06 | Netplay no Wind Waker começa em **modo delay + rollback curto** (atraso de 2 quadros, previsão de até 2), com lockstep como alternativa configurável | a thread do jogo está a 91 % no mar: não há CPU para ressimular 7 quadros |
| D-07 | Camada P2P: **iroh** (QUIC, hole punching, relay como fallback); relays públicos da n0 na fase 2, **relay próprio** num VPS que o dono já tem na fase 4 | é a única pilha que resolve achar, furar NAT e cair para relay com um binário só; a infraestrutura mínima é inevitável e está nomeada |
| D-08 | Flutter e Rust conversam por **flutter_rust_bridge v2**; o Rust é um crate único (`nucleo`) consumido pelo launcher (via bridge) e pelo jogo (via ABI em C) | um código de rede e rollback, dois consumidores; o bridge cobre os cinco alvos |
| D-09 | Launcher ↔ jogo em execução: **argumentos de linha de comando na entrada e a própria sala P2P como canal de status**; sem IPC local | funciona igual no desktop (processo filho) e no celular (app separado por Intent/URL); um protocolo só |
| D-10 | Biblioteca e catálogo em **TOML, um arquivo por jogo**, escritos atomicamente; catálogo é um repositório git público lido por HTTPS | sem servidor, editável à mão, sincronizável; o `apps.json` único do Quiver precisou de backup e recuperação porque corrompia |
| D-11 | **Compilar a própria recomp é funcionalidade de primeira classe** do launcher (assistente de compilação), não um caso especial | código traduzido do disco não pode ser distribuído; sem isso, nenhum amigo joga |
| D-12 | A skill `scaffold-flutter-rust` **não é a base** deste projeto | ela gera um SaaS (axum + SeaORM + Postgres, Flutter web + Android); aqui o Rust roda dentro do app, sem servidor, em cinco alvos nativos |
| D-13 | **Uma raiz de dados só** (`~/jogos/corujao/`, trocável por variável): jogos, capas, receitas e registros numa pasta que se copia inteira | pedido do dono ("tudo numa pastinha"); o Quiver espalha por `apps.json`, `Apps/`, `Cache/` e pastas do sistema |
| D-14 | **Receitas de execução por máquina são dados** (`maquina.toml`), separadas do que é do jogo (`jogo.toml`), e viajam como texto no catálogo (RF-01) | o offload da Nvidia existe porque ESTE laptop é Optimus; cravar no código acerta uma máquina e erra as outras. Detecção automática e erro apontando para a receita ficam para depois |

---

## 1. O contrato de netplay

### 1.1 O que rollback exige, e o que o BlueWake já tem

Rollback é: a cada quadro, salvar o estado; quando a entrada do outro chega diferente da prevista,
restaurar o estado daquele quadro e avançar de novo até o presente. Exige quatro coisas (briefing,
seção 5). Confrontei cada uma com o código do BlueWake (`runtime/host/src/main.c`,
`save_state.{c,h}`) e com medições feitas hoje nesta máquina (seção 10):

| exigência | o que o BlueWake tem hoje | o que falta |
|---|---|---|
| **salvar em poucos ms** | `host_save_state`: contêiner de chunks (`CPU` 3 KiB, `MEM1` 32 MiB, `ARAM` 16 MiB, `ALIASES` 1,5 MiB, `GX` 742 KiB, `HOSTVARS` 68 KiB, `DSPHLE` 4 KiB, `VICLOCK`, `PE`, `LOOPVARS`), **em gzip para arquivo**. Medido: o gzip -1 do estado real leva 0,88 s; o jogo carregou o estado em 210 ms (`ms=210` no log) | um caminho **em memória**: os mesmos chunks, sem compressão, com MEM1/ARAM/ALIASES **incrementais** (só páginas sujas). Medido: cópia integral de 48 MiB = 6,5 ms (7,8 GB/s); o jogo muda **~0,4 MiB/quadro** no mar (114 páginas de 4 KiB, mediana). Ou seja: integral não cabe num quadro de 33 ms com a thread a 91 %; incremental cabe com folga |
| **restaurar igualmente rápido** | `host_load_state` restaura campo a campo (`fields=151 missing=0 mismatched=0`) e **reconstrói** caches do renderer, EFB copies, histórico do Smooth Motion e fila de áudio | o mesmo caminho em memória; restaurar incremental = copiar de volta as páginas que mudaram desde o snapshot alvo (a pilha de snapshots sabe quais) |
| **avançar N quadros com entradas dadas** | `BLUEWAKE_PAD_SCRIPT`, `BLUEWAKE_PAD_BUTTONS/STICK_*` (pad virtual no backend headless), `bluewake_pad_merge` (entrada ao vivo **mesclada** no SI a cada retrace), `BLUEWAKE_MAX_RETRACES` | uma função "avance **um quadro de jogo** com estas amostras de pad, pare no próximo ponto limpo"; e silenciar áudio/apresentação enquanto ressimula |
| **mesmo estado byte a byte nas duas máquinas** | domínio de ciclos **independente do relógio de parede** (`Services.md`: "the shared cycle domain orders device work independently of host wall-clock speed"); `[guest-checkpoint]` já calcula XXH3 de CPU, MEM1, ARAM e aliases; o PE finish é levantado no **pc** de `GXSetDrawDone` (determinístico, não quando a GPU termina). **Medido hoje:** duas execuções headless, sem freio de ritmo, com o mesmo script de 5 pressionamentos a partir do `quick-2766`, salvaram estados **idênticos em todos os chunks** (CPU, MEM1, ARAM, ALIASES, VICLOCK, HOSTVARS, PE, LOOPVARS, DSPHLE) após 10 e após 600 quadros, com 13 checkpoints intermediários iguais | o mesmo teste **com janela** (H-03: o chunk GX nem existe no headless, e há EFB peeks) e **entre duas máquinas** com o mesmo binário (H-11); e um gate de identidade: mesma tradução (digest), mesmos mods, mesmas opções de host |

Conclusão: o BlueWake tem **mais da metade**: contêiner de estado, ponto limpo, checksum, pad virtual,
e um relógio de guest que não olha o relógio da parede. Falta o caminho rápido e o laço "um quadro
por chamada". É trabalho de host (C, em `runtime/host`), não de tradução.

### 1.2 Forma do contrato: biblioteca dentro do processo do jogo (D-01)

**Decisão.** O launcher publica uma biblioteca (`libnetplay`, Rust compilado como `cdylib`/`staticlib`
com ABI em C, Apache-2.0) e um cabeçalho. O jogo **liga** essa biblioteca, registra quatro callbacks e
passa a chamá-la uma vez por quadro. Rede, previsão, rollback, espectadores e detecção de dessincronia
ficam na biblioteca; salvar, restaurar e avançar ficam no jogo, que é quem sabe fazê-los.

**Alternativas descartadas.**

- *Launcher carrega o jogo como `.so` com símbolos conhecidos e dirige o laço.* O host do BlueWake
  (291 MB) é dono da janela, do SDL, do Dawn, de três threads e do laço de turnos; transformá-lo em
  biblioteca passiva é reescrever o host. Além disso, GPLv3 carregado no processo do launcher força o
  launcher a ser GPL — a licença ainda está com o dono, e este desenho não deve depender dela.
- *Jogo em processo separado, rollback no launcher, protocolo entre os dois.* Salvar/restaurar são
  dezenas de MiB **por quadro**; nenhuma IPC faz isso em tempo. O rollback precisa estar onde a memória
  está.
- *Netplay escrito dentro de cada jogo, sem biblioteca comum.* É o que o Slippi fez para o Melee
  (referência de que rollback em GameCube emulado é possível, com snapshot específico do jogo).
  Funciona para um jogo; não escala para uma coleção, e repete a parte difícil (rede) a cada recomp.

### 1.3 A interface mínima

Nomes provisórios (seguem o nome do projeto quando houver). Identificadores em português, como o
resto do projeto.

**Descritor que o jogo entrega ao criar a sessão**

```c
typedef struct NetplayDescritor {
    uint32_t abi;                 // NETPLAY_ABI = 1; a biblioteca recusa outro valor
    char     jogo[32];            // identidade: "GZLE01" + digest da tradução (ex.: "54f54434")
    uint32_t mods;                // máscara dos mods compilados (o BlueWake já tem; precisa bater)
    uint32_t versao_estado;       // muda quando o layout do snapshot muda; precisa bater
    uint32_t quadros_por_segundo; // 30 no Wind Waker
    uint32_t tamanho_entrada;     // bytes por slot por quadro (8 para um PAD de GameCube)
    uint32_t slots;               // quantos jogadores o jogo aceita (1 no Wind Waker)
} NetplayDescritor;
```

**Callbacks que o jogo implementa** (todos chamados na thread do jogo, nunca em outra):

| callback | contrato |
|---|---|
| `salvar(ctx, quadro, NetplayEstado** saida, uint64_t* checksum)` | Produz um snapshot **em memória** do estado completo do quadro corrente e, opcionalmente, um checksum. Alvo: < 1 ms no caso comum. Buffers são do jogo (pool), por isso existe `liberar` |
| `restaurar(ctx, const NetplayEstado* estado)` | Volta a máquina ao snapshot. Caches de renderer/áudio podem ser reconstruídos; nada observável pelo guest pode sobrar |
| `liberar(ctx, NetplayEstado* estado)` | A biblioteca não precisa mais desse snapshot |
| `avancar(ctx, const uint8_t* entradas, uint32_t n_slots, uint32_t flags)` | Avança **exatamente um quadro de jogo** com as amostras de pad dadas (uma por slot), parando no próximo ponto limpo. `flags & NETPLAY_RESSIMULACAO` = não apresentar vídeo nem empurrar áudio |

**O que o jogo chama**

| função | contrato |
|---|---|
| `netplay_criar(descritor, callbacks, ticket) → sessao` | `ticket` é a string que o launcher passou em `--netplay-sessao` (1.3.1). A biblioteca abre a conexão P2P, faz o handshake de identidade (descritor igual nos dois lados ou recusa) e sincroniza |
| `netplay_entrada_local(sessao, slot, bytes)` | A amostra local deste quadro |
| `netplay_quadro(sessao) → estado` | Uma vez por quadro de jogo. Internamente chama `salvar`/`restaurar`/`avancar` quantas vezes for preciso (é a lista de `GgrsRequest` do ggrs sendo cumprida). Devolve `OK`, `AGUARDANDO` (lockstep ou previsão esgotada: o jogo repete o quadro anterior na tela), `DESCONECTADO`, `DESSINCRONIZADO` |
| `netplay_evento(sessao, NetplayEvento* saida) → bool` | Fila de eventos: par conectado, sincronizado, latência, desconexão, dessincronia detectada, espectador entrou |
| `netplay_destruir(sessao)` | |

**Regras que o cabeçalho escreve por extenso**

1. A **entrada é o único canal não determinístico**. Mouse-câmera, botão de pulo, sprint, opções em
   tempo real, mods aplicados por retrace (`host_mods_reapply`, `bluewake_game_options_retrace`,
   `bluewake_mouse_camera_retrace`, `bluewake_jump_button_retrace`, `bluewake_sprint_retrace`, portas
   rápidas, fast load) ou viram bytes da entrada, ou ficam travados com valor igual nos dois lados
   durante a sessão.
2. O jogo **não lê relógio de parede** para decidir nada que entre no estado.
3. `avancar` com `NETPLAY_RESSIMULACAO` não pode ter efeito fora da máquina emulada.
4. O checksum, quando fornecido, cobre tudo que `restaurar` restaura. No BlueWake é o
   `[guest-checkpoint]` de hoje (XXH3 de CPU + MEM1 + ARAM + aliases). Calcular a cada N quadros,
   não a cada quadro: um XXH3 de 48 MiB custa alguns ms (H-08).

#### 1.3.1 Como o launcher fala com o jogo (D-09)

Dois argumentos de linha de comando, iguais em todas as plataformas:

- `jogo --netplay-info` imprime o descritor em JSON e sai. É como o launcher descobre, **sem carregar
  nada no próprio processo**, que o jogo cumpre o contrato e com que identidade.
- `jogo --netplay-sessao <ticket>` inicia já conectando. O ticket (string base32) traz: chave da sala,
  slot deste jogador (ou "espectador"), `EndpointId` dos pares, relay a usar, modo (rollback/lockstep),
  atraso de entrada e janela de previsão.

No desktop o launcher é o processo pai e passa `argv`; no Android é um Intent com `extras`; no iOS é
uma URL `nomedoapp://sessao/<ticket>`. Não há IPC local: o jogo, pela biblioteca, **entra na sala**
P2P e publica status lá (conectado, sincronizando, latência, caiu). O launcher — nesta ou noutra
máquina — lê da sala. No desktop o launcher ainda vigia o processo filho e o log, como o
`recomps.sh` faz hoje.

#### 1.3.2 O que "estado completo" quer dizer (não é só memória)

O contrato define `salvar` como "tudo que influencia o quadro seguinte". No BlueWake isso é o que
`host_load_state` restaura hoje (o log diz `fields=151 missing=0 mismatched=0`), e serve de lista de
conferência para qualquer recomp que venha a cumprir o contrato:

| chunk | tamanho medido | o que é | no snapshot incremental |
|---|---|---|---|
| `CPU` | 3,4 KiB | prefixo de `CPUState`: GPR/FPR/paired-single, CR/LR/CTR/XER, pc, MSR, exceção pendente, reserva (`lwarx`), contagem de ciclos, orçamento e deadlines | cópia integral, todo quadro |
| `MEM1` | 32 MiB | RAM do guest (24 MiB de varejo + 8 MiB da extensão ligada) | páginas sujas |
| `ARAM` | 16 MiB | memória auxiliar de áudio, escrita só por DMA modelado pelo host | páginas sujas (ou marca no modelo de DMA) |
| `ALIASES` | 1,5 MiB | armazenamento das seções ligadas das RELs (dados e BSS fora da MEM1) | páginas sujas |
| `GX` | 742 KiB (só com renderer) | modelo de registradores CP/XF/BP, paletas de TMEM, **bytes do comando GX inacabado** e registradores do sink; sem isso o quadro depois do restore decodifica vértices com formato errado | cópia integral (0,1 ms) |
| `HOSTVARS` | 68 KiB | ~80 variáveis nomeadas do host (abaixo) | cópia integral |
| `DSPHLE` | 4,4 KiB | `DSPHLE::DoState` do Dolphin (mixer de áudio) | cópia integral |
| `VICLOCK` | 1 KiB | relógio de vídeo (retraces) | cópia integral |
| `PE`, `LOOPVARS` | 8 B, 91 B | interrupções do Pixel Engine; variáveis do laço do host | cópia integral |

As variáveis de `HOSTVARS`, por grupo (tabela `HS_FIELD` em `main.c`):

- **registro de RELs**: aliases, contagem, faixas cruas, slots, base do módulo 1;
- **DSP**: controle, mails nos dois sentidos, handshake de boot e de tarefa, palavras de áudio restantes;
- **dispositivos**: DMA de áudio, interrupções (`g_interrupts`), **SI e pad virtual**, DMA de ARAM, IPL/SRAM (registradores, cursor, escritas), DI (status, tampa, comando, DMA, controle, config), fila de conclusões de DVD;
- **tempo**: decrementador do guest (valor, validade, expiração, resto de ciclos, pendência), `cycle_domain` (ciclos absolutos e de dispatch), cursores de ciclo de VI, áudio e DSP, adaptador do DSP (pendência, fatia, decorrido, contagem de DMA), **contagem de retraces**, timebase do retrace anterior;
- **integração**: sombras de contexto, digest de entrega, commits do draw-done assíncrono;
- **automação de rota** (marcos de título, seleção de arquivo, cena de nome, cartão, abertura, Outset; pulsos de pad; rota de save; `live_takeover`) — é estado do harness do upstream, não do jogo. **No contrato isso fica fora**: em sessão de netplay a automação está desligada e esses campos são constantes.

Dois pontos que o contrato escreve por extenso por causa desta lista: (1) a **reserva de memória**
(`lwarx`/`stwcx.`) e a **exceção pendente** estão no `CPU` e precisam voltar com ele — por isso o
ponto limpo exige "sem exceção pendente"; (2) o **lote inacabado do gather pipe** está no chunk `GX`:
salvar num ponto que não seja o draw-done obriga a carregar esse resto, como o BlueWake já faz.

### 1.4 O BlueWake cumpre o contrato? O que falta nele

**Cumpre, com trabalho de host.** Lista do que precisa existir num **fork** do BlueWake (a cópia de
trabalho em `~/projetos/RECOMPS/bluewake` não é tocada; o fork vive em outro diretório, com os patches
como `patches/`, para oferecer ao upstream depois):

1. **Snapshot em memória e incremental.** Reusar a lista de chunks de `host_save_state`, escrevendo em
   um buffer em vez do gzip. MEM1 (32 MiB), ARAM (16 MiB) e ALIASES (1,5 MiB) só copiam páginas sujas:
   - Linux/Android: bits *soft-dirty* do `/proc/self/pagemap` + `clear_refs`, **com
     `madvise(MADV_NOHUGEPAGE)` na MEM1** — hoje o THP está em `always` nesta máquina e uma escrita
     suja uma página de 2 MiB inteira (foi o que a medição mostrou: 16 MiB "sujos" por quadro,
     0,4 MiB de fato alterados). Alternativa sem dependência de kernel: `mprotect` + handler de
     SIGSEGV marcando a página e liberando-a até o próximo snapshot (o Dolphin usa isso para a cache de
     texturas); custo = uma falha por página por quadro (~114/quadro → décimos de ms).
   - Windows: `VirtualAlloc(MEM_WRITE_WATCH)` + `GetWriteWatch` — feito para isso.
   - macOS/iOS: `mprotect` + exceção Mach/SIGBUS (H-07: confirmar que o iOS deixa o handler rodar).
   - ARAM só é escrita por DMA que o **host** modela: marcar sujo no próprio modelo de DMA é mais
     barato que vigiar páginas.
   - Clear_refs por processo custou 0,7 ms/quadro medido de fora; dentro do processo e restrito a um
     intervalo, menos (H-01).
2. **Laço "um quadro por chamada".** O host já roda em turnos e já detecta o ponto limpo
   (`host_state_try_save`: `pc == 0x80322BC8` sem exceção pendente, sem prólogo de REL, sem callback
   de cartão, sem troca de cena). Falta uma função que: latcha a amostra de pad dos slots no SI, corre
   turnos até o próximo ponto limpo, devolve. Hoje a entrada ao vivo é mesclada no SI **a cada
   retrace**; no modo netplay a mescla é desligada e a amostra vem só do callback (H-04: confirmar que o
   jogo lê o PAD uma vez por quadro de jogo, como a maioria dos jogos de GameCube; se ler por retrace,
   a mesma amostra serve aos dois retraces do quadro).
3. **Ressimulação silenciosa.** Com `NETPLAY_RESSIMULACAO`, não empurrar áudio ao dispositivo (a fila é
   reconstruída, não salva — já é assim) e descartar a apresentação. O gather pipe do GX pode seguir
   sendo escrito (é 2,5 % do custo) ou ser descartado; a lógica do jogo é o custo de verdade.
4. **Gates de determinismo com janela.** Headless está provado hoje (execução contra execução, byte a
   byte, 600 quadros). A dúvida levantada pela gerente — o `behaviour-timing` move eventos de
   dispositivo para o início do bloco seguinte e "os checkpoints diferem de uma build sem ele" — é
   diferença **entre builds**, não entre execuções da mesma build: o deslocamento é função do bloco,
   não do relógio, e a medição confirma. O gate de identidade (mesmo binário nos dois lados) absorve
   isso. O que segue aberto é a execução **com janela**, que o upstream diz "variar de execução para
   execução". Suspeitos, por leitura do código:
   - **EFB peeks** (`GXPeekZ`/`GXPeekARGB`) respondidos pelo snapshot **assíncrono** de profundidade do
     Aurora: o sol (glare/lens flare) e a Picto Box leem o resultado. Em netplay, isso tem de ser
     determinístico nas duas máquinas — e GPUs diferentes não produzem a mesma profundidade. Proposta:
     em sessão de netplay, peek devolve valor fixo (como o headless faz, "keeps 0"). Custo honesto:
     sem brilho do sol e a Picto Box atribui foto à primeira pessoa registrada. Alternativa cara:
     reimplementar a visibilidade do sol na CPU.
   - Recursos de host que escrevem na MEM1 por retrace (lista do item 1 das regras). Travar ou virar
     entrada.
   - Confirmar com `[guest-checkpoint]` em modo janela, dois processos lado a lado (H-03).
5. **Identidade.** `--netplay-info` com digest da tradução, máscara de mods e opções travadas. O
   BlueWake já recusa estado "feito por outra tradução" e avisa "saved with mods 0x%X"; é só expor.

**Ordem de grandeza do esforço:** itens 1–3 são algumas semanas de C no host, sem tocar no gerado. Item 4
é investigação com ferramenta pronta. Nada disso exige recompilar a tradução (ciclo de ~3 min do app,
não 1h45).

### 1.5 O que não fecha como pedido, e o que fecha

- **Rollback "de jogo de luta" (7 quadros a 60 Hz) não cabe nesta máquina com este jogo.** Um quadro do
  Wind Waker no mar custa ~30 ms de CPU da thread do jogo (91 % de 33 ms, medição de 22/10). Ressimular
  2 quadros = +60 ms = engasgo visível. O que fecha (D-06): **atraso de entrada de 2 quadros (66 ms) com
  previsão de até 2**. Entre duas casas no Brasil (RTT típico 20–80 ms) isso quase elimina rollbacks;
  quando acontece, custa um engasgo, não dessincronia. Cenas leves (99,7 % de velocidade) têm folga
  maior. E `max_prediction_window = 0` dá **lockstep** puro para quem preferir estabilidade a
  resposta. Hardware melhor ou o upstream mais rápido mudam os números, não o desenho.
- **O Wind Waker é single-player.** Netplay nele significa um slot compartilhado (quem segura o
  controle), passagem de controle e espectadores — não versus. O contrato já nasce com N slots porque
  a próxima recomp pode ser multiplayer; o dono decide se isso basta para a v1.
- **Espectador em celular não é por entradas.** Espectador do ggrs recebe entradas e **simula o jogo**;
  o celular que não roda o jogo não pode assistir assim. No celular, assistir = **vídeo** vindo do PC de
  quem joga (fase 4). Corrijo aqui a minha própria proposta à gerente.
- **Mesma tradução nos dois lados.** Netplay exige digest da tradução, mods e opções iguais. O amigo no
  Windows pode usar a build pronta dos mantenedores (0.7.0) **se** o digest bater com o do dono; os
  patches de host do dono (chassis, gather_pipe, PGO) não mudam o guest, então não entram no gate.

---

## 2. Rust + Flutter, na prática

### 2.1 Divisão e ponte (D-08)

**Um crate Rust, `nucleo`**, com tudo que não é pixel: biblioteca e catálogo, instalação e atualização
(GitHub/GitLab), assistente de compilação, identidade e amigos, sala/convites/presença, transporte P2P,
motor de rollback. Ele é compilado de **dois jeitos**:

- como biblioteca do app Flutter, via **flutter_rust_bridge v2** (gera as ligações Dart; cargokit
  constrói o Rust para cada alvo: `.so` em Linux/Android, `.dll` no Windows, framework no macOS,
  **biblioteca estática no iOS**, onde dylib avulsa não é permitida); Rust `async` vira `Future` em
  Dart e `StreamSink` vira `Stream` — é por aí que presença, progresso de download e status de sessão
  chegam à interface;
- como `libnetplay` (cdylib/staticlib, ABI em C, `cbindgen`) para o jogo ligar (seção 1). É o
  **mesmo** código de rede e rollback: `features` do Cargo tiram biblioteca, catálogo e instalação
  dessa variante.

**Alternativas descartadas.**

- *Servidor HTTP local em Rust + Flutter como cliente* (o padrão da `scaffold-flutter-rust`): no iOS não
  há processo de fundo para o servidor; no Android é um serviço a mais para o sistema matar; e todo
  evento vira polling ou WebSocket consigo mesmo. É forma de SaaS, não de app.
- *`dart:ffi` à mão*: funciona, mas cada struct, callback e stream vira código manual nos dois lados;
  o bridge gera isso e já resolve threads e tipos opacos.
- *Interface em Rust (egui/Dioxus/Tauri)*: a pilha está decidida; não discutida.

### 2.2 Quem é dono do ciclo de vida

O **Flutter é dono do processo** nas cinco plataformas: é ele que o sistema inicia, pausa e mata. No
primeiro quadro o Dart chama `nucleo::iniciar(diretorios)`, que sobe um runtime Tokio (2 workers —
a máquina é modesta), abre a biblioteca local, carrega a identidade e devolve um objeto opaco. Tudo
que o Rust faz de longo (download, build, rede) roda nesse runtime e reporta por `Stream`. Quando o
sistema pausa o app (celular), o Dart avisa e o Rust fecha presença; quando mata, não há limpeza — por
isso toda escrita em disco é atômica (arquivo temporário + rename).

**O jogo nunca é filho do Flutter no celular e sempre é no desktop:**

| | Linux / Windows / macOS | Android | iOS |
|---|---|---|---|
| como o jogo roda | `nucleo` faz `spawn` do executável com o ambiente certo (ex.: offload Nvidia, `taskset`, variáveis por jogo), vigia saída e log, como o `recomps.sh` | é **outro APK** (o do BlueWake); o launcher abre por Intent explícito com o ticket nos `extras`; ao voltar, lê o status pela sala | é **outro app** sideloaded; URL scheme com o ticket; enquanto o jogo está na frente o launcher está suspenso — por isso o jogo precisa da pilha de rede inteira (D-09) |
| onde o Rust roda | dentro do app Flutter (`.so`/`.dll`/framework) | `.so` por ABI (`arm64-v8a`; `x86_64` para emulador), via NDK/cargokit | `staticlib` ligada ao app; sem JIT, sem dlopen de terceiros |
| rede em segundo plano | ilimitada | sockets congelam em Doze; presença só com o app aberto ou com *foreground service* (fase 2 decide) | ~30 s depois de ir para o fundo; presença só com o app aberto. **Sem servidor não há push**: convite só chega com o app aberto — custo honesto do "sem servidor" |
| arquivos | pastas XDG / `%APPDATA%` / `~/Library/Application Support` (crate `directories`) | pasta privada do app; instalar APK exige permissão `REQUEST_INSTALL_PACKAGES` e o instalador do sistema | sandbox do app; **não instala nada** |
| distribuição do launcher | AppImage/zip/`.app` em releases; macOS pede notarização ou "abrir mesmo assim"; Windows mostra SmartScreen sem assinatura | APK em releases (a Play Store não aceita app que instala APKs) | TestFlight (conta de desenvolvedor, US$ 99/ano) ou sideload — mesmo caminho que o BlueWake |

### 2.3 Sobre a `scaffold-flutter-rust` (D-12)

Resposta à pergunta da gerente: **não serve como base**, e o motivo é de forma, não de qualidade. Ela
gera um sistema cliente-servidor (Flutter web + Android falando HTTP com axum sobre Postgres, Docker
para subir o banco). Aqui não há servidor, o Rust roda **dentro** do app, e os alvos são cinco
plataformas nativas — o web nem está na lista. Usar o scaffold obrigaria a manter um backend que não
existe no desenho.

O que **aproveita**: a prática do dono com Flutter (layout responsivo, organização de telas) e, no
lado Rust, a skill `setup-hot-reload` (Subsecond/`dx serve --hot-patch`) para encurtar o ciclo de
compilar o `nucleo` durante o desenvolvimento. Nenhum arquivo gerado pelo scaffold entra no repo.

---

## 3. A camada P2P

### 3.1 O problema concreto

Dois amigos atrás de roteadores domésticos não têm IP alcançável. No Brasil é pior: muita operadora
usa **CGNAT**, e com CGNAT dos dois lados conexão direta IPv4 é frequentemente impossível (IPv6, quando
há, resolve). Qualquer solução precisa de três coisas: (a) um jeito de **achar** o outro, (b) tentar
**furar** o NAT, (c) **cair para um relay** quando não fura. (a) e (c) são infraestrutura, por menor
que seja. "Sem servidor próprio" vira "sem servidor **de aplicação**": não há conta, não há banco, não
há estado nosso em lugar nenhum — mas há relays e um serviço de descoberta rodando em algum lugar.

### 3.2 iroh (D-07)

**Decisão:** iroh, biblioteca Rust de rede P2P sobre QUIC.

- **Endereço é uma chave.** Cada instalação gera um par ed25519; o `EndpointId` (chave pública) é o
  endereço. Amigo = troca de chaves, uma vez, por **ticket** (string ou QR) mandado por qualquer canal
  (WhatsApp, Discord). Não há cadastro.
- **Descoberta**: o par publica onde está (relay + endereços diretos) num serviço DNS da n0 ou na
  **DHT Mainline** (BEP 44, a rede do BitTorrent, que ninguém controla), e na rede local por mDNS.
- **Hole punching dentro da conexão QUIC**, com o relay ajudando os dois a se verem; quando fura, o
  tráfego vai direto; quando não fura, **o relay encaminha** e a conexão continua funcionando, mais
  lenta. A troca direto↔relay é transparente para quem usa a conexão.
- **Relays**: a n0 opera relays públicos **com limite de taxa** (terceiro, de graça, sem garantia — fase
  2 vive disso e o documento diz isso). O `iroh-relay` é um binário só, e "o iroh funciona com um único
  relay": na fase 4 ele roda num VPS que o dono já tem (seção "Máquinas" do CLAUDE.md), e aí o único
  terceiro que sobra é a DHT, que não é de ninguém. Um relay encaminhando 2 jogadores a ~10 KB/s cada
  cabe em qualquer VPS pequeno; vídeo de espectador (fase 4) é outra conversa e é medido antes.
- **Plataformas**: Rust nativo nos cinco alvos (há bindings Swift/Kotlin, mas nós chamamos do Rust).
- Para o ggrs precisamos de **datagramas não confiáveis** (ele faz a própria confiabilidade). QUIC tem
  datagramas (RFC 9221) e o quinn, base do iroh, implementa; **H-05: confirmar que o iroh expõe
  `send_datagram`/`read_datagram` na conexão e qual MTU**. Se não expuser, um stream unidirecional por
  pacote é o fallback, com custo de latência pequeno.
- Licença MIT/Apache-2.0 (H-06: confirmar no repositório antes da fase 2).

**Alternativas descartadas.**

- *WebRTC (matchbox/str0m/webrtc-rs).* Resolve (b), mas exige servidor de sinalização **nosso** para
  (a) e TURN para (c) — TURN é justamente a parte cara de operar. Faz sentido para navegador, que não
  está na lista.
- *libp2p.* Faz tudo, mas é uma pilha grande e genérica; o iroh é o subconjunto que precisamos, já
  montado.
- *UDP hole punching artesanal + rendezvous próprio.* Fura ~70–80 % dos NATs domésticos e **zero**
  CGNAT duplo; teríamos de escrever o relay de qualquer jeito. É reinventar o iroh pior.
- *Steam Networking / Tailscale / Hamachi.* Dependem de conta e serviço de terceiro — é o que o
  briefing proíbe.

### 3.3 O que acontece quando a conexão direta falha

1. O ticket traz o `EndpointId` do par e o relay "de casa" dele. A biblioteca conecta ao relay
   (sempre funciona: é TCP/HTTPS de saída), e por ele os dois trocam endereços candidatos.
2. Tentam furar (IPv6 direto, IPv4 por UDP, pares de endereços). Se funciona em algum, o QUIC migra
   para o caminho direto; latência cai para o RTT real entre as casas.
3. Se nada fura (CGNAT duplo, firewall corporativo), **a sessão segue pelo relay**. Latência =
   casa→relay→casa; com relay da n0 fora do país isso pode ser 150–250 ms. O launcher **mostra** que
   está em relay e a latência medida, e sugere lockstep ou mais atraso de entrada. Com o relay próprio
   em São Paulo (fase 4), cai para dezenas de ms.
4. Se o relay cair, a conexão QUIC tenta os caminhos diretos que já conhece; se nenhum houver, cai, o
   ggrs reporta desconexão e o jogo entra em pausa de rede.

### 3.4 Presença, convites, sala

- **Presença**: ao abrir, o launcher conecta aos amigos que estiverem alcançáveis e troca "estou
  aqui, jogando X". Sem servidor, "offline" significa só "não alcancei". Nada é guardado em lugar
  nenhum.
- **Sala**: quem convida gera a chave da sala e o ticket; os convidados entram pela conexão direta com
  quem convidou (um pequeno protocolo nosso sobre streams QUIC: entrar, slot, pronto, iniciar, status).
  Antes de iniciar, cada launcher roda `jogo --netplay-info` e a sala **compara descritores**: tradução,
  mods, versão de estado, ABI. Divergiu, não inicia, e diz o que divergiu.
- **Chaves de sessão**: para o jogo não precisar da chave do launcher, cada launcher gera um par
  efêmero por sessão, publica a chave pública na sala, e é ela que vai no ticket do jogo.

---

## 4. Rollback

### 4.1 ggrs (D-05)

**Decisão: usar o ggrs** (v0.13.0, set/2026, MIT/Apache-2.0), reimplementação em Rust do GGPO, não escrever
o nosso. Fatos verificados hoje na documentação:

- Três sessões: `P2PSession` (jogadores), `SpectatorSession` (recebe entradas confirmadas e simula),
  `SyncTestSession` (**sem rede**: volta `check_distance` quadros a cada passo e compara checksums —
  é o detector de não determinismo que a fase 0 precisa).
- Modelo de **requisições, não callbacks**: a cada quadro devolve uma lista `SaveGameState`,
  `LoadGameState`, `AdvanceFrame` que quem chama cumpre em ordem. Encaixa direto nos quatro callbacks do
  contrato — a `libnetplay` é quem cumpre a lista chamando o jogo.
- `NonBlockingSocket`: duas funções, `send_to` e `receive_all_messages`, **não confiável e sem ordem**;
  "GGRS provides its own reliability and ordering on top". É o que ligamos aos datagramas do iroh.
- Parâmetros: `num_players` (≥ 1), `input_delay`, `max_prediction_window` (padrão 8; **0 = lockstep**),
  `fps`, `sparse_saving` (salva só o quadro confirmado; menos saves, rollbacks mais longos),
  `desync_detection` (compara checksums entre pares e emite evento), `disconnect_timeout`.
- Tipo de entrada precisa ser `Copy + PartialEq + Pod`; usamos um `[u8; 16]` fixo e o descritor diz
  quantos bytes valem.

**Alternativas descartadas.** *GGPO original* (C++, voltado a Windows, sem SyncTest em Rust, mais uma
ABI para manter). *backroll-rs* (menos mantido). *Escrever o nosso* — rollback é onde os bugs são
sutis e caros; a vantagem do ggrs é justamente ter sido depurado por outros.

### 4.2 Como se liga ao contrato

| ggrs | contrato (seção 1) |
|---|---|
| `SaveGameState{frame}` | `salvar(ctx, quadro, &estado, &checksum)`; o `State` do ggrs é um handle para o snapshot do jogo |
| `LoadGameState{cell}` | `restaurar(ctx, estado)` |
| `AdvanceFrame{inputs}` | `avancar(ctx, entradas, slots, flags)`; `flags` tem `RESSIMULACAO` quando o quadro já havia sido apresentado |
| célula descartada | `liberar` |
| `fps` | `quadros_por_segundo` do descritor |
| `Config::Input` | `[u8; 16]`, `tamanho_entrada` bytes válidos |
| `desync_detection` | usa o `checksum` devolvido por `salvar`; intervalo configurado para não calcular XXH3 de 48 MiB todo quadro |

### 4.3 Parâmetros para o nosso caso (D-06)

Wind Waker nesta máquina: `fps = 30`, `input_delay = 2` (66 ms), `max_prediction_window = 2`,
`sparse_saving = false` (salvar é barato quando incremental; o caro é ressimular), `desync_detection`
ligado a cada 30 quadros. Lockstep (`max_prediction_window = 0`) fica na interface como "modo
estável". Esses números são **ponto de partida**; a fase 0 mede o custo real de `avancar` em
ressimulação (H-08) e a fase 3 ajusta com gente jogando.

### 4.4 Ordem de prova

Nenhuma linha de rede antes de o `SyncTestSession` rodar o BlueWake headless a partir do
`quick-2766.bwstate` por 10 minutos com `check_distance = 2` e zero divergência. Depois o mesmo com
janela. Só então `P2PSession` em duas instâncias **na mesma máquina** (socket UDP local), e só então o
iroh. Cada degrau isola uma classe de erro.

---

## 5. Biblioteca e catálogo

### 5.1 Inventário do que o Quiver faz (lido nos `*ViewModel.cs` e no README)

Biblioteca em grade com capas, busca por nome/tag/repositório/pasta, **tags livres**, filtros de
exibição salvos (incluir/excluir tags), ordenação, "continuar jogando"; **entradas manuais** (pasta
sem repositório; "promover" para gerenciada ao preencher repositório); editor de entrada (nome,
projeto, nome de exibição, pasta, repositório GitHub/GitLab, filtro de asset da release, ícone, tags,
pasta e fontes de mods); catálogo remoto com filtros (tipo de projeto, plataforma, console), página
com README, releases, avaliações; **instalar e atualizar por releases** com verificação de release
("verificada / não verificada / bloqueada"), SHA-256, tokens de API, revisão de atualizações
(atualizar/pular, em lote); mods (Thunderstore, GameBanana) com dependências, NSFW oculto, layout por
mod; mapeamento de controles e navegação por gamepad em toda a interface; modo quiosque com PIN;
atualização do próprio launcher; Android instala **APKs** de releases e nada mais
(`PlatformCapabilities`: sem pasta de instalação, sem Wine, sem mods, sem SDL no celular).

Do `recomps.sh` o que entra: verificar ambiente com o comando que resolve cada falta, identificar
disco pelo cabeçalho (ID, revisão, assinatura), converter imagem comprimida, compilar com
acompanhamento, jogar com o perfil certo (offload Nvidia) e mostrar o fim do log se o jogo morrer.

### 5.2 Onde ficam os metadados (D-10)

```
<dados do app>/
  biblioteca/<id>/jogo.toml      # uma entrada; escrita atômica
  biblioteca/<id>/capa.png
  catalogo/                      # cópia local do catálogo (git ou tarball)
  instalado/<id>/                # binários instalados por release (desktop)
  identidade.toml                # chave privada (permissão 0600), apelido
  amigos.toml                    # EndpointId + apelido + chave de relay de cada amigo
  configuracao.toml
```

`jogo.toml` carrega os campos do Quiver que fazem sentido (nome, projeto, nome de exibição, tags,
repositório e fonte, filtro de asset, ícone, pasta, mods) **mais** os nossos: `console`, `disco`
(`id`, `revisao`), `executaveis` (padrões por plataforma, como o `JOGAR` dos `.conf`), `lancamento`
(variáveis, prefixos como `taskset`/offload), `compilacao` (receita: imagem Docker, script, duração
estimada, saída esperada), `netplay` (`abi`, descritor lido do `--netplay-info` na última execução).

**Por que não um `apps.json` único** (Quiver): corrompe inteiro quando o app morre no meio da escrita —
tanto que o Quiver construiu snapshots e recuperação. **Por que não SQLite**: não é editável à mão,
não entra em git, e a biblioteca de uma pessoa tem dezenas de entradas, não milhões.

### 5.3 Catálogo

Um repositório git público (`catalogo/<id>.toml` + `indice.json` gerado por CI) servido por GitHub
Pages/raw. O app baixa o índice por HTTPS e os `.toml` sob demanda. Nenhum servidor nosso; qualquer um
abre PR; o dono mantém. Verificação de release como no Quiver: o catálogo registra o SHA-256 da release
verificada; release nova é "não verificada" até alguém marcar. **O catálogo também guarda receitas
de execução** (RF-01): o binário de uma recomp não pode circular, mas a receita que o faz abrir numa
máquina Optimus, num desktop de placa única ou no Windows é texto e pode — por isso a entrada do
catálogo traz receitas por perfil de máquina, e o launcher propõe a que casa com o perfil detectado.

### 5.4 Como o usuário adiciona o que compilou

Três portas, as três no desktop:

1. **"Adicionar pasta/executável"** (entrada manual): aponta o AppImage ou a pasta; o launcher lê o
   cabeçalho, pergunta nome, tags e perfil de lançamento. É como o BlueWake de hoje entra.
2. **Assistente de compilação (D-11)**: a entrada do catálogo traz a receita; o launcher verifica o
   ambiente (Docker, clang, AVX2, Vulkan, 30 GB livres — a lista do `recomps.sh`), pede a imagem de
   disco, confere ID/revisão, converte se preciso, roda a receita num terminal/painel com log ao vivo e
   estimativa de tempo (a heurística do `painel-build.sh`: ritmo dos últimos minutos, porque o fim
   desacelera), e **registra a saída** na biblioteca. Primeira versão **embrulha** os scripts que já
   existem (`montar-bluewake.sh`, `converter.sh`); a reescrita em Rust vem depois, se valer.
3. **Importar do catálogo** uma entrada manual já descrita (capa, tags, receita), para quem já tem o
   binário.

### 5.5 Instalação a partir de releases, por plataforma

| plataforma | o que o launcher faz | limites honestos |
|---|---|---|
| Linux | GitHub/GitLab API → escolhe asset por filtro e arquitetura → SHA-256 → extrai (zip/tar) ou `chmod +x` no AppImage → registra | nada |
| Windows | idem; zip portátil ou `.exe`; cria atalho | SmartScreen em binário sem assinatura |
| macOS | idem; `.app` em zip; remove `com.apple.quarantine` **com o consentimento do usuário** | sem notarização o Gatekeeper reclama; o launcher explica, não contorna sozinho |
| Android | baixa o APK da release, confere assinatura e versão, chama o instalador do sistema (como o Quiver) | só APKs; o usuário aceita "fontes desconhecidas" |
| iOS | **não instala**. Mostra a release, explica o caminho (`altstore://install?url=…` / SideStore) e "compile o seu" para recomps | é a plataforma; não há como prometer mais |

**Recomps não têm release instalável** em lugar nenhum (código traduzido do disco). Para elas a porta é
sempre a 5.4; a entrada do catálogo diz isso em vez de oferecer um botão que não pode existir. No
Android (fase 5) o fluxo é "compile no PC, **envie ao celular**" pela rede local, e o launcher do
celular instala o APK que o launcher do PC gerou.

### 5.6 Mods

Thunderstore e GameBanana entram na fase 4 com o modelo do Quiver (fontes por jogo, layout por mod,
dependências). No BlueWake, mods são **compilados** no composite (máscara de bits), não arquivos
soltos — e mudam o gate de netplay. O catálogo distingue "mods de arquivo" de "mods compilados".

---

## 6. Plano por fases

A ordem segue a regra "primeiro uma versão útil e pequena", com a decisão número um (contrato) provada
antes de qualquer tela de rede.

**Fase 0 — Prova do contrato (sem interface).** Fork do BlueWake em diretório próprio. Snapshot em
memória incremental (1.4 item 1), laço "um quadro por chamada" (item 2), ressimulação silenciosa
(item 3). Harness Rust com `SyncTestSession` headless a partir do `quick-2766`: 10 minutos sem
divergência. Medir: ms de `salvar`, de `restaurar`, de `avancar` em ressimulação; páginas sujas com
janela. **Entrega:** `netplay_abi.h` v1 congelado e uma tabela de números no lugar das hipóteses
H-01, H-03, H-04, H-08. Se a prova falhar em determinismo com janela, a seção 1.4 item 4 vira a
fase inteira, e o resto espera.

**Fase 1 — Launcher útil, desktop Linux (depois Windows).** Projeto Flutter + `nucleo` via bridge.
Biblioteca em grade com capas, página do jogo, tags, busca, filtros, entradas manuais, perfis de
lançamento (offload Nvidia, `taskset`, variáveis), vigia de processo e log, verificação de ambiente,
identificação/conversão de disco e **assistente de compilação embrulhando os scripts atuais**.
**Substitui o `recomps.sh`.** Sem rede. Critério: o dono joga pelo launcher e compila por ele.

**Fase 2 — Amigos.** Identidade, tickets/QR, lista de amigos, presença e convites via iroh (relays
públicos). Catálogo em git e instalação por releases no desktop. App Android e iOS com biblioteca,
amigos e convites — sem rodar jogo. Atualização do próprio launcher. Critério: dois amigos se veem
online e trocam convite sem servidor nosso.

**Fase 3 — Netplay v1.** `libnetplay` integrada no fork do BlueWake; sala compara descritores; **modo
lockstep primeiro**, depois delay + rollback curto; slot compartilhado com passagem de controle;
espectador por entradas no desktop. Critério: duas casas jogam Wind Waker com o mesmo controle sem
dessincronia por uma hora.

**Fase 4 — Acabamento.** Relay próprio no VPS do dono; ajuste de rollback com medição real; mods
(Thunderstore/GameBanana); macOS; espectador por **vídeo** para o celular (medir banda antes).

**Fase 5 — Jogo no celular.** Intent/URL com ticket; "enviar ao celular" o APK compilado no PC;
iOS pelo caminho de sideload do próprio BlueWake. Depende de o BlueWake Android sair de experimental.

---

## 7. Riscos

| risco | efeito | mitigação |
|---|---|---|
| Não determinismo **com janela** que não se consegue fechar (EFB peeks, GPU); headless já está provado | netplay só headless = inútil | fase 0 ataca primeiro; plano B é peek determinístico com perda visual declarada |
| CPU sem folga para ressimular | engasgos frequentes | delay-first, lockstep disponível, medir antes de prometer |
| Relays públicos limitados/instáveis | sessões em relay ruins ou recusadas | relay próprio na fase 4; indicador de relay na interface desde a fase 3 |
| Amigo não consegue compilar (Docker, 30 GB, 1h45) | ninguém para jogar | assistente de compilação é fase 1, não extra; Windows pode usar build pronta se o digest bater |
| Upstream do BlueWake muda o host rápido (está ativo) | fork desatualiza | patches pequenos, de host, oferecidos ao upstream cedo; nunca tocar no gerado |
| Licença do dono vier GPL | nada muda no desenho (D-01 já isola) | — |

---

## 8. Perguntas abertas para o dono

1. O que é "jogar com amigos" no Wind Waker: slot compartilhado + espectador basta para a v1? Há
   próxima recomp multiplayer na fila?
2. Licença do launcher e da biblioteca (premissa: Apache-2.0).
3. Distribuição aos amigos: cada um compila o seu (premissa, e recomendação da gerente)?
4. Celular na v1 sem rodar jogo (premissa: sim).
5. Nome do projeto — define o prefixo da ABI (`netplay_*` por enquanto) e o URL scheme.

---

## 9. Hipóteses a verificar

| id | hipótese | como verificar | fase |
|---|---|---|---|
| H-01 | Com janela, a MEM1 muda mais que os 0,4 MiB/quadro medidos headless (XFB, EFB copies), mas segue abaixo de alguns MiB | repetir a medição de hoje com renderer Aurora | 0 |
| H-02 | Os 16 MiB "sujos" por quadro são artefato de THP (`enabled = always`, faixas inteiras de 2 MiB alinhadas, 704 MB de AnonHugePages) | `madvise(MADV_NOHUGEPAGE)` na MEM1 e repetir | 0 |
| H-03 | A variação "de execução para execução" com janela vem dos EFB peeks e de recursos de host por retrace, não do domínio de ciclos (headless **provado** igual byte a byte em 22/10) | repetir o teste A/B de hoje com renderer Aurora, depois com peeks desligados; comparar também o chunk GX | 0 |
| H-04 | O jogo lê o PAD uma vez por quadro de jogo (30 Hz) | contar chamadas ao SI por retrace no host | 0 |
| H-05 | O iroh expõe datagramas QUIC não confiáveis na conexão, com MTU ≥ 1200 bytes | ler `iroh::endpoint::Connection` na versão escolhida | 2 |
| H-06 | iroh e flutter_rust_bridge são MIT/Apache-2.0 | LICENSE dos repositórios | 2 |
| H-07 | iOS permite `mprotect` em memória anônima com handler de exceção no próprio processo | protótipo de 50 linhas no simulador e num aparelho | 5 |
| H-08 | `avancar` em ressimulação custa ~30 ms no mar (91 % de 33 ms); XXH3 de 48 MiB custa 3–6 ms | medir na fase 0 | 0 |
| H-09 | A maioria dos amigos do dono está atrás de CGNAT IPv4 | o próprio launcher reporta direto/relay por sessão | 3 |
| H-10 | Kernels Android têm soft-dirty desligado | ler config do kernel do aparelho; fallback é `mprotect` | 5 |
| H-11 | O mesmo binário em **duas máquinas diferentes** (CPUs, GPUs) produz o mesmo estado byte a byte — a contagem de ciclos é por instrução, não por tempo, então deve valer | o teste A/B de hoje com A nesta máquina e B em outra; `jet-dev` serve se o dono autorizar levar o binário para lá (é cópia privada dele, não distribuição) | 0 |

---

## 10. Medições feitas em 22/10/2026 (esta máquina: i7-7700HQ, 15 GB, Debian, kernel 6.12)

Autorizadas pela gerente; máquina livre (carga 0,4), nada de build nem jogo rodando; cartão de memória
do dono preservado (cópia em `/tmp`); nenhum processo deixado vivo.

**Cópia de memória** (`memcpy`, 20 repetições, melhor tempo): 48 MiB em **6,45 ms** (7,8 GB/s),
24 MiB em 3,2 ms, 4 MiB em 0,23 ms (cabe em cache).

**Estado real `quick-2766.bwstate`** (mar aberto): 20,7 MB em disco; descomprimido **50,3 MiB**
(MEM1 32 MiB + ARAM 16 MiB + ALIASES 1,5 MiB + GX 742 KiB + HOSTVARS 68 KiB + CPU 3 KiB + DSPHLE
4 KiB + VICLOCK/PE/LOOPVARS); 4 031 de 12 873 páginas de 4 KiB são zero. `gunzip` 0,31 s;
`gzip -1` 0,88 s (o cabeçalho do BlueWake estima "algumas centenas de ms"); `zstd -1` **0,10 s** para o
mesmo tamanho (19,4 MiB) — vale para o save manual, não para rollback. O jogo carregou o estado em
210 ms (`[state] loaded … ms=210`).

**Páginas sujas por quadro** (BlueWake headless, `BLUEWAKE_WALL_PACE=1`, um núcleo, a partir do
`quick-2766`, 50 s a 30 amostras/s e depois 30 s a 10 amostras/s, soft-dirty via
`/proc/<pid>/pagemap` + `clear_refs` de um processo vizinho):

- região de 32,2 MiB (MEM1): **4 111 páginas marcadas por quadro, constantes** (mediana = p99), em
  faixas inteiras de 2 MiB alinhadas → THP (H-02). Comparando o **conteúdo** das páginas marcadas:
  **~114 páginas de 4 KiB realmente alteradas por quadro** (mediana; máximo 368 por janela de 100 ms)
  = **~0,4 MiB/quadro**, concentradas em +23 MiB (117/janela), +10 MiB (55), +22 MiB (47), +9 MiB (36).
- heap de 21 MiB: 518 páginas marcadas/quadro (uma página grande); região anônima de 6,6 MiB: 1 035;
  região de 8 MiB: ~0.
- custo do método externo: leitura do pagemap 2,6 ms, `clear_refs` 0,7 ms por amostra — é custo da
  ferramenta, não do mecanismo que o jogo usaria.

**Leitura:** snapshot integral por quadro (6,5 ms) não cabe numa thread a 91 %; snapshot incremental a
4 KiB copia ~0,4 MiB (décimos de ms). A granularidade de 2 MiB do THP precisa ser desligada na MEM1,
senão o incremental copia 16 MiB (2 ms) — ainda aceitável, mas desnecessário.

**A medição e a resposta.** A primeira leitura dizia "16 MiB sujos por quadro, constantes" — e
"constante" é o que não fecha: lógica de jogo não escreve a mesma quantidade exata de páginas a cada
quadro, e faixas inteiras de 2 MiB alinhadas marcadas de ponta a ponta são a assinatura de página
grande, não de escrita. Aceitar o número teria levado a "cópia incremental custa 2 ms e o rollback
não compensa". Comparar o **conteúdo** das páginas marcadas separou o que o kernel contou do que o
jogo fez. Fica registrado como regra: um número que sai redondo demais é pergunta, não resposta.

**Determinismo, execução contra execução** (segundo experimento, autorizado pela gerente; headless,
`BLUEWAKE_WALL_PACE=0`, um núcleo, 41 s no total, nenhum processo deixado vivo). Duas execuções A e B,
idênticas: carregar `quick-2766`, aplicar `BLUEWAKE_PAD_SCRIPT` com cinco pressionamentos (stick para
a frente, A, stick para o lado, Z, B, entre os retraces 2780 e 3606), salvar no retrace 2786 (10
quadros) e no 3966 (600 quadros), com `[guest-checkpoint]` a cada 100 retraces.

- Os dois saves de cada execução caíram no mesmo ponto (`retrace=2786` e `retrace=3967`,
  `pc=0x80322BC8`), ou seja, o ponto limpo também é determinístico.
- **Todos os chunks iguais byte a byte** entre A e B, em 10 e em 600 quadros: `CPU` (3 400 B),
  `MEM1` (32 MiB), `ARAM` (16 MiB), `ALIASES` (1,5 MiB), `VICLOCK`, `HOSTVARS` (70 242 B), `PE`,
  `LOOPVARS`, `DSPHLE` (4 414 B). Os 13 checkpoints intermediários (XXH3 de CPU, MEM1, ARAM, aliases)
  também iguais.
- Sem freio de ritmo, as duas execuções correram em velocidades de host ligeiramente diferentes — e
  nada vazou para o estado. É a confirmação prática de que o domínio de ciclos não lê o relógio.
- Limite do teste: headless não tem chunk `GX` nem EFB peeks; a execução com janela é H-03, e a
  comparação entre máquinas é H-11. O DSP está em HLE nos dois; o LLE (não usado) não entra no save.
