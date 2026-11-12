// Kanso — agent_tasks_page.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_session.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_dialog.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_page_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../state/agent_tasks/agent_tasks_bloc.dart';
import '../widgets/agent_session_actions.dart';
import '../widgets/agent_session_tile.dart';
import '../widgets/agent_trouble_card.dart';
import 'agent_review_page.dart';
import 'agent_session_page.dart';

/// Agent tasks: a goal, the AI at work, and the changes it proposes. With
/// [noteIds] the AI may read only those notes.
class AgentTasksPage extends StatelessWidget {
  const AgentTasksPage({this.noteIds = const <String>[], super.key});

  final List<String> noteIds;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AgentTasksBloc>(
      create: (_) =>
          getIt<AgentTasksBloc>()..add(const AgentTasksEvent.started()),
      child: KansoPage(
        backgroundColor: context.colors.app,
        body: _Body(noteIds: noteIds),
      ),
    );
  }
}

class _Body extends StatefulWidget {
  const _Body({required this.noteIds});

  final List<String> noteIds;

  @override
  State<_Body> createState() => _BodyState();
}

class _BodyState extends State<_Body> {
  final TextEditingController _goal = TextEditingController();
  String _lastGoal = '';
  bool _lastConfirmed = false;

  /// The notebook new notes go in; empty is the Inbox.
  String _bookId = '';
  String? _bookName;

  @override
  void dispose() {
    _goal.dispose();
    super.dispose();
  }

  /// The rename and delete menu of [session], from a long press on its row.
  Future<void> _sessionMenu(
    BuildContext context,
    AgentTasksBloc bloc,
    AgentSession session,
  ) async {
    final size = MediaQuery.sizeOf(context);
    final action = await showAgentSessionMenu(
      context,
      Offset(size.width / 2, size.height / 2),
    );
    if (action == null || !context.mounted) {
      return;
    }
    switch (action) {
      case AgentSessionAction.rename:
        final title = await askAgentSessionTitle(context, session.title);
        if (title != null) {
          bloc.add(AgentTasksEvent.renamed(session.key, title));
        }
      case AgentSessionAction.delete:
        if (await confirmDeleteAgentSession(context)) {
          bloc.add(AgentTasksEvent.deleted(session.key));
        }
    }
  }

  Future<void> _send(BuildContext context) async {
    final goal = _goal.text.trim();
    if (goal.isEmpty) {
      return;
    }
    final bloc = context.read<AgentTasksBloc>();
    var confirmed = false;
    // With no notes chosen the task may read every notebook: ask first,
    // when Settings say so.
    if (widget.noteIds.isEmpty && bloc.state.prefs.askBeforeReadingAll) {
      final l10n = context.l10n;
      confirmed = await showKansoConfirm(
        context,
        title: l10n.agentTasksReadAllTitle,
        body: l10n.agentTasksReadAllBody,
        cancelLabel: l10n.agentTasksReadAllCancel,
        confirmLabel: l10n.agentTasksReadAllConfirm,
      );
      if (!confirmed) {
        return;
      }
    }
    _lastGoal = goal;
    _lastConfirmed = confirmed;
    bloc.add(
      AgentTasksEvent.goalSent(
        goal,
        noteIds: widget.noteIds,
        newNoteBook: _bookId,
        confirmedAll: confirmed,
      ),
    );
  }

