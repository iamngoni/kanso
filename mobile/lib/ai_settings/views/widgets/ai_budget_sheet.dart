// Kanso — ai_budget_sheet.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../state/ai_settings/ai_settings_bloc.dart';

/// The sizes a question may be, smallest first.
const List<String> askBudgets = <String>['small', 'standard', 'large'];

/// Opens the sizes over the page; choosing one saves it and closes the list.
Future<void> showAiBudgetSheet(BuildContext context, AiSettingsBloc bloc) {
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => BlocProvider<AiSettingsBloc>.value(
      value: bloc,
      child: const AiBudgetSheet(),
    ),
  );
}

/// The words of a size in the picker and the row.
String askBudgetName(BuildContext context, String budget) {
  final l10n = context.l10n;
  return switch (budget) {
    'small' => l10n.aiAskBudgetSmall,
    'large' => l10n.aiAskBudgetLarge,
    _ => l10n.aiAskBudgetStandard,
  };
}

/// Small, Standard and Large; the one in use has a tick and, under it, how
/// many words of the notes it sends and what that costs.
class AiBudgetSheet extends StatelessWidget {
  const AiBudgetSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return BlocBuilder<AiSettingsBloc, AiSettingsState>(
      builder: (context, state) {
        final settings = state.settings;
        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: Text(
                context.l10n.aiAskBudget,
                style: typography.heading2,
              ),
            ),
            for (final budget in askBudgets) ...<Widget>[
              const KansoDivider(indent: KansoSpacing.md),
              KansoPressable(
                onTap: () {
                  if (budget != settings.askBudget) {
                    context.read<AiSettingsBloc>().add(
                      AiSettingsEvent.askChanged(budget: budget),
                    );
                  }
                  Navigator.of(context).maybePop();
                },
                child: ConstrainedBox(
                  constraints: const BoxConstraints(
                    minHeight: KansoSizes.settingsRow,
                  ),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.md,
                      vertical: KansoSpacing.sm,
                    ),
                    child: Row(
                      children: <Widget>[
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: <Widget>[
                              Text(
                                askBudgetName(context, budget),
                                style: typography.body,
                              ),
                              if (budget == settings.askBudget) ...<Widget>[
                                const SizedBox(height: KansoSpacing.xxs),
                                if (settings.budgetWordsLine.isNotEmpty)
                                  Text(
                                    settings.budgetWordsLine,
                                    style: typography.meta,
                                  ),
                                if (settings.budgetCostLine.isNotEmpty)
                                  Text(
                                    settings.budgetCostLine,
                                    style: typography.meta,
                                  ),
                              ],
                            ],
                          ),
                        ),
                        if (budget == settings.askBudget)
                          KansoIcon(KansoIcons.check, color: colors.accent),
                      ],
                    ),
                  ),
                ),
              ),
            ],
            const SizedBox(height: KansoSpacing.md),
          ],
        );
      },
    );
  }
}
