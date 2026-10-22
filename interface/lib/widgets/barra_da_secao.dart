import 'package:flutter/material.dart';

import '../tema.dart';

/// A barra que abre cada aba (BIBLIOTECA, EXPLORAR): título em caixa alta, um detalhe ao lado
/// ("3 jogos", "515 recomps · cache local") e as ações da aba à direita.
class BarraDaSecao extends StatelessWidget {
  const BarraDaSecao({super.key, required this.titulo, this.detalhe, this.acoes = const []});
  final String titulo;
  final String? detalhe;
  final List<Widget> acoes;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Container(
      padding: const EdgeInsets.fromLTRB(24, 14, 16, 10),
      child: Row(children: [
        Text(titulo, style: TextStyle(letterSpacing: 2, color: cores.destaque, fontWeight: FontWeight.w600)),
        const SizedBox(width: 12),
        // Expanded, não Flexible + Spacer: os dois dividiriam o espaço meio a meio e o detalhe
        // saía cortado mesmo com a barra larga.
        Expanded(child: Text(detalhe ?? '', style: TextStyle(color: cores.textoApagado), overflow: TextOverflow.ellipsis)),
        const SizedBox(width: 16),
        ...acoes,
      ]),
    );
  }
}