  Future<void> _pickBook(BuildContext context) async {
    final index = context.read<LibraryIndexBloc>();
    final picked = await showKansoBottomSheet<(String, String?)>(
      context,
      builder: (_) => BlocProvider<LibraryIndexBloc>.value(
        value: index,
        child: _BookSheet(chosen: _bookId),
      ),
    );
    if (picked != null && mounted) {
      setState(() {
        _bookId = picked.$1;
        _bookName = picked.$2;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<AgentTasksBloc>();
    return BlocConsumer<AgentTasksBloc, AgentTasksState>(
      listenWhen: (before, now) => before.started != now.started,
      listener: (context, state) => _goal.clear(),
      builder: (context, state) {
        final trouble = state.trouble;
        return Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              leading: KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.settingsClose,
                onPressed: () => context.canPop()
                    ? context.pop()
                    : context.go(KansoRoutes.notes),
              ),
              title: l10n.agentTasksTitle,
            ),
            Expanded(
              child: KansoListScrollbar(
                child: ListView(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  children: <Widget>[
                    Text(
                      l10n.agentTasksGoalLabel,
                      style: context.typography.meta.copyWith(
                        color: colors.textMuted,
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.xxs),
                    KansoTextField(
                      controller: _goal,
                      hint: l10n.agentTasksGoalHint,
                      minLines: 2,
                      maxLines: 4,
                    ),
                    if (widget.noteIds.isNotEmpty) ...<Widget>[
                      const SizedBox(height: KansoSpacing.xxs),
                      Text(
                        l10n.agentTasksChosenNotes(widget.noteIds.length),
                        style: context.typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                      ),
                    ],
                    const SizedBox(height: KansoSpacing.xs),
                    KansoSettingsRow(
                      label: l10n.agentTasksNewNotesIn,
                      value: _bookName ?? l10n.agentTasksInbox,
                      chevron: true,
                      onTap: () => _pickBook(context),
                    ),
                    const SizedBox(height: KansoSpacing.sm),
                    KansoButton(
                      label: l10n.agentTasksStart,
                      onPressed: state.starting
                          ? null
                          : () => unawaited(_send(context)),
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    if (trouble != null)
                      Center(
                        child: AgentTroubleCard(
                          message: trouble,
                          onTryAgain: () {
                            bloc.add(const AgentTasksEvent.troubleCleared());
                            if (_lastGoal.isNotEmpty) {
                              bloc.add(
                                AgentTasksEvent.goalSent(
                                  _lastGoal,
                                  noteIds: widget.noteIds,
                                  newNoteBook: _bookId,
                                  confirmedAll: _lastConfirmed,
                                ),
                              );
                            }
                          },
                          onSettings: () =>
                              context.push(KansoRoutes.settingsAi),
                        ),
                      )
                    else if (state.sessions.isEmpty)
                      KansoEmptyState(
                        title: l10n.agentTasksEmptyTitle,
                        message: l10n.agentTasksEmptyMessage,
                      ),
                    // Latest activity first: the core sorts them.
                    for (final session in state.sessions) ...<Widget>[
                      AgentSessionTile(
                        key: ValueKey<String>(session.key),
                        session: session,
                        nowMs: DateTime.now().millisecondsSinceEpoch,
                        onOpen: () => Navigator.of(context).push<void>(
                          KansoPageRoute<void>(
                            builder: (_) =>
                                AgentSessionPage(sessionKey: session.key),
                          ),
                        ),
                        onLongPress: () => unawaited(
                          _sessionMenu(context, bloc, session),
                        ),
                        onStop: () =>
                            bloc.add(AgentTasksEvent.stopped(session.id)),
                        onReview: () => Navigator.of(context).push<void>(
                          KansoPageRoute<void>(
                            builder: (_) => AgentReviewPage(task: session.id),
                          ),
                        ),
                      ),
                      const KansoDivider(),
                    ],
                  ],
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}

/// The notebooks to put new notes in: the Inbox first, then the library's
/// notebooks that are not locked.
class _BookSheet extends StatelessWidget {
  const _BookSheet({required this.chosen});

  final String chosen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final notebooks = context.select<LibraryIndexBloc, List<NotebookRow>>(
      (index) => index.state.notebooks,
    );
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Text(
            l10n.agentTasksNewNotesIn,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          children: <Widget>[
            KansoCheckRow(
              label: l10n.agentTasksInbox,
              icon: KansoIcons.notebook,
              checked: chosen.isEmpty,
              onTap: () => Navigator.of(context).pop(('', null)),
            ),
            for (final notebook in notebooks)
              if (notebook.role != NotebookRole.inbox)
                KansoCheckRow(
                  label: notebook.name,
                  icon: notebook.locked ? KansoIcons.lock : KansoIcons.notebook,
                  indent: notebook.depth * KansoSpacing.md,
                  checked: chosen == notebook.id,
                  onTap: notebook.locked
                      ? null
                      : () => Navigator.of(
                          context,
                        ).pop((notebook.id, notebook.name)),
                ),
          ],
        ),
      ],
    );
  }
}
