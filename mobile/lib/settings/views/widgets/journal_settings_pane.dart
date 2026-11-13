// Kanso — journal_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../capture/views/widgets/capture_task_section.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../digest/views/widgets/brief_section.dart';
import '../../../reminders/views/widgets/reminders_section.dart';
import '../../../templates/views/widgets/daily_template_section.dart';
import 'settings_category_pane.dart';

/// Settings › Daily notes and reminders: the template daily notes start
/// from, where captured tasks go, task reminders, and the brief and weekly
/// review. It needs a `DailyTemplateBloc`, a `CaptureTaskCubit` and a
/// `RemindersSettingsBloc` above it.
class JournalSettingsPane extends StatelessWidget {
  const JournalSettingsPane({super.key});

  @override
  Widget build(BuildContext context) {
    return SettingsCategoryPane(
      title: context.l10n.settingsJournal,
      sections: const <Widget>[
        DailyTemplateSection(dividerIndent: SettingsCategoryPane.iconInset),
        CaptureTaskSection(dividerIndent: SettingsCategoryPane.iconInset),
        RemindersSection(dividerIndent: SettingsCategoryPane.iconInset),
        // Last: it brings its own space under it, and is left out where
        // there is no digest.
        BriefSection(dividerIndent: SettingsCategoryPane.iconInset),
      ],
    );
  }
}
