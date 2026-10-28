// Kanso — note_mode_shortcuts.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../shell/models/data/kanso_intent.dart';
import '../../models/data/note_mode.dart';
import '../../state/note/note_bloc.dart';

/// Answers the shortcuts that switch the open note between reading and
/// editing (see `kansoShortcutBindings`), for the `NoteBloc` above it.
///
/// The shortcuts are found from the focus, so it takes the focus when the
/// note opens, unless a text field holds it (the editor's own, or the list's
/// search).
class NoteModeShortcuts extends StatefulWidget {
  const NoteModeShortcuts({required this.child, super.key});

  final Widget child;

  @override
  State<NoteModeShortcuts> createState() => _NoteModeShortcutsState();
}

class _NoteModeShortcutsState extends State<NoteModeShortcuts> {
  final FocusNode focusNode = FocusNode(
    debugLabel: 'Note',
    skipTraversal: true,
  );

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && !typingElsewhere()) {
        focusNode.requestFocus();
      }
    });
  }

  /// Whether a text field holds the focus, which is not to be taken from
  /// it: the editor's own, or the list's search.
  bool typingElsewhere() {
    final focused = FocusManager.instance.primaryFocus?.context;
    return focused?.findAncestorStateOfType<EditableTextState>() != null;
  }

  @override
  void dispose() {
    focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<NoteBloc>();
    return Actions(
      actions: <Type, Action<Intent>>{
        EditNoteIntent: CallbackAction<EditNoteIntent>(
          onInvoke: (_) {
            bloc.add(const NoteEvent.modeChanged(NoteMode.edit));
            return null;
          },
        ),
        PreviewNoteIntent: CallbackAction<PreviewNoteIntent>(
          onInvoke: (_) {
            bloc.add(const NoteEvent.modeChanged(NoteMode.preview));
            return null;
          },
        ),
      },
      child: Focus(focusNode: focusNode, child: widget.child),
    );
  }
}
