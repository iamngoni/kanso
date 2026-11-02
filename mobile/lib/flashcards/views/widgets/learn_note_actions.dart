// Kanso — learn_note_actions.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';
import 'make_flashcards_sheet.dart';

/// "Draft from This Note…", "Make Flashcards…" and "Quiz Me" for a note's
/// info menu. They are drawn only while AI is on. A note kept out of AI
/// shows Make Flashcards dimmed with the reason.
class LearnNoteActions extends StatefulWidget {
  const LearnNoteActions({required this.noteId, super.key});

  final String noteId;

  @override
  State<LearnNoteActions> createState() => _LearnNoteActionsState();
}

class _LearnNoteActionsState extends State<LearnNoteActions> {
  String? _blocked;
  bool _known = false;

  @override
  void initState() {
    super.initState();
    try {
      getIt<KansoLibrary>()
          .cards()
          .makeBlocked(noteId: widget.noteId)
          .then((why) {
            if (mounted) {
              setState(() {
                _blocked = why;
                _known = true;
              });
            }
          })
          .catchError((Object _) {});
    } on Object {
      // No library to ask (a preview): the rows stay as they are.
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AiStatusBloc, AiStatusState>(
      buildWhen: (before, now) => before.enabled != now.enabled,
      builder: (context, state) {
        if (!state.enabled) {
          return const SizedBox.shrink();
        }
        final router = GoRouter.of(context);
        return Column(
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.penTool,
              label: FlashcardsText.draftFrom,
              onTap: _blocked != null
                  ? null
                  : () {
                      Navigator.of(context).pop();
                      router.push(
                        KansoRoutes.draft,
                        extra: <String>[widget.noteId],
                      );
                    },
            ),
            KansoSettingsRow(
              icon: KansoIcons.copy,
              label: FlashcardsText.makeCards,
              value: _known ? _blocked : null,
              onTap: _blocked != null
                  ? null
                  : () {
                      final navigator = Navigator.of(context);
                      final host = navigator.context;
                      navigator.pop();
                      showMakeFlashcards(host, widget.noteId);
                    },
            ),
            KansoSettingsRow(
              icon: KansoIcons.sparkles,
              label: FlashcardsText.quizMe,
              onTap: _blocked != null
                  ? null
                  : () {
                      Navigator.of(context).pop();
                      router.push(
                        KansoRoutes.quiz,
                        extra: <String>[widget.noteId],
                      );
                    },
            ),
          ],
        );
      },
    );
  }
}
