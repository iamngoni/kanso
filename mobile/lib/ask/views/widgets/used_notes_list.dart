// Kanso — used_notes_list.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/ai_note_ref.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// "Used 3 notes", which opens to the notes an answer was written from; a
/// tap on one opens it.
class UsedNotesList extends StatefulWidget {
  const UsedNotesList({
    required this.notes,
    required this.onOpen,
    this.initiallyExpanded = false,
    super.key,
  });

  final List<AiNoteRef> notes;
  final ValueChanged<String> onOpen;
  final bool initiallyExpanded;

  @override
  State<UsedNotesList> createState() => _UsedNotesListState();
}

class _UsedNotesListState extends State<UsedNotesList> {
  late final ValueNotifier<bool> expanded = ValueNotifier<bool>(
    widget.initiallyExpanded,
  );

  @override
  void dispose() {
    expanded.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return ValueListenableBuilder<bool>(
      valueListenable: expanded,
      builder: (context, open, _) {
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoPressable(
              onTap: () => expanded.value = !open,
              semanticLabel: l10n.askUsedNotes(widget.notes.length),
              child: SizedBox(
                height: KansoSizes.touchTarget,
                child: Row(
                  children: <Widget>[
                    KansoIcon(
                      open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                      size: KansoSizes.iconSmall,
                      color: colors.textMuted,
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    Text(
                      l10n.askUsedNotes(widget.notes.length),
                      style: typography.sectionLabel,
                    ),
                  ],
                ),
              ),
            ),
            if (open)
              for (final note in widget.notes)
                KansoPressable(
                  onTap: () => widget.onOpen(note.id),
                  child: SizedBox(
                    height: KansoSizes.touchTarget,
                    child: Row(
                      children: <Widget>[
                        KansoIcon(
                          KansoIcons.fileText,
                          size: KansoSizes.iconSmall,
                          color: colors.textSecondary,
                        ),
                        const SizedBox(width: KansoSpacing.xs),
                        Expanded(
                          child: Text(
                            note.title.trim().isEmpty
                                ? l10n.askUntitled
                                : note.title,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: typography.body.copyWith(
                              color: colors.accent,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
          ],
        );
      },
    );
  }
}
