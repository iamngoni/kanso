// Kanso — note_lock_menu_button.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart' hide LockState;
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../lock/models/data/note_menu_action.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../lock/utils/note_menu_actions.dart';
import '../../../lock/views/widgets/note_more_sheet.dart';
import '../../state/note/note_bloc.dart';

/// The More button of a note, which opens the menu of its lock actions, and
/// of the actions in [moved] that the bar it is in had no room for (see
/// `KansoTopBarItem`, whose id is the action), and of the actions in [extra].
/// It is not there while the note is not read, or has nothing to offer (a
/// locked note that is closed, with no [extra]).
class NoteLockMenuButton extends StatelessWidget {
  const NoteLockMenuButton({
    this.moved = const {},
    this.extra = const {},
    super.key,
  });

  /// What each action the bar may have no room for does.
  final Map<NoteMenuAction, VoidCallback> moved;

  /// Actions the menu always offers, and what each does.
  final Map<NoteMenuAction, VoidCallback> extra;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<NoteBloc, NoteState>(
      builder: (context, note) {
        final view = note.note;
        if (view == null) {
          return const SizedBox.shrink();
        }
        return BlocBuilder<LockStatusBloc, LockStatusState>(
          builder: (context, lock) {
            final actions = noteMenuActions(
              lock: view.lock,
              unlocked: lock.unlocked,
            );
            if (actions.isEmpty && extra.isEmpty) {
              return const SizedBox.shrink();
            }
            return KansoIconButton(
              icon: KansoIcons.ellipsis,
              semanticLabel: context.l10n.lockMoreMenu,
              onPressed: () => NoteMoreSheet.show(
                context,
                noteId: view.id,
                lock: view.lock,
                status: context.read<LockStatusBloc>(),
                moved: _hidden(context),
                extra: extra,
              ),
            );
          },
        );
      },
    );
  }

  /// The actions of [moved] that gave way in the bar, in the bar's order.
  Map<NoteMenuAction, VoidCallback> _hidden(BuildContext context) {
    final hidden = KansoTopBar.hiddenOf(context);
    return <NoteMenuAction, VoidCallback>{
      for (final MapEntry(:key, :value) in moved.entries)
        if (hidden.contains(key)) key: value,
    };
  }
}
