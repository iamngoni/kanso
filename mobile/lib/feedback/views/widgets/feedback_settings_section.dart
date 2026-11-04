// Kanso — feedback_settings_section.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/my_feedback/my_feedback_bloc.dart';
import '../../utils/feedback_labels.dart';

/// Settings › Feedback, above About: Send feedback, and My feedback with the
/// newest report's status (or "Fixed in 0.9.2" once a report was fixed and
/// not yet looked at). Settings only reads what the phone kept; the server
/// is asked when My feedback opens and at launch.
class FeedbackSettingsSection extends StatelessWidget {
  const FeedbackSettingsSection({required this.dividerIndent, super.key});

  /// How far the hairlines are inset from the start.
  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<MyFeedbackBloc, MyFeedbackState>(
      builder: (context, state) {
        final l10n = context.l10n;
        final bloc = context.read<MyFeedbackBloc>();
        final fixed = state.freshRelease;
        final newest = state.newest;
        final value = fixed != null
            ? l10n.feedbackFixedIn(fixed)
            : newest == null
            ? null
            : l10n.feedbackChipLabel(newest);
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoGroupedSection(
              title: l10n.feedbackSection,
              footer: l10n.feedbackSectionFooter,
              dividerIndent: dividerIndent,
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.messageSquare,
                  label: l10n.feedbackSend,
                  chevron: true,
                  // Back from the page, the row says what the phone now knows.
                  onTap: () => context
                      .push(KansoRoutes.settingsFeedbackNew)
                      .then(
                        (_) => bloc.add(const MyFeedbackEvent.started()),
                      ),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.inbox,
                  label: l10n.feedbackMine,
                  value: value,
                  chevron: true,
                  // Back from the list, the row says what the phone now knows.
                  onTap: () => context
                      .push(KansoRoutes.settingsFeedback)
                      .then(
                        (_) => bloc.add(const MyFeedbackEvent.started()),
                      ),
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}
