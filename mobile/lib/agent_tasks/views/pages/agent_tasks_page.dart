// Kanso — agent_tasks_page.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../state/agent_tasks/agent_tasks_bloc.dart';
import '../widgets/agent_task_tile.dart';
import '../widgets/agent_trouble_card.dart';
import 'agent_review_page.dart';

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

  @override
  void dispose() {
    _goal.dispose();
    super.dispose();
  }

  void _send(BuildContext context) {
    final goal = _goal.text.trim();
    if (goal.isEmpty) {
      return;
    }
    _lastGoal = goal;
    context.read<AgentTasksBloc>().add(
      AgentTasksEvent.goalSent(goal, noteIds: widget.noteIds),
    );
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
                  const SizedBox(height: KansoSpacing.sm),
                  KansoButton(
                    label: l10n.agentTasksStart,
                    onPressed: state.starting ? null : () => _send(context),
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
                              ),
                            );
                          }
                        },
                        onSettings: () => context.push(KansoRoutes.settingsAi),
                      ),
                    )
                  else if (state.tasks.isEmpty)
                    KansoEmptyState(
                      title: l10n.agentTasksEmptyTitle,
                      message: l10n.agentTasksEmptyMessage,
                    ),
                  for (final task in state.tasks.reversed)
                    AgentTaskTile(
                      key: ValueKey<int>(task.id),
                      task: task,
                      onStop: () => bloc.add(AgentTasksEvent.stopped(task.id)),
                      onDismiss: () =>
                          bloc.add(AgentTasksEvent.dismissed(task.id)),
                      onReply: (text) =>
                          bloc.add(AgentTasksEvent.replied(task.id, text)),
                      onReview: () => Navigator.of(context).push<void>(
                        MaterialPageRoute<void>(
                          builder: (_) => AgentReviewPage(task: task.id),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}
