// Kanso — my_feedback_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_refresh_control.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/feedback_entry.dart';
import '../../state/my_feedback/my_feedback_bloc.dart';
import 'feedback_entry_row.dart';
import 'feedback_entry_sheet.dart';

/// Settings › Feedback › My feedback: the reports, newest first, each with
/// its chip and date. Swipe or long-press offers Delete; tapping opens the
/// status timeline. It needs a [MyFeedbackBloc] above it.
class MyFeedbackPane extends StatefulWidget {
  const MyFeedbackPane({super.key});

  @override
  State<MyFeedbackPane> createState() => _MyFeedbackPaneState();
}

class _MyFeedbackPaneState extends State<MyFeedbackPane> {
  /// How many bars the page shows while the reports are read.
  static const int loadingRows = 3;

  late MyFeedbackBloc _bloc;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _bloc = context.read<MyFeedbackBloc>();
  }

  @override
  void dispose() {
    // The released reports were on screen: their rows stop being bold.
    if (!_bloc.isClosed) {
      _bloc.add(const MyFeedbackEvent.seen());
    }
    super.dispose();
  }

  Future<bool> _confirmDelete(BuildContext context, FeedbackEntry entry) {
    final l10n = context.l10n;
    final bloc = context.read<MyFeedbackBloc>();
    return KansoConfirmSheet.show(
      context,
      title: l10n.feedbackDeleteTitle,
      message: l10n.feedbackDeleteMessage,
      confirmLabel: l10n.feedbackDelete,
    ).then((confirmed) {
      if (confirmed) {
        bloc.add(MyFeedbackEvent.deleteRequested(entry.id));
      }
      // The list redraws from the bloc, so the row is not dismissed here.
      return false;
    });
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.feedbackMine,
          ),
          Expanded(
            child: BlocBuilder<MyFeedbackBloc, MyFeedbackState>(
              builder: (context, state) {
                final bloc = context.read<MyFeedbackBloc>();
                if (state.status == LoadStatus.loading) {
                  return Semantics(
                    label: l10n.feedbackMineLoading,
                    container: true,
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: Column(
                        children: <Widget>[
                          for (var i = 0; i < loadingRows; i++)
                            const Padding(
                              padding: EdgeInsets.symmetric(
                                vertical: KansoSpacing.sm,
                              ),
                              child: KansoSkeleton(height: 14),
                            ),
                        ],
                      ),
                    ),
                  );
                }
                if (state.status == LoadStatus.failed) {
                  return Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        title: l10n.feedbackMine,
                        message: l10n.feedbackMineFailed,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => bloc.add(
                              const MyFeedbackEvent.refreshRequested(),
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                }
                return CustomScrollView(
                  physics: const AlwaysScrollableScrollPhysics(
                    parent: BouncingScrollPhysics(),
                  ),
                  slivers: <Widget>[
                    KansoRefreshControl(
                      onRefresh: () async {
                        bloc.add(const MyFeedbackEvent.refreshRequested());
                        await bloc.stream.firstWhere((s) => !s.refreshing);
                      },
                    ),
                    SliverPadding(
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      sliver: SliverList.list(
                        children: <Widget>[
                          Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: KansoSpacing.xs,
                            ),
                            child: Text(
                              l10n.feedbackMineIntro,
                              style: context.typography.secondary,
                            ),
                          ),
                          const SizedBox(height: KansoSpacing.md),
                          if (state.entries.isEmpty)
                            SizedBox(
                              height: 240,
                              child: KansoEmptyState(
                                title: l10n.feedbackMineEmptyTitle,
                                message: l10n.feedbackMineEmptyMessage,
                              ),
                            )
                          else
                            ClipRRect(
                              borderRadius: KansoRadii.largeBorder,
                              child: ColoredBox(
                                color: context.groupedSurface,
                                child: Column(
                                  children: <Widget>[
                                    for (
                                      var i = 0;
                                      i < state.entries.length;
                                      i++
                                    ) ...<Widget>[
                                      if (i > 0)
                                        const KansoDivider(
                                          indent: KansoSpacing.md,
                                        ),
                                      Dismissible(
                                        key: ValueKey<String>(
                                          'feedback-${state.entries[i].id}',
                                        ),
                                        direction: state.entries[i].deleting
                                            ? DismissDirection.none
                                            : DismissDirection.endToStart,
                                        confirmDismiss: (_) => _confirmDelete(
                                          context,
                                          state.entries[i],
                                        ),
                                        background: ColoredBox(
                                          color: colors.danger,
                                          child: Align(
                                            alignment:
                                                AlignmentDirectional.centerEnd,
                                            child: Padding(
                                              padding: const EdgeInsets.only(
                                                right: KansoSpacing.md,
                                              ),
                                              child: Text(
                                                l10n.feedbackDelete,
                                                style: context.typography.body
                                                    .copyWith(
                                                      color: Colors.white,
                                                    ),
                                              ),
                                            ),
                                          ),
                                        ),
                                        child: FeedbackEntryRow(
                                          entry: state.entries[i],
                                          onTap: () => FeedbackEntrySheet.show(
                                            context,
                                            state.entries[i],
                                            onDelete: () => _confirmDelete(
                                              context,
                                              state.entries[i],
                                            ),
                                          ),
                                          onLongPress: state.entries[i].deleting
                                              ? null
                                              : () => _confirmDelete(
                                                  context,
                                                  state.entries[i],
                                                ),
                                        ),
                                      ),
                                    ],
                                  ],
                                ),
                              ),
                            ),
                        ],
                      ),
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
