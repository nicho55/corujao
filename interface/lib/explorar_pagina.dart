import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import 'src/rust/api.dart';
import 'tema.dart';
import 'widgets/barra_da_secao.dart';
import 'widgets/erros.dart';
import 'widgets/etiqueta.dart';
import 'widgets/faixa.dart';
import 'widgets/formato.dart';

/// A aba Explorar: o catálogo de recomps com o que cada repositório É (que jogo, de que
/// console, recompilação ou porte) e qual cópia do jogo cobre (RF-05). Lê do disco; a rede
/// só entra no botão "Atualizar pela rede".
class ExplorarPagina extends StatefulWidget {
  const ExplorarPagina({super.key});

  @override
  State<ExplorarPagina> createState() => _ExplorarPaginaState();
}

class _ExplorarPaginaState extends State<ExplorarPagina> {
  late CatalogoInfo _catalogo;
  final _busca = TextEditingController();
  String _tipo = '';
  bool _soComAtualizacao = false;
  bool _atualizando = false;
  String _mensagem = '';

  @override
  void initState() {
    super.initState();
    _catalogo = catalogo();
  }

  @override
  void dispose() {
    _busca.dispose();
    super.dispose();
  }

  void _recarregar() => setState(() => _catalogo = catalogo());

  Future<void> _atualizarPelaRede() async {
    setState(() {
      _atualizando = true;
      _mensagem = '';
    });
    try {
      final resultado = await atualizarCatalogo();
      if (!mounted) return;
      setState(() => _mensagem = resultado);
      _recarregar();
    } catch (e) {
      if (mounted) setState(() => _mensagem = 'Não deu para atualizar: ${textoDoErro(e)}');
    } finally {
      if (mounted) setState(() => _atualizando = false);
    }
  }

  /// Os tipos presentes no catálogo, id → rótulo, para o filtro.
  Map<String, String> get _tipos {
    final m = <String, String>{};
    for (final e in _catalogo.entradas) {
      m[e.tipo] = e.tipoRotulo;
    }
    return m;
  }

  List<EntradaDoCatalogo> get _filtradas {
    final termo = _busca.text.trim().toLowerCase();
    return [
      for (final e in _catalogo.entradas)
        if ((_tipo.isEmpty || e.tipo == _tipo) && (!_soComAtualizacao || e.temAtualizacao) && (termo.isEmpty || _casa(e, termo))) e,
    ];
  }

  static bool _casa(EntradaDoCatalogo e, String termo) =>
      e.jogo.toLowerCase().contains(termo) ||
      e.projeto.toLowerCase().contains(termo) ||
      e.repositorio.toLowerCase().contains(termo) ||
      e.slug.contains(termo) ||
      e.consoles.any((c) => c.toLowerCase().contains(termo)) ||
      e.tags.any((t) => t.toLowerCase().contains(termo));

