import 'package:flutter/material.dart';

/// Uma faixa de aviso ou erro no topo de uma seção. A cor diz a gravidade; vem do tema.
class Faixa extends StatelessWidget {
  const Faixa({super.key, required this.texto, required this.cor, this.icone = Icons.warning_amber_rounded});
  final String texto;
  final Color cor;
  final IconData icone;

  @override
  Widget build(BuildContext context) {
    return Container(
      color: cor.withValues(alpha: 0.15),
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 10),
      child: Row(children: [
        Icon(icone, color: cor, size: 20),
        const SizedBox(width: 10),
        Expanded(child: SelectableText(texto, style: TextStyle(color: cor))),
      ]),
    );
  }
}
