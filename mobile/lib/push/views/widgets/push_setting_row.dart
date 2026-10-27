// Kanso — push_setting_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../models/data/push_setting_status.dart';
import '../../state/push_setting/push_setting_bloc.dart';

/// The switch "Wake this device when notes change elsewhere", with a line
/// in plain words under it. Where push can't work it is dimmed and says so.
///
/// It needs a [PushSettingBloc] above it.
class PushSettingRow extends StatelessWidget {
  const PushSettingRow({super.key});

  /// How far the line under the label starts: past the row's icon.
  static const double textInset =
      KansoSpacing.md + KansoSizes.icon + KansoSpacing.sm;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<PushSettingBloc, PushSettingState>(
      builder: (context, setting) {
        final ready = setting.status == PushSettingStatus.ready;
        final unavailable = setting.status == PushSettingStatus.unavailable;
        void toggle({required bool enabled}) =>
            context.read<PushSettingBloc>().add(
              PushSettingEvent.toggled(enabled: enabled),
            );
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.bell,
              label: l10n.settingsPushWake,
              onTap: ready ? () => toggle(enabled: !setting.enabled) : null,
              trailing: KansoSwitch(
                value: ready && setting.enabled,
                onChanged: ready ? (value) => toggle(enabled: value) : null,
              ),
            ),
            Padding(
              padding: const EdgeInsetsDirectional.only(
                start: textInset,
                end: KansoSpacing.md,
                bottom: KansoSpacing.sm,
              ),
              child: Text(
                unavailable
                    ? l10n.settingsPushUnavailable
                    : l10n.settingsPushWakeNote,
                style: context.typography.meta,
              ),
            ),
          ],
        );
      },
    );
  }
}
