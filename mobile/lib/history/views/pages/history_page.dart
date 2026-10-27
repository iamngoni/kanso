// Kanso — history_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../models/data/history_status.dart';
import '../../state/note_history/note_history_bloc.dart';
import 'history_content.dart';

/// A note's saved versions, their changes, and the action that restores one.
class HistoryPage extends StatelessWidget {
  const HistoryPage({required this.noteId, super.key});

  /// The note whose versions are shown.
  final String noteId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<NoteHistoryBloc>(
      create: (_) =>
          getIt<NoteHistoryBloc>()..add(NoteHistoryEvent.started(noteId)),
      child: BlocListener<NoteHistoryBloc, NoteHistoryState>(
        listenWhen: (before, now) => !before.restored && now.restored,
        listener: (context, state) {
          _leaveHistory(context, noteId);
        },
        child: Scaffold(
          backgroundColor: context.colors.app,
          body: Column(
            children: <Widget>[
              KansoTopBar(
                title: context.l10n.historyTitle,
                leading: KansoIconButton(
                  icon: KansoIcons.close,
                  semanticLabel: context.l10n.historyClose,
                  onPressed: () => _leaveHistory(context, noteId),
                ),
              ),
              Expanded(
                child: BlocBuilder<NoteHistoryBloc, NoteHistoryState>(
                  builder: (context, state) => switch (state.status) {
                    HistoryStatus.loading => Center(
                      child: Semantics(
                        label: context.l10n.historyLoading,
                        child: const CircularProgressIndicator(),
                      ),
                    ),
                    HistoryStatus.failed => Center(
                      child: Padding(
                        padding: const EdgeInsets.all(KansoSpacing.md),
                        child: KansoMessageCard(
                          icon: KansoIcons.clock,
                          title: context.l10n.historyFailedTitle,
                          message: context.l10n.historyFailedMessage,
                          actions: <Widget>[
                            KansoButton(
                              label: context.l10n.retry,
                              onPressed: () => context
                                  .read<NoteHistoryBloc>()
                                  .add(const NoteHistoryEvent.retried()),
                            ),
                          ],
                        ),
                      ),
                    ),
                    HistoryStatus.locked => Center(
                      child: Padding(
                        padding: const EdgeInsets.all(KansoSpacing.md),
                        child: KansoMessageCard(
                          icon: KansoIcons.lock,
                          title: context.l10n.historyLockedTitle,
                          message: context.l10n.historyLockedMessage,
                          actions: const <Widget>[],
                        ),
                      ),
                    ),
                    HistoryStatus.notFound => Center(
                      child: Padding(
                        padding: const EdgeInsets.all(KansoSpacing.md),
                        child: KansoMessageCard(
                          icon: KansoIcons.fileText,
                          title: context.l10n.historyNotFoundTitle,
                          message: context.l10n.historyNotFoundMessage,
                          actions: const <Widget>[],
                        ),
                      ),
                    ),
                    HistoryStatus.ready => HistoryContent(state: state),
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Returns to the note, or to All Notes when history was opened as a link.
void _leaveHistory(BuildContext context, String noteId) {
  if (context.canPop()) {
    context.pop();
  } else {
    context.go('${KansoRoutes.noteById}/${Uri.encodeComponent(noteId)}');
  }
}
