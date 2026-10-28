// Kanso — ask_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../state/ask/ask_bloc.dart';
import '../../utils/open_answer_link.dart';
import '../../utils/open_cited_note.dart';
import 'ask_composer.dart';
import 'ask_conversation.dart';
import 'ask_intro.dart';

/// Ask your notes: the conversation above, the question field below. It
/// reads the `AskBloc` above it.
///
/// [inSheet] says the page is a sheet over the panes (on a tablet), which a
/// cited note closes before it opens.
class AskPane extends StatelessWidget {
  const AskPane({this.inSheet = false, this.usedExpanded = false, super.key});

  final bool inSheet;

  /// Whether each list of notes used starts open.
  final bool usedExpanded;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bloc = context.read<AskBloc>();
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          BlocBuilder<AskBloc, AskState>(
            buildWhen: (before, now) =>
                before.turns.isEmpty != now.turns.isEmpty,
            builder: (context, state) => KansoTopBar(
              backgroundColor: colors.app,
              bordered: false,
              leading: KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.settingsClose,
                onPressed: () => context.canPop()
                    ? context.pop()
                    : context.go(KansoRoutes.notes),
              ),
              title: l10n.askTitle,
              trailing: <Widget>[
                if (state.turns.isNotEmpty)
                  KansoIconButton(
                    icon: KansoIcons.circleX,
                    semanticLabel: l10n.askClear,
                    onPressed: () => bloc.add(const AskEvent.cleared()),
                  ),
              ],
            ),
          ),
          Expanded(
            child: BlocBuilder<AskBloc, AskState>(
              builder: (context, state) {
                if (state.status == LoadStatus.loading) {
                  return const Padding(
                    padding: EdgeInsets.all(KansoSpacing.md),
                    child: KansoSkeleton(height: 14),
                  );
                }
                if (state.status == LoadStatus.failed) {
                  return Center(
                    child: KansoMessageCard(
                      icon: KansoIcons.sparkles,
                      title: l10n.aiLoadFailedTitle,
                      message: l10n.aiLoadFailedMessage,
                      actions: <Widget>[
                        KansoButton(
                          label: l10n.retry,
                          onPressed: () => bloc.add(const AskEvent.started()),
                        ),
                      ],
                    ),
                  );
                }
                if (!state.aiEnabled) {
                  return KansoEmptyState(
                    title: l10n.askOffTitle,
                    message: l10n.askOffMessage,
                    action: KansoButton(
                      label: l10n.askOpenSettings,
                      expanded: false,
                      onPressed: () => context.push(KansoRoutes.settingsAi),
                    ),
                  );
                }
                return Column(
                  children: <Widget>[
                    Expanded(
                      child: state.turns.isEmpty
                          ? AskIntro(privacy: state.privacy)
                          : AskConversation(
                              state: state,
                              usedExpanded: usedExpanded,
                              onOpenNote: (id) => openCitedNote(
                                context,
                                id,
                                closeFirst: inSheet,
                              ),
                              onOpenLink: openAnswerLink,
                              onRetry: () => bloc.add(const AskEvent.retried()),
                            ),
                    ),
                    AskComposer(
                      busy: state.busy,
                      onAsk: (question) =>
                          bloc.add(AskEvent.questionAsked(question)),
                      onStop: () => bloc.add(const AskEvent.stopped()),
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
