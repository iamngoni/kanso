// Kanso — capture_task_section.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart' show CaptureTarget;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../state/capture_task/capture_task_cubit.dart';

/// Settings › Captured tasks go to: today's daily note or the Inbox, the
/// desktop's setting of the same name, for the capture sheet's Task mode.
/// It needs the [CaptureTaskCubit] above it.
class CaptureTaskSection extends StatelessWidget {
  const CaptureTaskSection({required this.dividerIndent, super.key});

  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<CaptureTaskCubit, CaptureTaskState>(
      buildWhen: (before, after) => before.target != after.target,
      builder: (context, state) {
        final cubit = context.read<CaptureTaskCubit>();
        return KansoGroupedSection(
          title: l10n.settingsCaptureTasks,
          footer: l10n.settingsCaptureTasksDetail,
          dividerIndent: dividerIndent,
          children: <Widget>[
            KansoCheckRow(
              icon: KansoIcons.calendar,
              label: l10n.settingsCaptureTasksDaily,
              checked: state.target == CaptureTarget.dailyNote,
              onTap: () => cubit.targetChosen(CaptureTarget.dailyNote),
            ),
            KansoCheckRow(
              icon: KansoIcons.inbox,
              label: l10n.settingsCaptureTasksInbox,
              checked: state.target == CaptureTarget.inbox,
              onTap: () => cubit.targetChosen(CaptureTarget.inbox),
            ),
          ],
        );
      },
    );
  }
}
