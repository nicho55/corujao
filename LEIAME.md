# POG — Plataforma Orientada a Gambiarra

Um launcher de jogos com cara de Steam, aberto e multiplataforma, para **preservar e jogar com
amigos** jogos recompilados. Você coloca na biblioteca as recomps que compilou, aperta Jogar,
e o POG cuida do resto: abre o jogo com os ajustes certos para a sua máquina, mostra o registro
da execução e fecha limpo quando você pede. A aba Explorar mostra o que existe no mundo das
recomps e explica o que cada repositório é.

O nome é a piada: *Programação Orientada a Gambiarra*. O programa, não.

> **Importante: o POG não vem com jogo nenhum, e não vai vir.** Leia
> [Por que o jogo não vem junto](#por-que-o-jogo-não-vem-junto) antes de procurar o Zelda.

## O que ele faz hoje (v0.1)

- **Biblioteca** em grade com capas; clique abre a página do jogo.
- **Jogar / Fechar / Forçar**: abre o jogo com a *receita* da sua máquina (variáveis de ambiente
  e prefixos vêm de um arquivo, não do código), guarda o PID, pede para fechar com calma e só
  oferece forçar depois de esperar.
- **Explorar**: o catálogo de recomps (hoje, o do QuiverLauncher) com o que cada repositório faz
  (recompilação? porte? descompilação?), de que console é, qual versão do jogo cobre e se há
  atualização. O catálogo é lido do disco; a rede só entra quando você clica em "Atualizar pela
  rede".
- **Aparência**: claro, escuro ou seguir o sistema, no botão do canto direito. Nenhuma cor está
  escrita nas telas: tudo vem do tema, então temas novos são só uma paleta a mais.
- **Uma pasta só** com tudo do launcher (`~/jogos/pog` no Linux, `%USERPROFILE%\jogos\pog` no
  Windows). Copie-a inteira para levar o POG para outra máquina.

## Instalar

### Windows

1. Baixe o `pog-windows-x64.zip` da [última release](../../releases/latest).
2. Extraia onde quiser e abra `pog.exe`. O Windows pode avisar que o programa não é assinado;
   é o preço de não pagarmos certificado. "Mais informações" → "Executar assim mesmo".
3. Na primeira abertura a biblioteca vem vazia. Siga [Trazer o seu jogo](#trazer-o-seu-jogo).

### Linux

Pacote pronto:

1. Baixe o `pog-linux-x64.tar.gz` da [última release](../../releases/latest).
2. `tar -xzf pog-linux-x64.tar.gz && ./pog/pog`.

Ou compile do repositório (precisa de Flutter, Rust, GTK 3, cmake, ninja e clang):

```bash
git clone https://github.com/nicho55/pog.git
cd pog
./abrir.sh --compilar      # compila interface + núcleo e abre; depois, só ./abrir.sh
```

Desenvolvimento com recarga a quente da interface: `cd interface && flutter run -d linux`.
Mudou `nucleo/src/api/`? Rode `cd interface && flutter_rust_bridge_codegen generate`.

### macOS, Android e iOS

Estão no desenho (é o mesmo código), mas ainda não há pacote. Quem quiser compilar: `flutter
build macos` / `apk` / `ios` em `interface/`.

## Por que o jogo não vem junto

Uma **recomp** (recompilação estática) traduz o código de um jogo para C e compila nativo. O
resultado é o código do jogo, só que em outra forma — e o código do jogo é de quem o fez. O
projeto de recompilação distribui a **ferramenta**; o jogo compilado sai do **seu** disco, na
**sua** máquina, e fica com você. Publicar um binário de recomp derrubaria o repositório e
exporia quem publicou. Por isso o POG distribui o launcher, e nunca um jogo.

O que **pode** circular é texto: a *receita* que faz o jogo abrir direito numa máquina como a
sua (quais variáveis, qual prefixo), a capa, as tags, qual versão do disco a recomp cobre. É
isso que o POG guarda e, no futuro, vai trocar entre amigos.

## Trazer o seu jogo

1. **Compile a recomp** a partir do seu disco, seguindo as instruções do repositório dela (a aba
   Explorar tem o link de cada um). Para o Wind Waker, o projeto é o BlueWake.
2. Crie a pasta do jogo na raiz do POG: `~/jogos/pog/jogos/<id>/` (`<id>` é um nome curto, sem
   espaços). Copie [`exemplos/jogos/meu-jogo/jogo.toml`](exemplos/jogos/meu-jogo/jogo.toml) para
   lá, aponte `executavel` para o que você compilou e ajuste nome, console e `[disco]`.
3. Ponha uma `capa.png` em 2:3 ao lado (600×900 fica bom).
4. Se a sua máquina precisa de ajuste para o jogo abrir (laptop com duas placas de vídeo, por
   exemplo), copie [`exemplos/maquina.toml`](exemplos/maquina.toml) para `~/jogos/pog/maquina.toml`
   e aponte a receita no `jogo.toml`.
5. No POG, clique em recarregar. O jogo aparece na biblioteca; a página dele mostra a linha de
   comando exata que o botão Jogar vai usar.

O POG **nunca mexe nos seus saves**: eles ficam onde o jogo os guarda.

## Onde está o quê

| pasta | o que é |
|---|---|
| `interface/` | o app Flutter. Telas em `interface/lib/`; `interface/lib/src/rust/` é **gerado** pelo bridge, não edite |
| `nucleo/` | o crate Rust: `raiz.rs` (a pasta de dados), `biblioteca.rs` (`jogo.toml`), `maquina.rs` (receitas, RF-01), `execucao.rs` (abrir/fechar por PID, com a parte de sistema operacional separada), `disco.rs` (qual cópia do jogo, RF-05), `catalogo/` (fontes do catálogo), `configuracao.rs` (tema), `api/` (o que o Flutter enxerga) |
| `interface/rust_builder/` | cola do cargokit: compila o `nucleo/` dentro do `flutter build`. Gerado; só muda quando o crate muda de lugar |
| `exemplos/` | `maquina.toml` e `jogo.toml` para copiar |
| `documentacao/` | briefing, arquitetura e decisões, armadilhas de rodar o BlueWake, requisitos |
| `.github/workflows/` | gera os pacotes Windows e Linux a cada tag `v*` |
| `abrir.sh` | abre o launcher no Linux (compila na primeira vez) |

A pasta de dados:

```
~/jogos/pog/
  jogos/<id>/jogo.toml   metadados de um jogo: nome, console, executável, receita, saves, disco
  jogos/<id>/capa.png    capa em 2:3
  maquina.toml           receitas de execução DESTA máquina
  configuracao.toml      suas preferências (tema)
  catalogo/              o catálogo baixado por "Atualizar pela rede"
  registros/             saída de cada execução, um arquivo por vez
```

Troque a raiz com a variável de ambiente `POG_RAIZ`. O nome do programa vive numa constante só,
em `nucleo/src/nome.rs`.

## Para quem for mexer

- Primeiro passo depois de clonar: `git config core.hooksPath .githooks`. O hook recusa commit com
  binário de jogo ou arquivo acima de 5 MB; o CI confere a mesma coisa em todo push.
- Tudo em português do Brasil, inclusive nomes de arquivo, função e variável.
- Classe da janela no Linux: `io.github.nicho55.pog` (`xdotool search --class io.github.nicho55.pog`).
  Não procure pelo título: um terminal mostrando este texto também casa.
- Testes do núcleo: `cd nucleo && cargo test`. Análise da interface: `cd interface && flutter analyze`.
- Decisões de arquitetura (D-01 a D-14) e requisitos (RF-01 a RF-06) em `documentacao/`.

## Licença

Apache-2.0 (ver [LICENSE](LICENSE)). Os jogos têm as licenças deles e não fazem parte deste projeto.
