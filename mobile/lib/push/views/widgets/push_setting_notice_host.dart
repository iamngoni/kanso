// Kanso — push_setting_notice_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../state/push_setting/push_setting_bloc.dart';
import 'push_setting_notice.dart';

/// Floats the [PushSettingNotice] at the foot of its [child] while the
/// wake setting's last change has failed. It needs a [PushSettingBloc] above
/// it.
class PushSettingNoticeHost extends StatelessWidget {
  const PushSettingNoticeHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<PushSettingBloc, PushSettingState>(
      buildWhen: (before, after) =>
          (before.error != null) != (after.error != null),
      builder: (context, setting) {
        return Stack(
          children: <Widget>[
            Positioned.fill(child: child),
            if (setting.error != null)
              PositionedDirectional(
                start: KansoSpacing.md,
                end: KansoSpacing.md,
                bottom: MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
                child: PushSettingNotice(
                  onDismiss: () => context.read<PushSettingBloc>().add(
                    const PushSettingEvent.errorDismissed(),
                  ),
                ),
              ),
          ],
        );
      },
    );
  }
}
