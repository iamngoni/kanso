// Kanso — remove_lock_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../injection.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/remove_status.dart';
import '../../state/lock_remove/lock_remove_bloc.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import 'lock_form_sheet.dart';

/// Asks whether to take a lock off an open note ("Remove the lock?") or
/// notebook ("Remove the notebook's lock?"), says what that means, and
/// shows the library's own sentence when it can't be done.
class RemoveLockSheet extends StatelessWidget {
  const RemoveLockSheet({required this.target, super.key});

  final LockTarget target;

  /// Shows the sheet for [target] over the whole app.
  static Future<void> show(
    BuildContext context,
    LockTarget target, {
    required LockStatusBloc status,
  }) async {
    final bloc = getIt<LockRemoveBloc>();
    try {
      await showKansoBottomSheet<void>(
        context,
        builder: (_) => MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<LockRemoveBloc>.value(value: bloc),
            BlocProvider<LockStatusBloc>.value(value: status),
          ],
          child: RemoveLockSheet(target: target),
        ),
      );
    } finally {
      await bloc.close();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final note = target is NoteLockTarget;
    return BlocConsumer<LockRemoveBloc, LockRemoveState>(
      listener: (context, state) {
        if (state.status == RemoveStatus.done) {
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed());
          Navigator.of(context).pop();
        }
      },
      builder: (context, state) {
        final error = state.error;
        return LockFormSheet(
          title: note ? l10n.lockRemoveNoteTitle : l10n.lockRemoveNotebookTitle,
          message: note
              ? l10n.lockRemoveNoteMessage
              : l10n.lockRemoveNotebookMessage,
          // The library's own sentence says why, as it is: "The note is in
          // a locked notebook, so it stays locked. Move it out first."
          notice: switch (error) {
            null => null,
            InvalidError(:final message) => message,
            LockedError() => l10n.lockRemoveClosed,
            _ => l10n.lockActionFailed,
          },
          primaryLabel: l10n.lockRemoveButton,
          danger: true,
          working: state.status == RemoveStatus.working,
          onPrimary: () => context.read<LockRemoveBloc>().add(
            LockRemoveEvent.confirmed(target),
          ),
        );
      },
    );
  }
}
