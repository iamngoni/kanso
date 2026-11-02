// Kanso — triage_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/triage_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/triage/triage_bloc.dart';
import '../widgets/triage_pager.dart';

/// Inbox Triage, at `/triage`: a card for each note waiting in the Inbox.
class TriagePage extends StatelessWidget {
  const TriagePage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<TriageBloc>(
      create: (context) =>
          getIt<TriageBloc>()..add(const TriageEvent.started()),
      child: KansoPage(
        backgroundColor: context.colors.app,
        body: const _TriagePane(),
      ),
    );
  }
}

class _TriagePane extends StatelessWidget {
  const _TriagePane();

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return BlocConsumer<TriageBloc, TriageState>(
      listenWhen: (a, b) => a.plan != b.plan && b.plan != null,
      listener: (context, state) async {
        final bloc = context.read<TriageBloc>();
        final plan = state.plan;
        if (plan == null) {
          return;
        }
        final yes = await KansoConfirmSheet.show(
          context,
          title: l10n.triageConfirmTitle(plan.filed),
          message: plan.sentence,
          confirmLabel: l10n.triageFile,
          destructive: false,
        );
        bloc.add(
          yes
              ? const TriageEvent.fileAllConfirmed()
              : const TriageEvent.planDismissed(),
        );
      },
      builder: (context, state) {
        final bloc = context.read<TriageBloc>();
        final shown = state.shown;
        final fileable = state.fileable.length;
        final info = state.info;
        return ColoredBox(
          color: colors.app,
          child: Column(
            children: <Widget>[
              KansoTopBar(
                backgroundColor: colors.app,
                bordered: false,
                leading: const ShellBackButton(),
                title: l10n.triageTitle,
                subtitle: l10n.triageSubtitle(shown.length),
                trailing: <Widget>[
                  PopupMenuButton<int>(
                    tooltip: l10n.triageMenu,
                    onSelected: (v) => bloc.add(
                      TriageEvent.fieldAcceptedEverywhere(
                        v == 0 ? TriageField.notebook : TriageField.tag,
                      ),
                    ),
                    itemBuilder: (context) => <PopupMenuEntry<int>>[
                      PopupMenuItem<int>(
                        value: 0,
                        child: Text(l10n.triageAcceptAllNotebooks),
                      ),
                      PopupMenuItem<int>(
                        value: 1,
                        child: Text(l10n.triageAcceptAllTags),
                      ),
                    ],
                  ),
                ],
              ),
              if (info != null && !info.aiOn)
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.md,
                  ),
                  child: Text(
                    l10n.triageNoAi,
                    style: context.typography.meta.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ),
              if (state.notice != null)
                GestureDetector(
                  onTap: () => bloc.add(const TriageEvent.noticeDismissed()),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(
                      state.notice!,
                      style: context.typography.meta.copyWith(
                        color: colors.danger,
                      ),
                    ),
                  ),
                ),
              Expanded(
                child: state.status == LoadStatus.loading
                    ? const Center(child: CircularProgressIndicator())
                    : shown.isEmpty
                    ? KansoEmptyState(
                        title: l10n.triageEmpty,
                        message: l10n.triageEmptyMessage,
                      )
                    : TriagePager(cards: shown),
              ),
              if (fileable > 0)
                SafeArea(
                  top: false,
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: KansoButton(
                      label: l10n.triageFileAllAccepted(fileable),
                      onPressed: () =>
                          bloc.add(const TriageEvent.fileAllRequested()),
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
