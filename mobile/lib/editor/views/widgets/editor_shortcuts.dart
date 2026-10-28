// Kanso — editor_shortcuts.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart' show EditCommand;
import '../../models/data/editor_intent.dart';
import '../../utils/editor_shortcut_bindings.dart';
import '../../utils/enabled_when_action.dart';
import '../controllers/kanso_markdown_controller.dart';

/// The keyboard shortcuts of the note's editor (see
/// [editorShortcutBindings]) around [child], which holds the field.
///
/// The keys that move through the completions are only taken while they are
/// open ([completionOpen]); the rest of the time they go on to the field.
class EditorShortcuts extends StatelessWidget {
  const EditorShortcuts({
    required this.controller,
    required this.completionOpen,
    required this.onSave,
    required this.onDismissCompletion,
    required this.onEscape,
    required this.child,
    this.platform,
    super.key,
  });

  final KansoMarkdownController controller;

  /// Whether completions are being offered now.
  final ValueGetter<bool> completionOpen;

  /// Save now.
  final VoidCallback onSave;

  /// Escape, with completions open: close them.
  final VoidCallback onDismissCompletion;

  /// Escape, with no completions to close: leave for Preview.
  final VoidCallback onEscape;

  final Widget child;

  /// The platform whose keys apply; the device's when null.
  final TargetPlatform? platform;

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: editorShortcutBindings(platform ?? defaultTargetPlatform),
      child: Actions(
        actions: <Type, Action<Intent>>{
          FormatIntent: CallbackAction<FormatIntent>(
            onInvoke: (intent) {
              controller.run(EditCommand.format(format: intent.format));
              return null;
            },
          ),
          SaveIntent: CallbackAction<SaveIntent>(
            onInvoke: (_) {
              onSave();
              return null;
            },
          ),
          TabIntent: CallbackAction<TabIntent>(
            onInvoke: (intent) {
              controller.run(EditCommand.tab(out: intent.out));
              return null;
            },
          ),
          EscapeIntent: CallbackAction<EscapeIntent>(
            onInvoke: (_) {
              if (completionOpen()) {
                onDismissCompletion();
              } else {
                onEscape();
              }
              return null;
            },
          ),
          CompletionStepIntent: EnabledWhenAction<CompletionStepIntent>(
            enabled: completionOpen,
            onInvoke: (intent) {
              controller.stepCompletion(intent.delta);
              return null;
            },
          ),
          CompletionChooseIntent: EnabledWhenAction<CompletionChooseIntent>(
            enabled: completionOpen,
            onInvoke: (_) {
              controller.chooseCompletion();
              return null;
            },
          ),
        },
        child: child,
      ),
    );
  }
}
