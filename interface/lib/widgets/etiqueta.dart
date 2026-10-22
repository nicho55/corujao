import 'package:flutter/material.dart';

/// Um rótulo pequeno sobre fundo colorido: console numa capa, "não instalado", "atualização".
class Etiqueta extends StatelessWidget {
  const Etiqueta(this.texto, {super.key, required this.cor, this.sobre});
  final String texto;
  final Color cor;
  /// Cor do texto; sem ela, o tema decide pelo brilho do fundo.
  final Color? sobre;

  @override
  Widget build(BuildContext context) {
    final texto0 = sobre ?? (ThemeData.estimateBrightnessForColor(cor) == Brightness.dark ? Colors.white : Colors.black);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(color: cor.withValues(alpha: 0.9), borderRadius: BorderRadius.circular(3)),
      child: Text(texto, style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: texto0)),
    );
  }
}
