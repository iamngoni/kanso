// Kanso — undo_notice_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_motion.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_undo_notice.dart';
import '../../../note_list/views/widgets/note_list_notice.dart';
import 'undo_notice_timeout.dart';

/// Shows the notice of the `UndoNoticeBloc` over [child], afloat at the
/// foot of the screen: over the list and the note, in every layout.
///
/// A failed undo shows as the list's own error line does.
class UndoNoticeHost extends StatelessWidget {
  const UndoNoticeHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        child,
        PositionedDirectional(
          start: KansoSpacing.md,
          end: KansoSpacing.md,
          bottom: MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
          child: Align(
            alignment: Alignment.bottomCenter,
            child: ConstrainedBox(
              constraints: const BoxConstraints(
                maxWidth: KansoSizes.noticeWidth,
              ),
              child: BlocBuilder<UndoNoticeBloc, UndoNoticeState>(
                builder: (context, state) {
                  final bloc = context.read<UndoNoticeBloc>();
                  final notice = state.notice;
                  final error = state.error;
                  final Widget shown;
                  if (error != null) {
                    shown = NoteListNotice(
                      key: const ValueKey<String>('error'),
                      message: error is LockedError
                          ? context.l10n.noteListChangeLocked
                          : context.l10n.noteListChangeFailed,
                      onDismiss: () =>
                          bloc.add(const UndoNoticeEvent.errorDismissed()),
                    );
                  } else if (notice != null) {
                    shown = UndoNoticeTimeout(
                      key: ValueKey<int>(notice.serial),
                      serial: notice.serial,
                      child: KansoUndoNotice(
                        message: notice.message,
                        busy: state.undoing,
                        onUndo: () =>
                            bloc.add(const UndoNoticeEvent.undoPressed()),
                      ),
                    );
                  } else {
                    shown = const SizedBox.shrink(
                      key: ValueKey<String>('none'),
                    );
                  }
                  return AnimatedSwitcher(
                    duration: KansoMotion.resolve(context, KansoMotion.short),
                    switchInCurve: KansoMotion.enter,
                    switchOutCurve: KansoMotion.exit,
                    transitionBuilder: (child, animation) =>
                        FadeTransition(opacity: animation, child: child),
                    child: shown,
                  );
                },
              ),
            ),
          ),
        ),
      ],
    );
  }
}
