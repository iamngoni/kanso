// Kanso — agent_settings_section.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_task_prefs.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../injection.dart';
import '../../state/agent_settings/agent_settings_cubit.dart';

/// Settings › AI › Agent tasks: the switch, and what an agent may propose.
class AgentSettingsSection extends StatelessWidget {
  const AgentSettingsSection({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AgentSettingsCubit>(
      create: (_) => getIt<AgentSettingsCubit>()..load(),
      child: BlocBuilder<AgentSettingsCubit, AgentTaskPrefs>(
        builder: (context, prefs) {
          final l10n = context.l10n;
          final cubit = context.read<AgentSettingsCubit>();
          Widget row(
            String label, {
            required bool on,
            required AgentTaskPrefs Function({required bool on}) next,
          }) => KansoSettingsRow(
            label: label,
            trailing: KansoSwitch(
              value: on,
              onChanged: prefs.enabled
                  ? (value) => cubit.change(next(on: value))
                  : null,
            ),
          );
          return Column(
            children: <Widget>[
              const SizedBox(height: KansoSpacing.lg),
              KansoGroupedSection(
                title: l10n.agentSettingsTitle,
                footer: l10n.agentSettingsFooter,
                children: <Widget>[
                  row(
                    l10n.agentSettingsEnabled,
                    on: prefs.enabled,
                    next: ({required on}) => prefs.copyWith(enabled: on),
                  ),
                  row(
                    l10n.agentSettingsNewNotes,
                    on: prefs.newNotes,
                    next: ({required on}) => prefs.copyWith(newNotes: on),
                  ),
                  row(
                    l10n.agentSettingsEdits,
                    on: prefs.edits,
                    next: ({required on}) => prefs.copyWith(edits: on),
                  ),
                  row(
                    l10n.agentSettingsMoves,
                    on: prefs.moves,
                    next: ({required on}) => prefs.copyWith(moves: on),
                  ),
                  row(
                    l10n.agentSettingsMerges,
                    on: prefs.merges,
                    next: ({required on}) => prefs.copyWith(merges: on),
                  ),
                ],
              ),
            ],
          );
        },
      ),
    );
  }
}
