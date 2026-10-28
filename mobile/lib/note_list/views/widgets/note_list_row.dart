// Kanso — note_list_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/models/data/kanso_swipe_action.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/status_label.dart';
import '../../../core/utils/tag_row_color.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_note_row.dart';
import '../../../core/views/widgets/kanso_swipe_actions.dart';
import '../../../core/views/widgets/kanso_tag_chip.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'move_sheet.dart';

/// A note of the library as a row of the note list: its title, a locked
/// note's lock, its text, the notebook it is in, its status and tags, and
/// when it was last changed.
///
/// A locked note shows its title and lock and nothing else of it.
///
/// A swipe towards the end pins or unpins it (all the way, at once); a swipe
/// the other way shows Move and Trash (all the way trashes it). In the
/// trash the only action is Move, which restores it. A long press starts
/// choosing several notes, and while that goes on there are no swipes. Where
/// there are tabs a right click (a secondary tap) offers Open in New Tab.
class NoteListRow extends StatelessWidget {
  const NoteListRow({
    required this.row,
    required this.index,
    required this.time,
    this.open = false,
    this.checked,
    this.onTap,
    this.onLongPress,
    this.onOpenInNewTab,
    super.key,
  });

  final NoteRow row;

  /// The notebooks and tags the row's names are looked up in.
  final LibraryIndexState index;

  /// When the note was last changed, as the list says it.
  final String time;

  /// Whether the note is the one open beside the list.
  final bool open;

  /// While several notes are being chosen, whether this one is.
  final bool? checked;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  /// Opens the note in a new tab: offered in a menu on a right click (a
  /// secondary tap), only where there are tabs.
  final VoidCallback? onOpenInNewTab;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<NoteListBloc>();
    final unlocked =
        row.locked &&
        context.select<LockStatusBloc, bool>((lock) => lock.state.unlocked);
    final inTrash = context.select<NoteListBloc, bool>(
      (list) => list.state.inTrash,
    );
    void pin() => bloc.add(
      NoteListEvent.changeRequested(const NoteChange.togglePin(), <String>[
        row.id,
      ]),
    );
    void trash() => bloc.add(
      NoteListEvent.changeRequested(const NoteChange.trash(), <String>[row.id]),
    );
    final newTab = onOpenInNewTab;
    final swipeable = KansoSwipeActions(
      id: row.id,
      enabled: checked == null,
      startActions: inTrash
          ? const <KansoSwipeAction>[]
          : <KansoSwipeAction>[
              KansoSwipeAction(
                icon: KansoIcons.pin,
                label: row.pinned ? l10n.noteListUnpin : l10n.noteListPin,
                background: colors.accent,
                onPressed: pin,
              ),
            ],
      endActions: <KansoSwipeAction>[
        KansoSwipeAction(
          icon: KansoIcons.notebook,
          label: l10n.noteListMove,
          background: colors.warning,
          onPressed: () =>
              MoveSheet.show(context, bloc: bloc, ids: <String>[row.id]),
        ),
        if (!inTrash)
          KansoSwipeAction(
            icon: KansoIcons.trash,
            label: l10n.noteListTrash,
            background: colors.danger,
            onPressed: trash,
          ),
      ],
      onCommitStart: inTrash ? null : pin,
      onCommitEnd: inTrash ? null : trash,
      onArmed: () => bloc.add(const NoteListEvent.swipeArmed()),
      child: KansoNoteRow(
        title: row.title.trim().isEmpty ? l10n.noteUntitled : row.title,
        excerpt: !row.locked
            ? row.snippet
            : (unlocked ? l10n.noteListUnlocked : l10n.noteListLocked),
        notebook: index.notebookPath(row.notebookId) ?? '',
        time: time,
        status: noteStatusOf(row.status),
        pinned: row.pinned,
        locked: row.locked,
        unlocked: unlocked,
        selected: open,
        checked: checked,
        onTap: onTap,
        onLongPress: (checked ?? false) ? null : onLongPress,
        tags: <KansoTagChip>[
          for (final id in row.tags)
            if (index.tag(id) case final tag?)
              KansoTagChip(name: tag.name, color: tag.tint),
        ],
      ),
    );
    Widget rowContent = swipeable;
    if (newTab != null) {
      rowContent = GestureDetector(
        onSecondaryTapUp: (details) async {
          final chosen = await showKansoMenu<bool>(
            context,
            details.globalPosition,
            <PopupMenuEntry<bool>>[
              kansoMenuItem<bool>(context, true, l10n.tabOpenInNewTab),
            ],
          );
          if (chosen ?? false) {
            newTab();
          }
        },
        child: swipeable,
      );
    }
    if (checked != true) {
      return rowContent;
    }
    final selectedRows = bloc.state.selectedRows;
    final data = NoteDragData(
      noteIds: <String>[for (final selected in selectedRows) selected.id],
      titles: <String>[
        for (final selected in selectedRows)
          selected.locked ? '' : selected.title,
      ],
    );
    return LongPressDraggable<NoteDragData>(
      data: data,
      feedback: _NoteDragFeedback(
        titles: data.titles,
        count: data.noteIds.length,
      ),
      childWhenDragging: Opacity(opacity: 0.55, child: rowContent),
      child: rowContent,
    );
  }
}

class _NoteDragFeedback extends StatelessWidget {
  const _NoteDragFeedback({required this.titles, required this.count});

  final List<String> titles;
  final int count;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final visibleTitles = <String>[
      for (final title in titles)
        title.isEmpty
            ? l10n.noteListLocked
            : title.trim().isEmpty
            ? l10n.noteUntitled
            : title,
    ];
    final summary = visibleTitles.take(2).join(', ');
    return Material(
      color: Colors.transparent,
      child: Chip(
        avatar: KansoIcon(
          titles.every((title) => title.isEmpty)
              ? KansoIcons.lock
              : KansoIcons.fileText,
          size: 16,
          color: colors.textSecondary,
        ),
        label: Text('$count · $summary'),
        backgroundColor: colors.sidebar,
        side: BorderSide(color: colors.divider),
      ),
    );
  }
}
