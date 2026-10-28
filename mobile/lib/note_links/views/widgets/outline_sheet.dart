// Kanso — outline_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/linking_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../models/repos/abs/note_links_repository.dart';

/// The headings of a note, in order, in a sheet: a way through a long note.
/// A heading opens the note on its line.
class OutlineSheet extends StatelessWidget {
  const OutlineSheet({
    required this.noteId,
    required this.text,
    this.links,
    super.key,
  });

  final String noteId;
  final String text;

  /// What reads the outline; the app's own when not given.
  final NoteLinksRepository? links;

  /// Shows the outline of [text] over the whole app.
  static Future<void> show(
    BuildContext context, {
    required String noteId,
    required String text,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => OutlineSheet(noteId: noteId, text: text),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final headings = (links ?? getIt<NoteLinksRepository>()).outline(text);
    void openAt(int line) {
      final router = GoRouter.of(context);
      Navigator.of(context).pop();
      unawaited(
        router.push(
          '${KansoRoutes.notes}/note/${Uri.encodeComponent(noteId)}'
          '?${KansoRoutes.taskLineParameter}=$line',
        ),
      );
    }

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
          child: Text(l10n.noteOutline, style: context.typography.barTitle),
        ),
        Flexible(
          child: headings.fold(
            (error) => Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: KansoInlineMessage(message: error.message),
            ),
            (found) => found.isEmpty
                ? Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: KansoEmptyState(
                      title: l10n.noteOutlineEmptyTitle,
                      message: l10n.noteOutlineEmptyMessage,
                    ),
                  )
                : ListView(
                    shrinkWrap: true,
                    padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
                    children: <Widget>[
                      KansoSheetGroup(
                        dividerIndent: KansoSettingsRow.iconInset,
                        children: <Widget>[
                          for (final heading in found)
                            _OutlineRow(
                              heading: heading,
                              onTap: () => openAt(heading.line),
                            ),
                        ],
                      ),
                    ],
                  ),
          ),
        ),
      ],
    );
  }
}

/// One heading, stepped in by its level.
class _OutlineRow extends StatelessWidget {
  const _OutlineRow({required this.heading, required this.onTap});

  final OutlineHeading heading;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsetsDirectional.only(
        start: (heading.level - 1).clamp(0, 5) * KansoSpacing.sm,
      ),
      child: KansoSettingsRow(label: heading.text, onTap: onTap),
    );
  }
}