  String get _detalhe {
    final partes = [contagem(_catalogo.entradas.length, 'recomp', 'recomps'), _catalogo.fonte];
    final em = _catalogo.atualizadoEm;
    if (em != null) partes.add('atualizado ${dataCurta(DateTime.fromMillisecondsSinceEpoch(em.toInt() * 1000))}');
    return partes.join(' · ');
  }

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    final lista = _filtradas;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      BarraDaSecao(
        titulo: 'EXPLORAR',
        detalhe: _detalhe,
        acoes: [
          IconButton(onPressed: _recarregar, tooltip: 'Reler o catálogo do disco', icon: const Icon(Icons.refresh)),
          const SizedBox(width: 4),
          Tooltip(
            message: 'Baixa o catálogo de quiverlauncher.com uma vez (descrição, console, tipo de cada recomp) e guarda em '
                '${_catalogo.arquivoBaixado ?? '<raiz>/catalogo/'}. Nada sai da sua máquina além do pedido.',
            child: FilledButton.tonalIcon(
              onPressed: _atualizando ? null : _atualizarPelaRede,
              icon: _atualizando
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
                  : const Icon(Icons.cloud_download_outlined, size: 18),
              label: Text(_atualizando ? 'Baixando…' : 'Atualizar pela rede'),
            ),
          ),
        ],
      ),
      for (final p in _catalogo.problemas) Faixa(texto: p, cor: cores.alerta),
      if (_mensagem.isNotEmpty) Faixa(texto: _mensagem, cor: _mensagem.startsWith('Não deu') ? cores.erro : cores.destaque, icone: Icons.info_outline),
      _Filtros(
        busca: _busca,
        tipo: _tipo,
        tipos: _tipos,
        soComAtualizacao: _soComAtualizacao,
        aoMudar: () => setState(() {}),
        aoEscolherTipo: (t) => setState(() => _tipo = t),
        aoAlternarAtualizacao: (v) => setState(() => _soComAtualizacao = v),
      ),
      Expanded(
        child: lista.isEmpty
            ? _Vazio(temCatalogo: _catalogo.entradas.isNotEmpty)
            : ListView.builder(
                padding: const EdgeInsets.fromLTRB(24, 4, 24, 24),
                itemCount: lista.length,
                itemBuilder: (_, i) => Padding(padding: const EdgeInsets.only(bottom: 10), child: _CartaoDaEntrada(entrada: lista[i])),
              ),
      ),
    ]);
  }
}

class _Filtros extends StatelessWidget {
  const _Filtros({
    required this.busca,
    required this.tipo,
    required this.tipos,
    required this.soComAtualizacao,
    required this.aoMudar,
    required this.aoEscolherTipo,
    required this.aoAlternarAtualizacao,
  });
  final TextEditingController busca;
  final String tipo;
  final Map<String, String> tipos;
  final bool soComAtualizacao;
  final VoidCallback aoMudar;
  final ValueChanged<String> aoEscolherTipo;
  final ValueChanged<bool> aoAlternarAtualizacao;

  @override
  Widget build(BuildContext context) {
    final chaves = tipos.keys.toList()..sort((a, b) => tipos[a]!.compareTo(tipos[b]!));
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 4, 24, 10),
      child: Row(children: [
        Expanded(
          child: TextField(
            controller: busca,
            onChanged: (_) => aoMudar(),
            decoration: InputDecoration(
              hintText: 'Buscar por jogo, projeto, repositório, console ou tag',
              prefixIcon: const Icon(Icons.search, size: 20),
              suffixIcon: busca.text.isEmpty
                  ? null
                  : IconButton(
                      icon: const Icon(Icons.clear, size: 18),
                      onPressed: () {
                        busca.clear();
                        aoMudar();
                      }),
            ),
          ),
        ),
        const SizedBox(width: 12),
        DropdownButton<String>(
          value: tipo,
          underline: const SizedBox.shrink(),
          items: [
            const DropdownMenuItem(value: '', child: Text('Todos os tipos')),
            for (final id in chaves) DropdownMenuItem(value: id, child: Text(tipos[id]!)),
          ],
          onChanged: (v) => aoEscolherTipo(v ?? ''),
        ),
        const SizedBox(width: 12),
        FilterChip(label: const Text('Só com atualização'), selected: soComAtualizacao, onSelected: aoAlternarAtualizacao),
      ]),
    );
  }
}

class _Vazio extends StatelessWidget {
  const _Vazio({required this.temCatalogo});
  final bool temCatalogo;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Center(
      child: Column(mainAxisSize: MainAxisSize.min, children: [
        Icon(Icons.explore_off_outlined, size: 64, color: cores.textoApagado),
        const SizedBox(height: 16),
        Text(temCatalogo ? 'Nada casa com a busca.' : 'Nenhum catálogo no disco.', style: const TextStyle(fontSize: 18)),
        const SizedBox(height: 8),
        Text(
          temCatalogo ? 'Tente outro termo ou limpe os filtros.' : 'Use "Atualizar pela rede" para baixar o catálogo do quiverlauncher.com.',
          style: TextStyle(color: cores.textoApagado),
        ),
      ]),
    );
  }
}

