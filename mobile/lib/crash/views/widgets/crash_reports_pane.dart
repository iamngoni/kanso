// Kanso — crash_reports_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/crash_send_outcome.dart';
import '../../state/crash_reports/crash_reports_bloc.dart';
import '../../utils/crash_labels.dart';
import 'crash_report_row.dart';

/// Settings › Privacy › Crash reports: the reports waiting on the device,
/// each with its exact text and the choice to send or delete it. It needs a
/// [CrashReportsBloc] above it.
class CrashReportsPane extends StatelessWidget {
  const CrashReportsPane({super.key});

  /// How many bars the page shows while the reports are read.
  static const int loadingRows = 3;

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
            title: l10n.crashReportsTitle,
          ),
          Expanded(
            child: BlocBuilder<CrashReportsBloc, CrashReportsState>(
              builder: (context, state) {
                final bloc = context.read<CrashReportsBloc>();
                if (state.status == LoadStatus.loading) {
                  return Semantics(
                    label: l10n.crashReportsLoading,
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
                        title: l10n.crashReportsTitle,
                        message: l10n.crashReportsFailedMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () =>
                                bloc.add(const CrashReportsEvent.started()),
                          ),
                        ],
                      ),
                    ),
                  );
                }
                final outcome = state.outcome;
                return ListView(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.xs,
                      ),
                      child: Text(
                        l10n.crashReportsIntro,
                        style: context.typography.secondary,
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    if (state.reports.isEmpty)
                      SizedBox(
                        height: 240,
                        child: KansoEmptyState(
                          title: l10n.crashReportsEmptyTitle,
                          message: l10n.crashReportsEmptyMessage,
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
                                i < state.reports.length;
                                i++
                              ) ...<Widget>[
                                if (i > 0)
                                  const KansoDivider(indent: KansoSpacing.md),
                                CrashReportRow(
                                  report: state.reports[i],
                                  open: state.expanded == state.reports[i].id,
                                  sending: state.sending.contains(
                                    state.reports[i].id,
                                  ),
                                  onToggle: () => bloc.add(
                                    CrashReportsEvent.reportToggled(
                                      state.reports[i].id,
                                    ),
                                  ),
                                  onSend: () => bloc.add(
                                    CrashReportsEvent.sendRequested(
                                      state.reports[i].id,
                                    ),
                                  ),
                                  onDelete: () => bloc.add(
                                    CrashReportsEvent.discardRequested(
                                      state.reports[i].id,
                                    ),
                                  ),
                                ),
                              ],
                            ],
                          ),
                        ),
                      ),
                    if (state.reports.length > 1) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoButton(
                        label: l10n.crashReportsSendAll,
                        variant: KansoButtonVariant.secondary,
                        onPressed: state.sending.isNotEmpty
                            ? null
                            : () => bloc.add(
                                const CrashReportsEvent.sendAllRequested(),
                              ),
                      ),
                    ],
                    if (outcome != null) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      Text(
                        l10n.crashOutcomeMessage(outcome),
                        textAlign: TextAlign.center,
                        style: context.typography.secondary.copyWith(
                          color: outcome == CrashSendOutcome.sent
                              ? colors.textSecondary
                              : colors.danger,
                        ),
                      ),
                    ],
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
