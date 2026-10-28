// Kanso — window_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../../shell/utils/kanso_shortcut_bindings.dart'
    show ShortcutContext, shortcutBindingsFor;
import '../../models/data/window_intent.dart';
import 'note_window_scope.dart';

/// Answers the window intents (see [WindowIntent]) for the note [noteId]:
/// Close Note Window closes the window this engine draws (⌘W), Open in New
/// Window opens [noteId], or the note the intent names, in a window of its
/// own (⌘⇧O). The keys are bound by the shortcut catalog.
///
/// With no [NoteWindowScope] above, or a device that shows one window, the
/// intents are not enabled.
class WindowActions extends StatelessWidget {
  const WindowActions({required this.noteId, required this.child, super.key});

  /// The note the surface shows; null when it shows none.
  final String? noteId;

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final windows = NoteWindowScope.maybeOf(context);
    return Shortcuts(
      shortcuts: shortcutBindingsFor(
        const <ShortcutContext>[ShortcutContext.noteWindow],
        defaultTargetPlatform,
      ),
      child: Actions(
        actions: <Type, Action<Intent>>{
          CloseNoteWindowIntent: CallbackAction<CloseNoteWindowIntent>(
            onInvoke: (_) {
              if (windows != null) {
                unawaited(windows.closeThisWindow());
              }
              return null;
            },
          ),
          OpenInNewWindowIntent: _OpenInNewWindowAction(
            noteId: noteId,
            enabledWhen: windows?.supported ?? false,
            onOpen: (id) {
              if (windows != null) {
                unawaited(windows.open(id));
              }
            },
          ),
        },
        child: child,
      ),
    );
  }
}

class _OpenInNewWindowAction extends Action<OpenInNewWindowIntent> {
  _OpenInNewWindowAction({
    required this.noteId,
    required this.enabledWhen,
    required this.onOpen,
  });

  final String? noteId;
  final bool enabledWhen;
  final ValueChanged<String> onOpen;

  @override
  bool isEnabled(OpenInNewWindowIntent intent) =>
      enabledWhen && (intent.noteId ?? noteId) != null;

  @override
  Object? invoke(OpenInNewWindowIntent intent) {
    final id = intent.noteId ?? noteId;
    if (id != null) {
      onOpen(id);
    }
    return null;
  }
}
