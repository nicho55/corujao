import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';

import 'src/rust/api.dart';
import 'tema.dart';
import 'widgets/erros.dart';

/// Página de um jogo: capa, ficha, o botão JOGAR e o controle do processo (fechar por PID,
/// com SIGTERM e espera; forçar só depois de esperar).
class JogoPagina extends StatefulWidget {
  const JogoPagina({super.key, required this.jogo});
  final JogoInfo jogo;

  @override
  State<JogoPagina> createState() => _JogoPaginaState();
}

class _JogoPaginaState extends State<JogoPagina> {
  Situacao _situacao = Situacao(situacao: 'parado');
  String _registro = '';
  String _mensagem = '';
  DateTime? _fecharPedidoEm;
  bool _ocupado = false;
  Timer? _relogio;

  static const _esperaAntesDeForcar = Duration(seconds: 12);

  JogoInfo get jogo => widget.jogo;
  bool get rodando => _situacao.situacao == 'rodando';
  bool get podeForcar => rodando && _fecharPedidoEm != null && DateTime.now().difference(_fecharPedidoEm!) > _esperaAntesDeForcar;

  @override
  void initState() {
    super.initState();
    _atualizar();
    _relogio = Timer.periodic(const Duration(seconds: 1), (_) => _atualizar());
  }

  @override
  void dispose() {
    _relogio?.cancel();
    super.dispose();
  }

  void _atualizar() {
    if (!mounted) return;
    setState(() {
      _situacao = estado(id: jogo.id);
      _registro = registro(id: jogo.id, linhas: 40);
      if (!rodando) _fecharPedidoEm = null;
    });
  }

