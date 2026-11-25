// Kanso — word_goal_default_section.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/bridge/api/word_goal.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../palette/models/data/palette_choice.dart';
import '../../../palette/views/widgets/palette_choice_sheet.dart';
import '../../../shell/state/focus_mode/focus_mode_bloc.dart';
import '../../state/word_goal_settings/word_goal_settings_bloc.dart';
import 'focus_live_preview_row.dart';

/// Settings › Editor: the goal notes follow when they do not set their own,
/// and whether focus mode shows notes in Live.
class WordGoalDefaultSection extends StatelessWidget {
  const WordGoalDefaultSection({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    // Focus mode's Live Preview switch sits here where the app has focus
    // mode.
    final focusMode = _hasFocusMode(context);
    return BlocBuilder<WordGoalSettingsBloc, WordGoalSettingsState>(
      builder: (context, state) {
        final value = switch (state.status) {
          LoadStatus.loading => l10n.editorWordGoalLoading,
          LoadStatus.failed => l10n.editorWordGoalUnavailable,
          LoadStatus.ready =>
            state.words == null
                ? l10n.editorWordGoalNone
                : l10n.editorWordGoalWords(
                    wordGoalFormatCount(count: state.words!),
                  ),
        };
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoGroupedSection(
              title: l10n.editorWritingSection,
              footer: l10n.editorDefaultWordGoalDetail,
              children: <Widget>[
                if (focusMode) const FocusLivePreviewRow(),
                KansoSettingsRow(
                  icon: KansoIcons.target,
                  label: l10n.editorDefaultWordGoal,
                  value: value,
                  chevron: true,
                  onTap: state.status == LoadStatus.ready
                      ? () => _choose(context, state.words)
                      : null,
                ),
                KansoSettingsRow(
                  icon: KansoIcons.target,
                  label: l10n.editorTickNested,
                  value: _tickLabel(context, state.tickNested),
                  chevron: true,
                  onTap: state.status == LoadStatus.ready
                      ? () => _chooseTick(context, state.tickNested)
                      : null,
                ),
              ],
            ),
            if (state.error case final error?) ...<Widget>[
              const SizedBox(height: KansoSpacing.xs),
              Padding(
                padding: const EdgeInsetsDirectional.only(
                  start: KansoSpacing.md,
                  end: KansoSpacing.md,
                ),
                child: Text(error.message, style: context.typography.meta),
              ),
            ],
          ],
        );
      },
    );
  }

  bool _hasFocusMode(BuildContext context) {
    try {
      context.read<FocusModeBloc>();
      return true;
    } on ProviderNotFoundException {
      return false;
    }
  }

  String _tickLabel(BuildContext context, TickNested setting) =>
      switch (setting) {
        TickNested.ask => context.l10n.tickNestedAsk,
        TickNested.always => context.l10n.tickNestedAlways,
        TickNested.never => context.l10n.tickNestedNever,
      };

  Future<void> _chooseTick(BuildContext context, TickNested current) async {
    final picked = await PaletteChoiceSheet.show(
      context,
      title: context.l10n.editorTickNested,
      choices: <PaletteChoice>[
        for (final setting in TickNested.values)
          PaletteChoice(
            label: _tickLabel(context, setting),
            value: setting.name,
            checked: current == setting,
          ),
      ],
    );
    if (picked == null || !context.mounted) {
      return;
    }
    final setting = TickNested.values.asNameMap()[picked];
    if (setting != null) {
      context.read<WordGoalSettingsBloc>().add(
        WordGoalSettingsEvent.tickNestedChanged(setting),
      );
    }
  }

  Future<void> _choose(BuildContext context, int? current) async {
    final none = context.l10n.editorWordGoalNone;
    final picked = await PaletteChoiceSheet.show(
      context,
      title: context.l10n.editorDefaultWordGoal,
      choices: <PaletteChoice>[
        PaletteChoice(
          label: none,
          value: 'none',
          checked: current == null,
        ),
        for (final words in wordGoalPresets())
          PaletteChoice(
            label: context.l10n.editorWordGoalWords(
              wordGoalFormatCount(count: words),
            ),
            value: '$words',
            checked: current == words,
          ),
      ],
    );
    if (picked == null || !context.mounted) {
      return;
    }
    context.read<WordGoalSettingsBloc>().add(
      WordGoalSettingsEvent.defaultChanged(
        picked == 'none' ? null : int.tryParse(picked),
      ),
    );
  }
}
