import 'package:flutter/material.dart';

/// Os papéis de cor da interface (RF-06). **Componente nenhum escreve cor**: todos leem daqui,
/// por `context.cores`, e o seletor troca o tema inteiro. Cada tema é uma instância desta
/// classe; paletas nomeadas e tema do usuário entram como novas instâncias, sem tocar nas telas.
class CoresDoPog extends ThemeExtension<CoresDoPog> {
  const CoresDoPog({
    required this.fundo,
    required this.painel,
    required this.painelEscuro,
    required this.destaque,
    required this.texto,
    required this.textoApagado,
    required this.jogar,
    required this.jogarClaro,
    required this.sobreJogar,
    required this.alerta,
    required this.erro,
  });

  /// Fundo da janela.
  final Color fundo;
  /// Cartões e painéis, um tom acima do fundo.
  final Color painel;
  /// Cabeçalho e áreas rebaixadas (registro, capa sem imagem).
  final Color painelEscuro;
  /// Links, aba ativa, borda de seleção.
  final Color destaque;
  final Color texto;
  final Color textoApagado;
  /// O botão JOGAR; `jogarClaro` é o estado "rodando" e `sobreJogar` o texto em cima dele.
  final Color jogar;
  final Color jogarClaro;
  final Color sobreJogar;
  final Color alerta;
  final Color erro;

  /// No espírito da Steam: azul-escuro, painéis um tom acima, destaque azul-claro, verde no jogar.
  /// Contraste medido (WCAG): texto ≥ 4,5:1 sobre fundo, painel e cabeçalho; botão JOGAR ≥ 3:1.
  static const escura = CoresDoPog(
    fundo: Color(0xFF1B2838),
    painel: Color(0xFF2A475E),
    painelEscuro: Color(0xFF171D25),
    destaque: Color(0xFF66C0F4),
    texto: Color(0xFFC6D4DF),
    textoApagado: Color(0xFFA9B4BD),
    jogar: Color(0xFF5BA32B),
    jogarClaro: Color(0xFF8ED629),
    sobreJogar: Color(0xFFFFFFFF),
    alerta: Color(0xFFE0A23C),
    erro: Color(0xFFEE7676),
  );

  /// A mesma estrutura em fundo claro. Contraste conferido: texto ≥ 4,5:1, botão ≥ 3:1.
  static const clara = CoresDoPog(
    fundo: Color(0xFFF2F4F7),
    painel: Color(0xFFFFFFFF),
    painelEscuro: Color(0xFFE3E8EE),
    destaque: Color(0xFF1565A0),
    texto: Color(0xFF1B2838),
    textoApagado: Color(0xFF5A6673),
    jogar: Color(0xFF3E8A1E),
    jogarClaro: Color(0xFF2E7A12),
    sobreJogar: Color(0xFFFFFFFF),
    alerta: Color(0xFF9A6200),
    erro: Color(0xFFB3261E),
  );

  @override
  CoresDoPog copyWith({
    Color? fundo,
    Color? painel,
    Color? painelEscuro,
    Color? destaque,
    Color? texto,
    Color? textoApagado,
    Color? jogar,
    Color? jogarClaro,
    Color? sobreJogar,
    Color? alerta,
    Color? erro,
  }) {
    return CoresDoPog(
      fundo: fundo ?? this.fundo,
      painel: painel ?? this.painel,
      painelEscuro: painelEscuro ?? this.painelEscuro,
      destaque: destaque ?? this.destaque,
      texto: texto ?? this.texto,
      textoApagado: textoApagado ?? this.textoApagado,
      jogar: jogar ?? this.jogar,
      jogarClaro: jogarClaro ?? this.jogarClaro,
      sobreJogar: sobreJogar ?? this.sobreJogar,
      alerta: alerta ?? this.alerta,
      erro: erro ?? this.erro,
    );
  }

  @override
  CoresDoPog lerp(CoresDoPog? outro, double t) {
    if (outro == null) return this;
    return CoresDoPog(
      fundo: Color.lerp(fundo, outro.fundo, t)!,
      painel: Color.lerp(painel, outro.painel, t)!,
      painelEscuro: Color.lerp(painelEscuro, outro.painelEscuro, t)!,
      destaque: Color.lerp(destaque, outro.destaque, t)!,
      texto: Color.lerp(texto, outro.texto, t)!,
      textoApagado: Color.lerp(textoApagado, outro.textoApagado, t)!,
      jogar: Color.lerp(jogar, outro.jogar, t)!,
      jogarClaro: Color.lerp(jogarClaro, outro.jogarClaro, t)!,
      sobreJogar: Color.lerp(sobreJogar, outro.sobreJogar, t)!,
      alerta: Color.lerp(alerta, outro.alerta, t)!,
      erro: Color.lerp(erro, outro.erro, t)!,
    );
  }
}

/// `context.cores.destaque` em qualquer widget.
extension CoresDoContexto on BuildContext {
  CoresDoPog get cores => Theme.of(this).extension<CoresDoPog>() ?? CoresDoPog.escura;
}

/// As escolhas do seletor (RF-06): o `id` é o mesmo que o núcleo grava em `configuracao.toml`.
enum ModoDeTema {
  sistema('sistema', 'Seguir o sistema', ThemeMode.system, Icons.brightness_auto),
  claro('claro', 'Claro', ThemeMode.light, Icons.light_mode),
  escuro('escuro', 'Escuro', ThemeMode.dark, Icons.dark_mode);

  const ModoDeTema(this.id, this.rotulo, this.modo, this.icone);
  final String id;
  final String rotulo;
  final ThemeMode modo;
  final IconData icone;

  /// Id desconhecido cai no padrão do núcleo (escuro, a cara de Steam com que o launcher nasceu).
  static ModoDeTema deId(String id) => values.firstWhere((m) => m.id == id, orElse: () => escuro);
}

ThemeData _construir(CoresDoPog c, Brightness brilho) {
  final base = brilho == Brightness.dark ? ThemeData.dark(useMaterial3: true) : ThemeData.light(useMaterial3: true);
  return base.copyWith(
    scaffoldBackgroundColor: c.fundo,
    colorScheme: base.colorScheme.copyWith(
      primary: c.destaque,
      surface: c.painel,
      onSurface: c.texto,
      onSurfaceVariant: c.textoApagado,
      error: c.erro,
    ),
    textTheme: base.textTheme.apply(bodyColor: c.texto, displayColor: c.texto),
    cardTheme: CardThemeData(color: c.painel, margin: EdgeInsets.zero),
    chipTheme: base.chipTheme.copyWith(backgroundColor: c.painel, side: BorderSide.none, labelStyle: TextStyle(color: c.texto)),
    snackBarTheme: SnackBarThemeData(backgroundColor: c.painelEscuro, contentTextStyle: TextStyle(color: c.texto)),
    popupMenuTheme: PopupMenuThemeData(color: c.painel, textStyle: TextStyle(color: c.texto)),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: c.painelEscuro,
      hintStyle: TextStyle(color: c.textoApagado),
      border: OutlineInputBorder(borderRadius: BorderRadius.circular(4), borderSide: BorderSide.none),
      isDense: true,
    ),
    dividerColor: c.painel,
    extensions: [c],
  );
}

ThemeData temaEscuro() => _construir(CoresDoPog.escura, Brightness.dark);
ThemeData temaClaro() => _construir(CoresDoPog.clara, Brightness.light);
