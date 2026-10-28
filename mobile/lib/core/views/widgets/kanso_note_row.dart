// Kanso — kanso_note_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../models/data/note_status.dart';
import '../../utils/context_extensions.dart';
import 'kanso_checkbox.dart';
import 'kanso_icon.dart';
import 'kanso_lock_mark.dart';
import 'kanso_pressable.dart';
import 'kanso_status_glyph.dart';
import 'kanso_tag_chip.dart';

/// A note in a list: its title, two lines of its text, and a quiet line
/// with its notebook, status, tags and time.
///
/// The row has no divider of its own; the list draws one between rows.
/// [selected] is the note open beside the list on a tablet. While several
/// notes are being chosen, [checked] says whether this one is: a check box
/// stands before the row; with no [checked] there is none.
class KansoNoteRow extends StatelessWidget {
  const KansoNoteRow({
    required this.title,
    required this.excerpt,
    required this.notebook,
    required this.time,
    this.status = NoteStatus.none,
    this.tags = const <KansoTagChip>[],
    this.pinned = false,
    this.locked = false,
    this.unlocked = false,
    this.selected = false,
    this.checked,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String title;
  final String excerpt;
  final String notebook;
  final String time;
  final NoteStatus status;
  final List<KansoTagChip> tags;
  final bool pinned;
  final bool locked;

  /// Whether a locked note is open now: its lock is drawn open.
  final bool unlocked;
  final bool selected;
  final bool? checked;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final choice = checked;
    final text = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Row(
          children: <Widget>[
            Expanded(
              child: Text(
                title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: typography.listTitle,
              ),
            ),
            if (locked) ...<Widget>[
              const SizedBox(width: KansoSpacing.xs),
              KansoLockMark(unlocked: unlocked),
            ],
            if (pinned) ...<Widget>[
              const SizedBox(width: KansoSpacing.xs),
              KansoIcon(
                KansoIcons.pin,
                size: KansoSizes.glyph,
                color: colors.accent,
              ),
            ],
          ],
        ),
        const SizedBox(height: KansoSpacing.xxs),
        Text(
          excerpt,
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: typography.secondary,
        ),
        const SizedBox(height: KansoSpacing.xs),
        Row(
          children: <Widget>[
            Expanded(
              child: Row(
                children: <Widget>[
                  KansoIcon(
                    KansoIcons.notebook,
                    size: KansoSizes.glyph,
                    color: colors.textMuted,
                  ),
                  const SizedBox(width: KansoSpacing.xxs),
                  Flexible(
                    child: Text(
                      notebook,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta,
                    ),
                  ),
                  if (status != NoteStatus.none) ...<Widget>[
                    const SizedBox(width: KansoSpacing.xs),
                    KansoStatusGlyph(status: status),
                  ],
                  for (final tag in tags) ...<Widget>[
                    const SizedBox(width: KansoSpacing.xs),
                    tag,
                  ],
                ],
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            Text(time, maxLines: 1, style: typography.meta),
          ],
        ),
      ],
    );
    return KansoPressable(
      onTap: onTap,
      onLongPress: onLongPress,
      child: Container(
        color: selected || (choice ?? false) ? colors.rowActive : null,
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: choice == null
            ? text
            : Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Padding(
                    padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                    child: IgnorePointer(child: KansoCheckbox(value: choice)),
                  ),
                  const SizedBox(width: KansoSpacing.sm),
                  Expanded(child: text),
                ],
              ),
      ),
    );
  }
}
