// Kanso — new_menu_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../models/data/new_choice.dart';

/// The menu of the list's new button: today's note and the calendar of the
/// journal, a note from a template and a new template. Choosing one closes
/// the sheet with it.
class NewMenuSheet extends StatelessWidget {
  const NewMenuSheet({super.key});

  /// Shows the menu over the whole app and completes with the choice, or
  /// null.
  static Future<NewChoice?> show(BuildContext context) =>
      showKansoBottomSheet<NewChoice>(
        context,
        builder: (_) => const NewMenuSheet(),
      );

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    void choose(NewChoice choice) => Navigator.of(context).pop(choice);
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      children: <Widget>[
        SheetHeader(title: l10n.newMenuTitle, actionLabel: l10n.searchCancel),
        KansoSettingsRow(
          icon: KansoIcons.calendar,
          label: l10n.journalTodaysNote,
          onTap: () => choose(NewChoice.today),
        ),
        KansoSettingsRow(
          icon: KansoIcons.book,
          label: l10n.journalCalendar,
          onTap: () => choose(NewChoice.calendar),
        ),
        KansoSettingsRow(
          icon: KansoIcons.layoutTemplate,
          label: l10n.newFromTemplate,
          onTap: () => choose(NewChoice.fromTemplate),
        ),
        KansoSettingsRow(
          icon: KansoIcons.plus,
          label: l10n.newTemplate,
          onTap: () => choose(NewChoice.newTemplate),
        ),
      ],
    );
  }
}
