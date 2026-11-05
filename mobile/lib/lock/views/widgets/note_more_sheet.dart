// Kanso — note_more_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart' hide LockState;

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../note/views/pages/presentation_page.dart';
import '../../../note/views/widgets/note_html_export.dart';
import '../../../note/views/widgets/note_pdf_export.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/note_menu_action.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../utils/note_menu_actions.dart';
import 'lock_item_sheet.dart';
import 'remove_lock_sheet.dart';

/// A note's More menu: first what the note's bar had no room for, then the
/// [show] caller's extra actions (Open in New Window, Record here), then
/// Export as PDF, and as far as locks go Lock note…, Remove lock… (the note
/// is locked and open) and Lock notes now (the notes are open). The choice
/// closes the menu and then runs.
class NoteMoreSheet extends StatelessWidget {
  const NoteMoreSheet({required this.actions, super.key});

  final List<NoteMenuAction> actions;

  /// Shows the menu of the note [noteId], whose lock is [lock], and runs
  /// what is chosen. [moved] are the actions the note's bar had no room
  /// for, in order, and what each does. [extra] are offered every time,
  /// after them. Nothing is shown when there is nothing to offer.
  static Future<void> show(
    BuildContext context, {
    required String noteId,
    required LockState lock,
    required LockStatusBloc status,
    Map<NoteMenuAction, VoidCallback> moved = const {},
    Map<NoteMenuAction, VoidCallback> extra = const {},
  }) async {
    final actions = <NoteMenuAction>[
      ...moved.keys,
      ...extra.keys,
      ...noteMenuActions(lock: lock, unlocked: status.state.unlocked),
    ];
    if (actions.isEmpty) {
      return;
    }
    final chosen = await showKansoBottomSheet<NoteMenuAction>(
      context,
      builder: (_) => NoteMoreSheet(actions: actions),
    );
    if (chosen == null || !context.mounted) {
      return;
    }
    if (moved[chosen] ?? extra[chosen] case final run?) {
      run();
      return;
    }
    switch (chosen) {
      case NoteMenuAction.exportPdf:
        await exportNotePdf(context, noteId: noteId);
      case NoteMenuAction.exportHtml:
        await exportNoteHtml(context, noteId: noteId);
      case NoteMenuAction.present:
        await PresentationPage.show(context, noteId: noteId);
      case NoteMenuAction.lockNote:
        await LockItemSheet.show(
          context,
          LockTarget.note(noteId),
          status: status,
        );
      case NoteMenuAction.removeLock:
        await RemoveLockSheet.show(
          context,
          LockTarget.note(noteId),
          status: status,
        );
      case NoteMenuAction.lockNow:
        status.add(const LockStatusEvent.lockedNow());
      // Run above, from [moved] or [extra].
      case NoteMenuAction.openInNewWindow ||
          NoteMenuAction.record ||
          NoteMenuAction.renameSpeakers ||
          NoteMenuAction.summarizeRecording ||
          NoteMenuAction.info ||
          NoteMenuAction.comments ||
          NoteMenuAction.share ||
          NoteMenuAction.previousDay ||
          NoteMenuAction.nextDay ||
          NoteMenuAction.calendar:
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
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
          child: Text(l10n.lockMoreTitle, style: context.typography.barTitle),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            for (final action in actions)
              KansoSettingsRow(
                icon: switch (action) {
                  NoteMenuAction.exportPdf => KansoIcons.download,
                  NoteMenuAction.exportHtml => KansoIcons.globe,
                  NoteMenuAction.present => KansoIcons.presentation,
                  NoteMenuAction.lockNote => KansoIcons.lock,
                  NoteMenuAction.removeLock => KansoIcons.lockOpen,
                  NoteMenuAction.lockNow => KansoIcons.lock,
                  NoteMenuAction.openInNewWindow => KansoIcons.externalLink,
                  NoteMenuAction.record => KansoIcons.mic,
                  NoteMenuAction.renameSpeakers => KansoIcons.users,
                  NoteMenuAction.summarizeRecording => KansoIcons.sparkles,
                  NoteMenuAction.info => KansoIcons.info,
                  NoteMenuAction.comments => KansoIcons.messageSquare,
                  NoteMenuAction.share => KansoIcons.share,
                  NoteMenuAction.previousDay => KansoIcons.chevronLeft,
                  NoteMenuAction.nextDay => KansoIcons.chevronRight,
                  NoteMenuAction.calendar => KansoIcons.calendar,
                },
                label: switch (action) {
                  NoteMenuAction.exportPdf => l10n.noteExportPdf,
                  NoteMenuAction.exportHtml => l10n.noteExportHtml,
                  NoteMenuAction.present => l10n.notePresent,
                  NoteMenuAction.lockNote => l10n.lockNoteMenu,
                  NoteMenuAction.removeLock => l10n.lockRemoveNoteMenu,
                  NoteMenuAction.lockNow => l10n.lockNowMenu,
                  NoteMenuAction.openInNewWindow => l10n.noteOpenInNewWindow,
                  NoteMenuAction.record => l10n.recordingMenuRecordHere,
                  NoteMenuAction.renameSpeakers => l10n.recordingSpeakersMenu,
                  NoteMenuAction.summarizeRecording =>
                    l10n.recordingSummaryMenu,
                  NoteMenuAction.info => l10n.noteInfo,
                  NoteMenuAction.comments => l10n.commentsTitle,
                  NoteMenuAction.share => l10n.shareTitle,
                  NoteMenuAction.previousDay => l10n.journalPreviousDay,
                  NoteMenuAction.nextDay => l10n.journalNextDay,
                  NoteMenuAction.calendar => l10n.journalCalendar,
                },
                danger: action == NoteMenuAction.removeLock,
                onTap: () => Navigator.of(context).pop(action),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}
