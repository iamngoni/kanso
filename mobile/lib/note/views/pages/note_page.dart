// Kanso — note_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../injection.dart';
import '../../../lock/state/unlock/unlock_bloc.dart';
import '../../../sharing/state/comments/comments_bloc.dart';
import '../../../sharing/state/shared_note_status/shared_note_status_bloc.dart';
import '../../../sharing/views/widgets/shared_note_live.dart';
import '../../state/note/note_bloc.dart';
import '../../state/note_discovery/note_discovery_bloc.dart';
import '../../state/note_mode_intent/note_mode_intent_bloc.dart';
import '../widgets/note_pane.dart';
import '../widgets/note_theme_sync.dart';

/// One note, opened from a place or a search: read when it is opened, and
/// given up when it is left.
class NotePage extends StatelessWidget {
  const NotePage({
    required this.noteId,
    this.inPane = false,
    this.bodyLine,
    this.cursor16,
    this.titleFirst = false,
    super.key,
  });

  final String noteId;

  /// Whether it is the note pane of a wide window rather than a page of its
  /// own, with nowhere to go back to.
  final bool inPane;

  /// A task source line to open in Edit, if a reminder led here.
  final int? bodyLine;

  /// A template cursor in UTF-16 units, if a template led here.
  final int? cursor16;

  /// Whether a template made an untitled note that should focus its title.
  final bool titleFirst;

  @override
  Widget build(BuildContext context) {
    // The theme the text is highlighted in is the one the screen is in: a
    // change of it builds this again, and the bloc is told.
    final themeId = syntaxThemeId(context);
    return BlocProvider<NoteBloc>(
      // Another note is a new read: nothing of the last is kept.
      key: ValueKey<String>(
        '$noteId:${bodyLine ?? ''}:${cursor16 ?? ''}:$titleFirst',
      ),
      create: (context) {
        // A note opened by a link, or any way but a list, is a visit too;
        // one the history is already at changes nothing.
        context.read<NavigationHistoryBloc>().add(
          NavigationHistoryEvent.visited(noteId),
        );
        return getIt<NoteBloc>()..add(
          NoteEvent.started(
            noteId,
            themeId,
            bodyLine: bodyLine,
            cursor16: cursor16,
            titleFirst: titleFirst,
          ),
        );
      },
      child: NoteThemeSync(
        themeId: themeId,
        // A shared note's live room is open while the note is.
        child: SharedNoteLive(
          noteId: noteId,
          child: MultiBlocProvider(
            providers: <BlocProvider<dynamic>>[
              BlocProvider<CommentsBloc>(
                create: (_) =>
                    getIt<CommentsBloc>()..add(CommentsEvent.started(noteId)),
              ),
              BlocProvider<SharedNoteStatusBloc>(
                create: (_) =>
                    getIt<SharedNoteStatusBloc>()
                      ..add(SharedNoteStatusEvent.started(noteId)),
              ),
              BlocProvider<NoteDiscoveryBloc>(
                create: (_) =>
                    getIt<NoteDiscoveryBloc>()
                      ..add(NoteDiscoveryEvent.started(noteId)),
              ),
              // Unlocking this note, if it is locked.
              BlocProvider<UnlockBloc>(create: (_) => getIt<UnlockBloc>()),
            ],
            child: Scaffold(
              resizeToAvoidBottomInset: false,
              backgroundColor: context.colors.editor,
              body: BlocListener<NoteModeIntentBloc, NoteModeIntentState>(
                listenWhen: (previous, current) =>
                    previous.noteId != current.noteId ||
                    previous.mode != current.mode,
                listener: (context, intent) {
                  final mode = intent.mode;
                  if (intent.noteId == noteId && mode != null &&
                      context.read<NoteBloc>().state.mode != mode) {
                    context.read<NoteBloc>().add(NoteEvent.modeChanged(mode));
                  }
                },
                child: NotePane(inPane: inPane),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
