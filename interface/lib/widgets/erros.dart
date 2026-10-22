/// `AnyhowException(mensagem)` vira só a mensagem; o resto passa como veio.
String textoDoErro(Object e) {
  final s = e.toString();
  final m = RegExp(r'^\w+Exception\((.*)\)$', dotAll: true).firstMatch(s);
  return m != null ? m.group(1)! : s;
}
