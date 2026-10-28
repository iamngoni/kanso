// Kanso — note_menu_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../window/models/repos/abs/note_window_repository.dart';

/// What can be done with the open note, in a sheet. For now: show it in a
/// window of its own, which only a device that shows more than one window
/// offers (the bar leaves the menu out where there is nothing in it).
///
/// The sheet is above the pages' own providers: it is given what it needs.
class NoteMenuSheet extends StatelessWidget {
  const NoteMenuSheet({required this.noteId, required this.windows, super.key});

  final String noteId;
  final NoteWindowRepository windows;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.sm,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: Text(l10n.noteMenu, style: context.typography.barTitle),
          ),
          if (windows.supported)
            KansoSettingsRow(
              icon: KansoIcons.externalLink,
              label: l10n.noteOpenInNewWindow,
              onTap: () {
                Navigator.of(context).pop();
                unawaited(windows.open(noteId));
              },
            ),
        ],
      ),
    );
  }
}
