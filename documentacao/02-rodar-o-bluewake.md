# Como lançar o BlueWake (o que já aprendemos na marra)

Escrito pela gerente em 22/10/2026, a partir do trabalho de otimização feito hoje nesta
máquina. **Leia antes de escrever o código que executa um jogo**, porque cada item abaixo
custou tempo para alguém descobrir.

## O que já está pronto no disco

| caminho | o que é |
|---|---|
| `~/projetos/RECOMPS/build/linux/BlueWake-x86_64.AppImage` | o jogo empacotado, 234 MB, build de 22/10 11:16 |
| `~/projetos/RECOMPS/build/linux/BlueWake/` | a árvore solta: `bluewake` (app) + `gGZLE01_recomp.so` (o jogo) + `libs/` |
| `~/.local/share/BlueWake/` | dados do jogador: saves, `quick-<N>.bwstate`, `settings.ini`, `logs/` |
| `~/projetos/RECOMPS/recomps.d/bluewake.conf` | o registro que o painel atual lê |

Prefira a **AppImage** para "instalar e jogar": é um arquivo só, que é o espírito do projeto.
A árvore solta serve para depuração.

## A receita de lançamento

```bash
__NV_PRIME_RENDER_OFFLOAD=1 __VK_LAYER_NV_optimus=NVIDIA_only \
__GLX_VENDOR_LIBRARY_NAME=nvidia \
stdbuf -oL -eL ./BlueWake-x86_64.AppImage
```

Para a árvore solta, acrescente `LD_LIBRARY_PATH` apontando para `../libs`.

## Armadilhas, cada uma confirmada aqui

1. **Placa de vídeo.** Este laptop é Optimus: tem Intel integrada e uma GTX 1060. Sem as três
   variáveis acima o jogo abre na Intel e fica lento. Com elas, sai pela Nvidia por Vulkan.
   Num app multiplataforma, isso é configuração por máquina, não constante no código.

2. **A saída fica presa quando não é terminal.** O jogo buffeiza stdout/stderr se não enxergar
   um tty, então um launcher que capture a saída por pipe não vê nada até o processo morrer.
   Lance com `stdbuf -oL -eL`, ou use pty.

3. **Nunca matar por nome com busca de padrão.** `pkill -f` com o nome da AppImage casa a
   própria linha de comando de quem procura e mata a sessão inteira. **Guarde o PID e mate por
   PID.** Isso vale para o botão "fechar jogo" do launcher.

4. **Saída suja deixa o jogo em modo seguro.** Instância que não encerra direito faz o próximo
   lançamento abrir em 4:3, em janela e sem alta resolução. Ele sai disso sozinho depois de uma
   saída limpa pelo menu. O launcher deve mandar `SIGTERM` e dar tempo, não `SIGKILL` de cara.

5. **A janela não é "a janela".** A classe é `dev.bluewake.BlueWake`, e o i3 pode colocá-la em
   outro espaço de trabalho — já pareceu "o botão não funcionou" quando o jogo tinha aberto do
   lado. Se o launcher for trazer a janela para a frente, procure pela classe.

6. **O `settings.ini` vence as variáveis de ambiente.** Para forçar uma configuração sem mexer
   nas preferências do jogador, monte um `XDG_CONFIG_HOME` próprio com o seu `settings.ini`.
   É o que o `bin/medir-ritmo.sh` faz.

7. **Turbo da CPU volta desligado a cada reinício.** Com ele desligado o jogo roda bem mais
   devagar. Ligar exige root. Um launcher pode **avisar**, não consertar sozinho.

## Como o jogo é lançado hoje

`~/projetos/RECOMPS/bin/jogar-bluewake.sh` e o painel `bin/recomps.sh` (zenity). Leia os dois:
é o comportamento que o launcher novo precisa cobrir antes de poder substituí-los.

## O estado do jogador é sagrado

Nunca apague nem sobrescreva `~/.local/share/BlueWake/`. Saves e cartão de memória do dono
vivem ali. Em teste, use `XDG_DATA_HOME` isolado ou cópia em `/tmp`.
