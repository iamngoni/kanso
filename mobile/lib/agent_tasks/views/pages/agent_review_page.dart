// Kanso — agent_review_page.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../core/views/widgets/kanso_undo_notice.dart';
import '../../../injection.dart';
import '../../state/agent_review/agent_review_bloc.dart';
import '../widgets/agent_card_view.dart';

/// How long Undo stays after the changes are applied.
const Duration agentUndoFor = Duration(seconds: 30);

/// The changes of task [task], one to a page: swipe between them, tick the
/// ones to apply, and apply them in one step.
class AgentReviewPage extends StatelessWidget {
  const AgentReviewPage({required this.task, super.key});

  final int task;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AgentReviewBloc>(
      create: (_) =>
          getIt<AgentReviewBloc>()..add(AgentReviewEvent.opened(task)),
      child: const KansoPage(body: AgentReviewView()),
    );
  }
}

/// The review itself, for the bloc above it.
class AgentReviewView extends StatefulWidget {
  const AgentReviewView({super.key});

  @override
  State<AgentReviewView> createState() => AgentReviewViewState();
}

class AgentReviewViewState extends State<AgentReviewView> {
  final PageController _pages = PageController();
  int _at = 0;
  Timer? _undoTimer;
  bool _undoShown = false;

  @override
  void dispose() {
    _pages.dispose();
    _undoTimer?.cancel();
    super.dispose();
  }

  void _appliedChanged() {
    _undoTimer?.cancel();
    setState(() => _undoShown = true);
    _undoTimer = Timer(agentUndoFor, () {
      if (mounted) {
        setState(() => _undoShown = false);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<AgentReviewBloc>();
    return BlocConsumer<AgentReviewBloc, AgentReviewState>(
      listenWhen: (before, now) =>
          before.applied == null && now.applied != null,
      listener: (context, state) => _appliedChanged(),
      builder: (context, state) {
        final review = state.review;
        final cards = review?.cards ?? const [];
        final applied = state.applied;
        return Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              leading: KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.settingsClose,
                onPressed: () => Navigator.of(context).maybePop(),
              ),
              title: l10n.agentReviewTitle,
              subtitle: cards.isEmpty
                  ? null
                  : l10n.agentReviewPosition(_at + 1, cards.length),
            ),
            Expanded(
              child: switch (state.status) {
                LoadStatus.loading => const SizedBox.shrink(),
                LoadStatus.failed => Center(
                  child: Text(l10n.agentTroubleGeneric),
                ),
                LoadStatus.ready when cards.isEmpty => Center(
                  child: Text(l10n.agentReviewNothing),
                ),
                LoadStatus.ready => Column(
                  children: <Widget>[
                    if (review!.fromAgent.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.fromLTRB(
                          KansoSpacing.md,
                          KansoSpacing.xs,
                          KansoSpacing.md,
                          0,
                        ),
                        child: Text(
                          review.fromAgent,
                          maxLines: 3,
                          overflow: TextOverflow.ellipsis,
                          style: context.typography.meta.copyWith(
                            color: colors.textSecondary,
                          ),
                        ),
                      ),
                    Expanded(
                      child: PageView.builder(
                        controller: _pages,
                        itemCount: cards.length,
                        onPageChanged: (page) => setState(() => _at = page),
                        itemBuilder: (context, page) {
                          final card = cards[page];
                          return Column(
                            children: <Widget>[
                              Expanded(
                                child: AgentCardView(
                                  card: card,
                                  ticked: state.ticked.contains(card.index),
                                  keep: state.keep.contains(card.index),
                                  onKeep: (on) => bloc.add(
                                    AgentReviewEvent.keepToggled(
                                      card.index,
                                      on: on,
                                    ),
                                  ),
                                ),
                              ),
                              KansoCheckRow(
                                label: l10n.agentReviewApply(1),
                                checked: state.ticked.contains(card.index),
                                onTap: () => bloc.add(
                                  AgentReviewEvent.cardToggled(
                                    card.index,
                                    on: !state.ticked.contains(card.index),
                                  ),
                                ),
                              ),
                            ],
                          );
                        },
                      ),
                    ),
                  ],
                ),
              },
            ),
            if (applied != null && _undoShown)
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: KansoUndoNotice(
                  message: applied,
                  onUndo: () => bloc.add(const AgentReviewEvent.undone()),
                ),
              ),
            if (state.trouble != null)
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  state.trouble!,
                  style: context.typography.meta.copyWith(color: colors.danger),
                ),
              ),
            SafeArea(
              top: false,
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: KansoButton(
                  label: l10n.agentReviewApply(state.ticked.length),
                  onPressed:
                      state.ticked.isEmpty || state.applying || applied != null
                      ? null
                      : () => bloc.add(const AgentReviewEvent.applied()),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}
