import 'package:flutter/material.dart';

import 'biblioteca_pagina.dart';
import 'explorar_pagina.dart';
import 'tema.dart';
import 'widgets/faixa.dart';

/// As abas do topo, como a Steam (LOJA / BIBLIOTECA): aqui BIBLIOTECA e EXPLORAR.
enum Aba {
  biblioteca('BIBLIOTECA', Icons.sports_esports),
  explorar('EXPLORAR', Icons.explore_outlined);

  const Aba(this.rotulo, this.icone);
  final String rotulo;
  final IconData icone;
}

/// A casca do app: cabeçalho com nome, abas e seletor de aparência; o corpo é a aba ativa.
/// As páginas ficam vivas numa [IndexedStack], então trocar de aba não perde busca nem rolagem.
class Casca extends StatefulWidget {
  const Casca({
    super.key,
    required this.nome,
    required this.nomeCompleto,
    required this.modo,
    required this.aoTrocarModo,
    this.aviso,
  });
  final String nome;
  final String nomeCompleto;
  final ModoDeTema modo;
  final ValueChanged<ModoDeTema> aoTrocarModo;
  /// Algo que o app quer dizer logo na abertura (ex.: configuração que não deu para ler).
  final String? aviso;

  @override
  State<Casca> createState() => _CascaState();
}

class _CascaState extends State<Casca> {
  Aba _aba = Aba.biblioteca;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Scaffold(
      body: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _Cabecalho(
          nome: widget.nome,
          nomeCompleto: widget.nomeCompleto,
          ativa: _aba,
          aoEscolher: (a) => setState(() => _aba = a),
          modo: widget.modo,
          aoTrocarModo: widget.aoTrocarModo,
        ),
        if (widget.aviso != null) Faixa(texto: widget.aviso!, cor: cores.alerta),
        Expanded(
          child: IndexedStack(
            index: _aba.index,
            children: const [BibliotecaPagina(), ExplorarPagina()],
          ),
        ),
      ]),
    );
  }
}

class _Cabecalho extends StatelessWidget {
  const _Cabecalho({
    required this.nome,
    required this.nomeCompleto,
    required this.ativa,
    required this.aoEscolher,
    required this.modo,
    required this.aoTrocarModo,
  });
  final String nome;
  final String nomeCompleto;
  final Aba ativa;
  final ValueChanged<Aba> aoEscolher;
  final ModoDeTema modo;
  final ValueChanged<ModoDeTema> aoTrocarModo;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Container(
      color: cores.painelEscuro,
      padding: const EdgeInsets.only(left: 24, right: 12),
      height: 56,
      child: Row(children: [
        Icon(Icons.sports_esports, color: cores.destaque, size: 28),
        const SizedBox(width: 12),
        Tooltip(
          message: nomeCompleto,
          child: Text(nome, style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700, color: cores.texto, letterSpacing: 1)),
        ),
        const SizedBox(width: 28),
        for (final aba in Aba.values) _BotaoDeAba(aba: aba, ativa: aba == ativa, aoEscolher: () => aoEscolher(aba)),
        const Spacer(),
        _SeletorDeTema(modo: modo, aoTrocar: aoTrocarModo),
      ]),
    );
  }
}

/// Uma aba do cabeçalho: texto em caixa alta, sublinhado na cor de destaque quando ativa.
class _BotaoDeAba extends StatelessWidget {
  const _BotaoDeAba({required this.aba, required this.ativa, required this.aoEscolher});
  final Aba aba;
  final bool ativa;
  final VoidCallback aoEscolher;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    final cor = ativa ? cores.destaque : cores.textoApagado;
    return InkWell(
      onTap: aoEscolher,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 14),
        decoration: BoxDecoration(border: Border(bottom: BorderSide(color: ativa ? cores.destaque : Colors.transparent, width: 3))),
        alignment: Alignment.center,
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Icon(aba.icone, size: 18, color: cor),
          const SizedBox(width: 8),
          Text(aba.rotulo, style: TextStyle(letterSpacing: 2, fontWeight: FontWeight.w600, color: cor)),
        ]),
      ),
    );
  }
}

/// O seletor de aparência (RF-06): sistema, claro ou escuro. Troca o tema inteiro.
class _SeletorDeTema extends StatelessWidget {
  const _SeletorDeTema({required this.modo, required this.aoTrocar});
  final ModoDeTema modo;
  final ValueChanged<ModoDeTema> aoTrocar;

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<ModoDeTema>(
      tooltip: 'Aparência: ${modo.rotulo}',
      icon: Icon(modo.icone),
      initialValue: modo,
      onSelected: aoTrocar,
      itemBuilder: (_) => [
        for (final m in ModoDeTema.values)
          CheckedPopupMenuItem(value: m, checked: m == modo, child: Row(children: [Icon(m.icone, size: 18), const SizedBox(width: 10), Text(m.rotulo)])),
      ],
    );
  }
}
