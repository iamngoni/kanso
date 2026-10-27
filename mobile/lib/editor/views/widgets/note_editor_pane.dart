// Kanso — note_editor_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../injection.dart';
import '../../../note/models/data/note_mode.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../note/views/widgets/note_loading_pane.dart';
import '../../models/data/editor_status.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../state/editor/editor_bloc.dart';
import 'note_editor_content.dart';

/// A note to edit: opened for editing when it appears, saved and let go when
/// it goes. It reads the `NoteBloc` above it for where the caret starts.
///
/// If the note's lock closes or the note is gone while it is open, the page
/// turns to Preview, which shows the lock or says the note is not there.
class NoteEditorPane extends StatelessWidget {
  const NoteEditorPane({
    required this.noteId,
    required this.gutter,
    this.caret,
    this.caretInBody = false,
    this.scroll = 0,
    this.focusTitle = false,
    super.key,
  });

  final String noteId;

  /// The space either side of the page.
  final double gutter;

  /// Where the caret starts; the start of the text when null.
  final int? caret;

  /// Whether [caret] counts the note's front matter (an offset in the whole
  /// body), which the editor does not show.
  final bool caretInBody;

  /// How far the page is scrolled to start with.
  final double scroll;

  /// Whether the title gets the focus rather than the text.
  final bool focusTitle;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocProvider<EditorBloc>(
      key: ValueKey<String>(noteId),
      create: (_) => getIt<EditorBloc>()..add(EditorEvent.started(noteId)),
      child: BlocConsumer<EditorBloc, EditorState>(
        listenWhen: (before, now) => before.status != now.status,
        listener: (context, state) {
          if (state.status == EditorStatus.locked ||
              state.status == EditorStatus.notFound) {
            context.read<NoteBloc>().add(
              const NoteEvent.modeChanged(NoteMode.preview),
            );
          }
        },
        builder: (context, state) {
          final document = state.document;
          final snapshot = state.snapshot;
          if (state.status == EditorStatus.ready &&
              document != null &&
              snapshot != null) {
            return NoteEditorContent(
              key: ValueKey<EditorDocument>(document),
              document: document,
              snapshot: snapshot,
              gutter: gutter,
              caret: caret,
              caretInBody: caretInBody,
              scroll: scroll,
              focusTitle: focusTitle,
            );
          }
          if (state.status == EditorStatus.failed) {
            return Center(
              child: Padding(
                padding: EdgeInsets.all(gutter),
                child: KansoMessageCard(
                  icon: KansoIcons.fileText,
                  title: l10n.noteFailedTitle,
                  message: l10n.noteFailedMessage,
                  actions: <Widget>[
                    KansoButton(
                      label: l10n.retry,
                      onPressed: () => context.read<EditorBloc>().add(
                        EditorEvent.started(noteId),
                      ),
                    ),
                  ],
                ),
              ),
            );
          }
          return Semantics(
            label: l10n.editorOpeningLabel,
            container: true,
            child: Padding(
              padding: const EdgeInsets.only(top: KansoSpacing.xxs),
              child: NoteLoadingPane(gutter: gutter),
            ),
          );
        },
      ),
    );
  }
}
