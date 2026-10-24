# Briefing do projeto — a "Steam dos recomps"

**Estado:** aberto em 22/10/2026. Nasceu como POG (Plataforma Orientada a Gambiarra) e, em 24/10/2026, o dono o renomeou para **Corujão**, em homenagem às noites viradas em lan houses e na casa de amigos jogando. Repositório em `~/projetos/corujao`, dados em `~/jogos/corujao`.

**Papéis:** o dono (neps) decide. A sessão Claude `neps-0d` é a gerente de projeto: é com ela
que se tiram dúvidas e é ela que escreve e atualiza esta documentação. Quem implementa recebe
pacotes de trabalho dela.

---

## 1. O que é, em uma frase

Um launcher de jogos com cara e funcionamento de Steam, aberto, que roda em qualquer PC e
celular, para **preservar e jogar com amigos** a coleção de jogos recompilados do dono — com
**multiplayer online P2P e rollback já embutido no app**, sem depender de serviço de terceiro.

## 2. Por que existe

O dono mantém `~/projetos/RECOMPS`, uma coleção de recompilações estáticas de jogos (hoje o
BlueWake, Wind Waker de GameCube). O painel atual é um script zenity (`bin/recomps.sh`): lista
o que está compilado e roda. É suficiente para uma pessoa numa máquina e insuficiente para o
objetivo real, que é jogar com os amigos e preservar o acervo.

## 3. Pilha decidida pelo dono

- **Rust** no núcleo (lógica, rede, estado, integração com os jogos).
- **Flutter** na interface.
- Alvos: **Windows, Linux, macOS, Android e iOS**. Mesmo app, mesma base.

Não discutir a pilha; discutir *como* montá-la bem.

## 4. Referência de produto

- **Visual e funcionamento: Steam.** Biblioteca em grade com capas, página do jogo, estado de
  instalação, amigos, convite para partida, "jogar agora".
- **Funcionalidades: QuiverLauncher**, em `~/projetos/quiver-launcher` (Avalonia, C#, .NET 10,
  Desktop + Android, licença no repo). Ler o `README.md` e as `*ViewModel.cs` para o inventário
  real. O que ele faz e nós queremos:
  - biblioteca com **tags**, busca por nome/tag/repositório/pasta;
  - **apps manuais**, sem repositório, apontando para uma pasta;
  - **catálogo** pesquisável (o dele é o quiverlauncher.com);
  - **instalar e atualizar a partir de releases** do GitHub e do GitLab;
  - **gestão de mods** (Thunderstore e GameBanana);
  - editor de entradas, filtros de exibição, mapeamento de controles.
  É um fork do GithubLauncher. Não copiar código: é C# e nós vamos para Rust + Flutter. Copiar
  **o conjunto de funcionalidades e as decisões de produto que fazem sentido**.

## 5. A pedra no caminho, e ela é a decisão número um

**Rollback não é algo que um launcher adiciona a um jogo qualquer.** Rollback exige:

1. salvar o estado completo da máquina emulada em poucos milissegundos;
2. restaurar esse estado igualmente rápido;
3. avançar N quadros de forma **determinística**, com as entradas que a rede mandar;
4. que dois computadores rodando o mesmo binário, do mesmo estado, com as mesmas entradas,
   cheguem ao **mesmo estado byte a byte**.

Um launcher que só executa um binário de terceiro não consegue nada disso. Então o desenho
certo é o launcher definir um **contrato** que o jogo implementa, e oferecer a camada de rede
para quem cumprir. Jogo que não cumprir entra na biblioteca normalmente, só sem netplay.

A boa notícia: **os recomps já têm metade da peça.** O BlueWake salva e carrega estado
(`~/.local/share/BlueWake/quick-<N>.bwstate`, variável `BLUEWAKE_LOAD_STATE`, e o log imprime
`[state] loaded ... retrace=N ... mismatched=0`). Falta medir se o salvar/restaurar é rápido o
bastante para rollback e se o avanço é determinístico.

Perguntar à gerente o que for preciso sobre o BlueWake; ela tem o histórico do projeto.

## 6. Restrições

- **Tudo em português do Brasil**, inclusive commits, documentação e nomes de requisito.
- **Não tocar em `~/projetos/RECOMPS/bluewake`**: é cópia de trabalho de um repositório de
  terceiro, com regras próprias. Não commitar, não abrir PR, não empurrar nada lá.
- Não commitar nada em lugar nenhum sem o dono pedir.
- A máquina é modesta: 4 núcleos, 15 GB de RAM. Evitar desenhos que só fechem em servidor.
- Aberto desde o início. A licença o dono escolhe; perguntar antes de assumir.

## 7. Glossário

- **recomp**: recompilação estática. O código do jogo é traduzido para C e compilado nativo,
  em vez de interpretado por um emulador.
- **BlueWake**: o recomp do Wind Waker que o dono mantém e otimiza.
- **rollback**: técnica de netcode que prevê as entradas do outro jogador, e quando erra volta
  o estado e recalcula. É o que faz luta e ação online parecerem locais.
- **estado (bwstate)**: retrato completo da máquina emulada, que o BlueWake já sabe salvar.
