// Kanso — reminders_section.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../state/reminders_settings/reminders_settings_bloc.dart';

/// Settings > Reminders: the row "Remind me of tasks" with its switch, and
/// under it what it does, or, when the system does not let Kanso show
/// notifications, how to allow them. It needs the `RemindersSettingsBloc`
/// above it.
class RemindersSection extends StatelessWidget {
  const RemindersSection({required this.dividerIndent, super.key});

  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<RemindersSettingsBloc, RemindersSettingsState>(
      builder: (context, state) {
        final bloc = context.read<RemindersSettingsBloc>();
        return KansoGroupedSection(
          title: l10n.settingsReminders,
          footer: state.blockedBySystem
              ? l10n.settingsRemindersDenied
              : l10n.settingsRemindersDetail,
          dividerIndent: dividerIndent,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.bell,
              label: l10n.settingsRemindMe,
              onTap: () => bloc.add(
                RemindersSettingsEvent.toggled(enabled: !state.enabled),
              ),
              trailing: KansoSwitch(
                value: state.enabled,
                onChanged: (on) =>
                    bloc.add(RemindersSettingsEvent.toggled(enabled: on)),
              ),
            ),
          ],
        );
      },
    );
  }
}