  Future<void> _executar(Future<String> Function() acao) async {
    setState(() => _ocupado = true);
    try {
      final resultado = await acao();
      setState(() => _mensagem = resultado);
    } catch (e) {
      setState(() => _mensagem = 'Erro: ${textoDoErro(e)}');
    } finally {
      setState(() => _ocupado = false);
      _atualizar();
    }
  }

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Scaffold(
      body: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Container(
          color: cores.painelEscuro,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(children: [
            IconButton(onPressed: () => Navigator.of(context).pop(), icon: const Icon(Icons.arrow_back), tooltip: 'Voltar à biblioteca'),
            const SizedBox(width: 8),
            Text(jogo.nome, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
          ]),
        ),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(28),
            child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              SizedBox(width: 280, child: _capa()),
              const SizedBox(width: 32),
              Expanded(child: _ficha()),
            ]),
          ),
        ),
      ]),
    );
  }

  Widget _capa() {
    final cores = context.cores;
    return AspectRatio(
      aspectRatio: 2 / 3,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(6),
        child: jogo.capa != null
            ? Image.file(File(jogo.capa!), fit: BoxFit.cover)
            : Container(color: cores.painelEscuro, alignment: Alignment.center, child: Text(jogo.nome, textAlign: TextAlign.center)),
      ),
    );
  }

  Widget _ficha() {
    final cores = context.cores;
    final subtitulo = [jogo.projeto, jogo.console].whereType<String>().join(' · ');
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text(jogo.nome, style: const TextStyle(fontSize: 30, fontWeight: FontWeight.w700)),
      if (subtitulo.isNotEmpty) Text(subtitulo, style: TextStyle(color: cores.textoApagado, fontSize: 15)),
      const SizedBox(height: 6),
      _cobertura(),
      const SizedBox(height: 10),
      Wrap(spacing: 6, children: [for (final t in jogo.tags) Chip(label: Text(t), visualDensity: VisualDensity.compact)]),
      const SizedBox(height: 20),
      _controles(),
      const SizedBox(height: 10),
      _linhaDeSituacao(),
      if (_mensagem.isNotEmpty) Padding(padding: const EdgeInsets.only(top: 6), child: Text(_mensagem, style: TextStyle(color: cores.textoApagado))),
      const SizedBox(height: 24),
      if (jogo.descricao != null) Text(jogo.descricao!, style: const TextStyle(fontSize: 15, height: 1.5)),
      const SizedBox(height: 24),
      _detalhes(),
      const SizedBox(height: 24),
      _registroDeExecucao(),
    ]);
  }

  /// Qual cópia do jogo esta recomp cobre (RF-05): sempre dito, mesmo quando não informado.
  Widget _cobertura() {
    final cores = context.cores;
    return Row(children: [
      Icon(Icons.album_outlined, size: 16, color: cores.textoApagado),
      const SizedBox(width: 6),
      Text('Cobre: ', style: TextStyle(color: cores.textoApagado, fontSize: 14)),
      Flexible(
        child: Text(
          jogo.disco,
          style: jogo.discoInformado ? const TextStyle(fontSize: 14) : TextStyle(fontSize: 14, fontStyle: FontStyle.italic, color: cores.textoApagado),
        ),
      ),
    ]);
  }

  Widget _controles() {
    final cores = context.cores;
    final podeJogar = !rodando && !_ocupado && jogo.executavelExiste && jogo.receitaExiste;
    return Row(children: [
      SizedBox(
        height: 52,
        child: ElevatedButton.icon(
          onPressed: podeJogar ? () => _executar(() async => 'Aberto (pid ${await jogar(id: jogo.id)}).') : null,
          icon: Icon(rodando ? Icons.videogame_asset : Icons.play_arrow, size: 28),
          label: Text(rodando ? 'RODANDO' : 'JOGAR', style: const TextStyle(fontSize: 19, fontWeight: FontWeight.w700, letterSpacing: 1)),
          style: ElevatedButton.styleFrom(
            backgroundColor: cores.jogar,
            disabledBackgroundColor: rodando ? cores.jogar.withValues(alpha: 0.5) : cores.painel,
            foregroundColor: cores.sobreJogar,
            disabledForegroundColor: cores.sobreJogar.withValues(alpha: 0.7),
            padding: const EdgeInsets.symmetric(horizontal: 28),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
          ),
        ),
      ),
      const SizedBox(width: 12),
      if (rodando)
        SizedBox(
          height: 52,
          child: OutlinedButton.icon(
            onPressed: _ocupado
                ? null
                : () {
                    _fecharPedidoEm ??= DateTime.now();
                    _executar(() => fechar(id: jogo.id));
                  },
            icon: const Icon(Icons.close),
            label: const Text('Fechar'),
            style: OutlinedButton.styleFrom(foregroundColor: cores.texto, side: BorderSide(color: cores.textoApagado)),
          ),
        ),
      if (podeForcar) ...[
        const SizedBox(width: 12),
        SizedBox(
          height: 52,
          child: OutlinedButton.icon(
            onPressed: _ocupado ? null : () => _executar(() => forcar(id: jogo.id)),
            icon: const Icon(Icons.dangerous_outlined),
            label: const Text('Forçar fechamento'),
            style: OutlinedButton.styleFrom(foregroundColor: cores.erro, side: BorderSide(color: cores.erro)),
          ),
        ),
      ],
    ]);
  }

  Widget _linhaDeSituacao() {
    final cores = context.cores;
    final s = _situacao;
    String texto;
    Color cor;
    switch (s.situacao) {
      case 'rodando':
        final seg = s.segundos ?? 0;
        final tempo = seg >= 60 ? '${seg ~/ 60} min ${seg % 60} s' : '$seg s';
        texto = 'Rodando há $tempo (pid ${s.pid}).';
        if (_fecharPedidoEm != null) {
          texto += podeForcar
              ? ' O jogo não respondeu ao pedido de fechar; se precisar, force (ele abrirá em modo seguro da próxima vez).'
              : ' Pedido de fechar enviado; esperando o jogo encerrar limpo…';
        }
        cor = cores.jogarClaro;
      case 'encerrado':
        texto = s.codigo == 0 ? 'Encerrou limpo.' : 'Encerrou com código ${s.codigo}. Veja o registro abaixo.';
        cor = s.codigo == 0 ? cores.textoApagado : cores.alerta;
      default:
        if (!jogo.executavelExiste) {
          texto = 'Executável não encontrado: ${jogo.executavel}';
          cor = cores.erro;
        } else if (!jogo.receitaExiste) {
          texto = 'A receita "${jogo.receita}" não existe em maquina.toml (ver detalhes).';
          cor = cores.erro;
        } else {
          texto = 'Pronto para jogar.';
          cor = cores.textoApagado;
        }
    }
    return Row(children: [
      Icon(Icons.circle, size: 10, color: cor),
      const SizedBox(width: 8),
      Expanded(child: Text(texto, style: TextStyle(color: cor))),
    ]);
  }

  Widget _detalhes() {
    String linha;
    try {
      linha = linhaDeComando(id: jogo.id);
    } catch (e) {
      linha = textoDoErro(e);
    }
    return Theme(
      data: Theme.of(context).copyWith(dividerColor: Colors.transparent),
      child: ExpansionTile(
        title: const Text('Detalhes de execução', style: TextStyle(fontSize: 15)),
        tilePadding: EdgeInsets.zero,
        childrenPadding: const EdgeInsets.only(bottom: 8),
        children: [
          _Linha('Executável', jogo.executavel, alerta: !jogo.executavelExiste),
          _Linha('Receita da máquina', jogo.receita ?? '(nenhuma)', alerta: !jogo.receitaExiste),
          _Linha('Linha de comando', linha, mono: true),
          _Linha('Disco coberto', jogo.disco, alerta: !jogo.discoInformado),
          if (jogo.saves != null) _Linha('Dados do jogador', '${jogo.saves} (o launcher nunca mexe aqui)'),
          _Linha('Pasta na biblioteca', jogo.pasta),
        ],
      ),
    );
  }

  Widget _registroDeExecucao() {
    final cores = context.cores;
    if (_registro.isEmpty) return const SizedBox.shrink();
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      const Text('Registro da execução', style: TextStyle(fontSize: 15, fontWeight: FontWeight.w600)),
      const SizedBox(height: 8),
      Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(color: cores.painelEscuro, borderRadius: BorderRadius.circular(4)),
        child: SelectableText(_registro, style: TextStyle(fontFamily: 'monospace', fontSize: 12, color: cores.textoApagado)),
      ),
    ]);
  }
}

class _Linha extends StatelessWidget {
  const _Linha(this.rotulo, this.valor, {this.alerta = false, this.mono = false});
  final String rotulo;
  final String valor;
  final bool alerta;
  final bool mono;

  @override
  Widget build(BuildContext context) {
    final cores = context.cores;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(width: 170, child: Text(rotulo, style: TextStyle(color: cores.textoApagado))),
        Expanded(
          child: SelectableText(
            valor,
            style: TextStyle(color: alerta ? cores.alerta : cores.texto, fontFamily: mono ? 'monospace' : null, fontSize: mono ? 12 : 14),
          ),
        ),
      ]),
    );
  }
}
