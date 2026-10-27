// Kanso — note_info_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/list_labels.dart';
import '../../../core/utils/status_label.dart';
import 'note_metadata_block.dart';

/// The facts of the note, in a sheet: where it is, its status and tags, and
/// when it was made and last changed. In the editor they are one tap away
/// rather than above the text.
///
/// The sheet is above the pages' own providers: it is given what it shows.
class NoteInfoSheet extends StatelessWidget {
  const NoteInfoSheet({required this.note, required this.index, super.key});

  final NoteView note;

  /// The notebooks and tags the facts are looked up in.
  final LibraryIndexState index;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final now = clock.now();
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(l10n.noteInfo, style: context.typography.barTitle),
          const SizedBox(height: KansoSpacing.sm),
          NoteMetadataBlock(
            notebook: index.notebookPath(note.notebookId) ?? '',
            status: noteStatusOf(note.status),
            tags: <TagRow>[
              for (final id in note.tags)
                if (index.tag(id) case final tag?) tag,
            ],
            created: dateLabel(l10n, note.createdMs),
            updated: dateTimeLabel(l10n, note.updatedMs, now: now),
          ),
        ],
      ),
    );
  }
}
