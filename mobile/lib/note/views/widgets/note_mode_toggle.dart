// Kanso — note_mode_toggle.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../models/data/note_mode.dart';

/// The switch between a note to read and a note as text to edit.
class NoteModeToggle extends StatelessWidget {
  const NoteModeToggle({
    required this.mode,
    required this.onChanged,
    super.key,
  });

  final NoteMode mode;
  final ValueChanged<NoteMode> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoSegmented<NoteMode>(
      selected: mode,
      semanticLabel: l10n.noteModeLabel,
      onChanged: onChanged,
      segments: <KansoSegment<NoteMode>>[
        KansoSegment<NoteMode>(
          value: NoteMode.preview,
          icon: KansoIcons.eye,
          semanticLabel: l10n.notePreview,
        ),
        KansoSegment<NoteMode>(
          value: NoteMode.edit,
          icon: KansoIcons.pencil,
          semanticLabel: l10n.noteEdit,
        ),
      ],
    );
  }
}
