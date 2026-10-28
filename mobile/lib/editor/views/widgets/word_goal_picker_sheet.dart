// Kanso — word_goal_picker_sheet.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/word_goal.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../palette/models/data/palette_choice.dart';
import '../../models/data/word_goal_choice.dart';

/// The desktop word-goal menu's presets, custom value, default and off states.
class WordGoalPickerSheet extends StatelessWidget {
  const WordGoalPickerSheet({
    required this.setting,
    required this.defaultGoal,
    super.key,
  });

  final WordGoalChoice setting;
  final int? defaultGoal;

  static Future<WordGoalChoice?> show(
    BuildContext context, {
    required WordGoalChoice setting,
    required int? defaultGoal,
  }) async {
    final value = await showKansoBottomSheet<String>(
      context,
      builder: (_) => WordGoalPickerSheet(
        setting: setting,
        defaultGoal: defaultGoal,
      ),
    );
    if (value == null || !context.mounted) {
      return null;
    }
    if (value == 'custom') {
      final words = await _custom(context);
      return words == null ? null : WordGoalWords(words);
    }
    if (value == 'default') {
      return const WordGoalDefault();
    }
    if (value == 'off') {
      return const WordGoalOff();
    }
    final words = int.tryParse(value);
    return words == null ? null : WordGoalWords(words);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final choices = <PaletteChoice>[
      for (final preset in wordGoalPresets())
        PaletteChoice(
          label: l10n.editorWordGoalWords(wordGoalFormatCount(count: preset)),
          value: '$preset',
          checked: switch (setting) {
            WordGoalWords(:final words) => words == preset,
            _ => false,
          },
        ),
      PaletteChoice(
        label: l10n.editorWordGoalCustom,
        value: 'custom',
        checked: switch (setting) {
          WordGoalWords(:final words) => !wordGoalPresets().contains(words),
          _ => false,
        },
      ),
      if (defaultGoal case final goal?)
        PaletteChoice(
          label: l10n.editorWordGoalDefault(
            wordGoalFormatCount(count: goal),
          ),
          value: 'default',
          checked: setting is WordGoalDefault,
        )
      else
        PaletteChoice(
          label: l10n.editorWordGoalNone,
          value: 'default',
          checked: setting is WordGoalDefault,
        ),
      if (defaultGoal != null)
        PaletteChoice(
          label: l10n.editorWordGoalNoGoalForNote,
          value: 'off',
          checked: setting is WordGoalOff,
        ),
    ];
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            l10n.editorWordGoalForNote,
            style: context.typography.barTitle,
          ),
        ),
        Flexible(
          child: ListView(
            shrinkWrap: true,
            padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
            children: <Widget>[
              KansoSheetGroup(
                dividerIndent: KansoSettingsRow.iconInset,
                children: <Widget>[
                  for (final choice in choices)
                    KansoSettingsRow(
                      icon: choice.checked
                          ? KansoIcons.circleCheck
                          : KansoIcons.circleDot,
                      label: choice.label,
                      onTap: () => Navigator.of(context).pop(choice.value),
                    ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  static Future<int?> _custom(BuildContext context) async {
    final controller = TextEditingController();
    final parsed = await showDialog<int>(
      context: context,
      builder: (context) => _CustomGoalDialog(controller: controller),
    );
    controller.dispose();
    return parsed;
  }
}

class _CustomGoalDialog extends StatefulWidget {
  const _CustomGoalDialog({required this.controller});

  final TextEditingController controller;

  @override
  State<_CustomGoalDialog> createState() => _CustomGoalDialogState();
}

class _CustomGoalDialogState extends State<_CustomGoalDialog> {
  final ValueNotifier<String?> error = ValueNotifier<String?>(null);

  @override
  void dispose() {
    error.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return AlertDialog(
      title: Text(l10n.editorWordGoalCustomTitle),
      content: ValueListenableBuilder<String?>(
        valueListenable: error,
        builder: (context, errorText, _) => TextField(
          controller: widget.controller,
          autofocus: true,
          keyboardType: TextInputType.text,
          decoration: InputDecoration(
            hintText: l10n.editorWordGoalCustomHint,
            errorText: errorText,
          ),
          onSubmitted: (_) => submit(),
        ),
      ),
      actions: <Widget>[
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.editorWordGoalCancel),
        ),
        FilledButton(
          onPressed: submit,
          child: Text(l10n.editorWordGoalSave),
        ),
      ],
    );
  }

  void submit() {
    final parsed = wordGoalParse(input: widget.controller.text);
    if (parsed == null) {
      error.value = context.l10n.editorWordGoalInvalid;
      return;
    }
    Navigator.of(context).pop(parsed);
  }
}
