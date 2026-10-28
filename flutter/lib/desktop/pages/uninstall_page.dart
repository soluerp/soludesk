import 'package:flutter/material.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:flutter_hbb/models/state_model.dart';
import 'package:window_manager/window_manager.dart';

class UninstallPage extends StatefulWidget {
  const UninstallPage({Key? key}) : super(key: key);

  @override
  State<UninstallPage> createState() => _UninstallPageState();
}

class _UninstallPageState extends State<UninstallPage> with WindowListener {
  bool _isUninstalling = false;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
    windowManager.setPreventClose(true);
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    super.dispose();
  }

  @override
  void onWindowClose() {
    if (!_isUninstalling) {
      windowManager.setPreventClose(false);
      windowManager.close();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return DragToResizeArea(
      resizeEdgeSize: stateGlobal.resizeEdgeSize.value,
      enableResizeEdges: windowManagerEnableResizeEdges,
      child: Scaffold(
        backgroundColor: theme.colorScheme.surface,
        body: Padding(
          padding: const EdgeInsets.all(28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Image.asset('assets/icon.png', width: 42, height: 42),
                  const SizedBox(width: 12),
                  Text(
                    'SOLUDESK',
                    style: theme.textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.w800,
                      letterSpacing: 0.5,
                    ),
                  ),
                ],
              ),
              const Spacer(),
              Text('Desinstalar o SOLUDESK?',
                  style: theme.textTheme.headlineSmall),
              const SizedBox(height: 12),
              Text(
                'O aplicativo será removido deste computador, incluindo atalhos e o serviço local do SOLUDESK. Seus arquivos pessoais não serão removidos.',
                style: theme.textTheme.bodyLarge,
              ),
              const Spacer(),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: _isUninstalling
                        ? null
                        : () {
                            windowManager.setPreventClose(false);
                            windowManager.close();
                          },
                    child: const Text('Cancelar'),
                  ),
                  const SizedBox(width: 10),
                  FilledButton.icon(
                    style: FilledButton.styleFrom(
                      backgroundColor: theme.colorScheme.error,
                      foregroundColor: theme.colorScheme.onError,
                    ),
                    onPressed: _isUninstalling
                        ? null
                        : () {
                            setState(() => _isUninstalling = true);
                            bind.installUninstallMe();
                          },
                    icon: _isUninstalling
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.delete_outline),
                    label: Text(
                        _isUninstalling ? 'Desinstalando…' : 'Desinstalar'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
