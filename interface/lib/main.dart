import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'casca.dart';
import 'src/rust/api.dart';
import 'src/rust/frb_generated.dart';
import 'tema.dart';
import 'widgets/erros.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  await windowManager.ensureInitialized();
  // O nome vem do núcleo (nucleo/src/nome.rs): é a única constante com o nome do projeto.
  final nome = nomeDoApp();
  await windowManager.setTitle(nome);
  await windowManager.setMinimumSize(const Size(900, 600));
  final config = configuracao();
  runApp(App(
    nome: nome,
    nomeCompleto: nomeCompleto(),
    modoInicial: ModoDeTema.deId(config.tema),
    aviso: config.problema,
  ));
}

/// O app: dois temas (claro e escuro) e o modo escolhido pelo usuário, guardado pelo núcleo
/// em `<raiz>/configuracao.toml` (RF-06).
class App extends StatefulWidget {
  const App({super.key, required this.nome, required this.nomeCompleto, required this.modoInicial, this.aviso});
  final String nome;
  final String nomeCompleto;
  final ModoDeTema modoInicial;
  final String? aviso;

  @override
  State<App> createState() => _AppState();
}

class _AppState extends State<App> {
  late ModoDeTema _modo = widget.modoInicial;
  late String? _aviso = widget.aviso;

  /// Muda a interface na hora e grava; se gravar falhar, a interface fica trocada e avisa.
  Future<void> _trocarModo(ModoDeTema modo) async {
    setState(() {
      _modo = modo;
      _aviso = null;
    });
    try {
      await definirTema(tema: modo.id);
    } catch (e) {
      setState(() => _aviso = 'A aparência mudou, mas não consegui guardar a escolha: ${textoDoErro(e)}');
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: widget.nome,
      debugShowCheckedModeBanner: false,
      theme: temaClaro(),
      darkTheme: temaEscuro(),
      themeMode: _modo.modo,
      home: Casca(nome: widget.nome, nomeCompleto: widget.nomeCompleto, modo: _modo, aoTrocarModo: _trocarModo, aviso: _aviso),
    );
  }
}