IconData _iconeDoTipo(String tipo) => switch (tipo) {
      'recompilacao' => Icons.memory,
      'descompilacao' => Icons.code,
      'porte' => Icons.devices,
      'reimplementacao' => Icons.settings_suggest_outlined,
      _ => Icons.help_outline,
    };

/// Um cartão do catálogo: jogo, console, o que o repositório faz, qual cópia cobre, versões.
class _CartaoDaEntrada extends StatelessWidget {
  const _CartaoDaEntrada({required this.entrada});
  final EntradaDoCatalogo entrada;

  String get _versoes {
    final e = entrada;
    if (e.versaoVerificada == null && e.versaoUpstream == null) return 'versão não informada';
    if (e.temAtualizacao) return 'verificada ${e.versaoVerificada} · upstream já em ${e.versaoUpstream}';
    return 'versão ${e.versaoVerificada ?? e.versaoUpstream}';
  }

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    final e = entrada;
    final apagado = TextStyle(color: cores.textoApagado, fontSize: 13);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          CircleAvatar(backgroundColor: cores.painelEscuro, radius: 22, child: Icon(_iconeDoTipo(e.tipo), color: cores.destaque)),
          const SizedBox(width: 16),
          Expanded(
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Wrap(crossAxisAlignment: WrapCrossAlignment.center, spacing: 8, runSpacing: 4, children: [
                Text(e.jogo, style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700)),
                if (e.jogoInferido)
                  Tooltip(
                    message: 'Nome deduzido do nome da pasta; nenhuma fonte o confirmou. "Atualizar pela rede" traz o nome certo.',
                    child: Icon(Icons.help_outline, size: 16, color: cores.textoApagado),
                  ),
                for (final c in e.consoles) Etiqueta(c, cor: cores.painelEscuro, sobre: cores.texto),
                if (e.temAtualizacao) Etiqueta('ATUALIZAÇÃO', cor: cores.alerta),
              ]),
              const SizedBox(height: 4),
              Text('${e.projeto} · ${e.tipoRotulo}${e.tipoInferido ? ' (inferido pelo nome)' : ''}', style: apagado),
              const SizedBox(height: 8),
              Text.rich(
                TextSpan(children: [
                  if (e.descricao == null) TextSpan(text: 'Sem descrição na fonte. Pelo tipo: ', style: TextStyle(color: cores.textoApagado, fontStyle: FontStyle.italic)),
                  TextSpan(text: e.descricao ?? e.tipoExplicacao),
                ]),
                style: const TextStyle(fontSize: 14, height: 1.4),
              ),
              const SizedBox(height: 8),
              Row(children: [
                Icon(Icons.album_outlined, size: 16, color: cores.textoApagado),
                const SizedBox(width: 6),
                Text('Cobre: ', style: apagado),
                Flexible(
                  child: Text(
                    e.disco,
                    style: e.discoInformado ? const TextStyle(fontSize: 13) : TextStyle(fontSize: 13, fontStyle: FontStyle.italic, color: cores.textoApagado),
                  ),
                ),
              ]),
              const SizedBox(height: 4),
              Row(children: [
                Icon(e.temAtualizacao ? Icons.new_releases_outlined : Icons.verified_outlined, size: 16, color: e.temAtualizacao ? cores.alerta : cores.textoApagado),
                const SizedBox(width: 6),
                Text(_versoes, style: apagado),
              ]),
            ]),
          ),
          const SizedBox(width: 16),
          Column(crossAxisAlignment: CrossAxisAlignment.end, children: [
            if (e.url.startsWith('http'))
              OutlinedButton.icon(
                onPressed: () => launchUrl(Uri.parse(e.url), mode: LaunchMode.externalApplication),
                icon: const Icon(Icons.open_in_new, size: 16),
                label: const Text('Abrir repositório'),
              )
            else
              Text('repositório não informado', style: apagado.copyWith(fontStyle: FontStyle.italic)),
            const SizedBox(height: 6),
            if (e.repositorio.isNotEmpty) SelectableText('${e.repositorio} (${e.provedor})', style: apagado.copyWith(fontSize: 12)),
          ]),
        ]),
      ),
    );
  }
}
