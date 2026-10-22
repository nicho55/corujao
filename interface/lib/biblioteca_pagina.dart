import 'dart:io';

import 'package:flutter/material.dart';

import 'jogo_pagina.dart';
import 'src/rust/api.dart';
import 'tema.dart';
import 'widgets/barra_da_secao.dart';
import 'widgets/etiqueta.dart';
import 'widgets/faixa.dart';
import 'widgets/formato.dart';

/// A biblioteca: grade de capas, como a Steam. Clicar numa capa abre a página do jogo.
class BibliotecaPagina extends StatefulWidget {
  const BibliotecaPagina({super.key});

  @override
  State<BibliotecaPagina> createState() => _BibliotecaPaginaState();
}

class _BibliotecaPaginaState extends State<BibliotecaPagina> {
  late Biblioteca _biblioteca;
  late List<String> _avisos;
  late String _raiz;

  @override
  void initState() {
    super.initState();
    _recarregar();
  }

  void _recarregar() {
    setState(() {
      _biblioteca = biblioteca();
      _avisos = avisos();
      _raiz = raiz();
    });
  }

  Future<void> _abrir(JogoInfo jogo) async {
    await Navigator.of(context).push(MaterialPageRoute(builder: (_) => JogoPagina(jogo: jogo)));
    _recarregar();
  }

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    final jogos = _biblioteca.jogos;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      BarraDaSecao(
        titulo: 'BIBLIOTECA',
        detalhe: contagem(jogos.length, 'jogo', 'jogos'),
        acoes: [
          Tooltip(
            message: 'Pasta com tudo do launcher: jogos, capas, receitas, catálogo e registros. Copie-a inteira para levar para outra máquina.',
            child: Text(_raiz, style: TextStyle(color: cores.textoApagado, fontSize: 12)),
          ),
          const SizedBox(width: 8),
          IconButton(onPressed: _recarregar, tooltip: 'Reler a biblioteca', icon: const Icon(Icons.refresh)),
        ],
      ),
      for (final aviso in _avisos) Faixa(texto: aviso, cor: cores.alerta),
      for (final problema in _biblioteca.problemas) Faixa(texto: problema, cor: cores.erro),
      Expanded(
        child: jogos.isEmpty
            ? _Vazio(raiz: _raiz)
            : GridView.builder(
                padding: const EdgeInsets.fromLTRB(24, 8, 24, 24),
                gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                  maxCrossAxisExtent: 220,
                  childAspectRatio: 2 / 3,
                  crossAxisSpacing: 20,
                  mainAxisSpacing: 20,
                ),
                itemCount: jogos.length,
                itemBuilder: (context, i) => _Capa(jogo: jogos[i], aoAbrir: () => _abrir(jogos[i])),
              ),
      ),
    ]);
  }
}

class _Vazio extends StatelessWidget {
  const _Vazio({required this.raiz});
  final String raiz;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Center(
      child: Column(mainAxisSize: MainAxisSize.min, children: [
        Icon(Icons.inventory_2_outlined, size: 64, color: cores.textoApagado),
        const SizedBox(height: 16),
        const Text('A biblioteca está vazia.', style: TextStyle(fontSize: 18)),
        const SizedBox(height: 8),
        Text('Crie $raiz/jogos/<id>/jogo.toml (e uma capa.png ao lado) e clique em recarregar.', style: TextStyle(color: cores.textoApagado)),
      ]),
    );
  }
}

/// Um cartão de capa em 2:3. Sem capa, mostra o nome sobre um fundo escuro.
class _Capa extends StatefulWidget {
  const _Capa({required this.jogo, required this.aoAbrir});
  final JogoInfo jogo;
  final VoidCallback aoAbrir;

  @override
  State<_Capa> createState() => _CapaState();
}

class _CapaState extends State<_Capa> {
  bool _sobre = false;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    final jogo = widget.jogo;
    return MouseRegion(
      onEnter: (_) => setState(() => _sobre = true),
      onExit: (_) => setState(() => _sobre = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.aoAbrir,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          transform: Matrix4.identity()..scaleByDouble(_sobre ? 1.04 : 1.0, _sobre ? 1.04 : 1.0, 1.0, 1.0),
          transformAlignment: Alignment.center,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(6),
            boxShadow: [
              BoxShadow(color: Colors.black.withValues(alpha: _sobre ? 0.6 : 0.35), blurRadius: _sobre ? 18 : 8, offset: const Offset(0, 6)),
            ],
            border: Border.all(color: _sobre ? cores.destaque : Colors.transparent, width: 2),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: Stack(fit: StackFit.expand, children: [
              if (jogo.capa != null)
                Image.file(File(jogo.capa!), fit: BoxFit.cover)
              else
                Container(
                  color: cores.painelEscuro,
                  alignment: Alignment.center,
                  padding: const EdgeInsets.all(12),
                  child: Text(jogo.nome, textAlign: TextAlign.center, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                ),
              if (!jogo.executavelExiste) Positioned(top: 8, left: 8, child: Etiqueta('não instalado', cor: cores.erro)),
              if (jogo.console != null) Positioned(bottom: 8, right: 8, child: Etiqueta(jogo.console!, cor: cores.painelEscuro, sobre: cores.texto)),
            ]),
          ),
        ),
      ),
    );
  }
}
