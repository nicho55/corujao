String _dois(int n) => n.toString().padLeft(2, '0');

/// `22/10 11:12`, para carimbos de "atualizado em".
String dataCurta(DateTime d) => '${_dois(d.day)}/${_dois(d.month)} ${_dois(d.hour)}:${_dois(d.minute)}';

/// `3 jogos`, `1 recomp`.
String contagem(int n, String singular, String plural) => '$n ${n == 1 ? singular : plural}';
