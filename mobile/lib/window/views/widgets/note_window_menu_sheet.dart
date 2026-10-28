// Kanso — note_window_menu_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../models/data/note_window_request.dart';
import '../../models/repos/abs/note_window_repository.dart';

/// The menu of a note's own window: what it can't do itself it hands to the
/// main window (a new note, a search), and Close Window.
///
/// The sheet is above the page's providers: it is given what it needs.
class NoteWindowMenuSheet extends StatelessWidget {
  const NoteWindowMenuSheet({required this.windows, super.key});

  final NoteWindowRepository windows;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    void hand(NoteWindowRequest request) {
      Navigator.of(context).pop();
      unawaited(windows.hand(request));
    }

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
          KansoSettingsRow(
            icon: KansoIcons.plus,
            label: l10n.noteWindowNewNote,
            onTap: () => hand(const NoteWindowRequest.newNote()),
          ),
          KansoSettingsRow(
            icon: KansoIcons.search,
            label: l10n.noteWindowSearch,
            onTap: () => hand(const NoteWindowRequest.search()),
          ),
          KansoSettingsRow(
            icon: KansoIcons.close,
            label: l10n.noteWindowClose,
            onTap: () {
              Navigator.of(context).pop();
              unawaited(windows.closeThisWindow());
            },
          ),
        ],
      ),
    );
  }
}
