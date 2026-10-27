// Kanso — notebook_unlock_gate.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/unlock_status.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../state/unlock/unlock_bloc.dart';
import 'unlock_sheet.dart';

/// Shown around the notes of a locked notebook. It asks once: a biometric
/// when they are on, otherwise (or when that does not work) the sheet
/// "Unlock notebook". A person who dismisses the sheet is not asked again
/// until the list is shown again.
class NotebookUnlockGate extends StatefulWidget {
  const NotebookUnlockGate({
    required this.notebook,
    required this.child,
    super.key,
  });

  final NotebookRow notebook;
  final Widget child;

  @override
  State<NotebookUnlockGate> createState() => _NotebookUnlockGateState();
}

class _NotebookUnlockGateState extends State<NotebookUnlockGate> {
  bool asked = false;

  LockTarget get target =>
      LockTarget.notebook(widget.notebook.id, widget.notebook.name);

  @override
  Widget build(BuildContext context) {
    // Read here: a provider's create may not depend on the localizations.
    final reason = context.l10n.biometricReasonUnlock;
    return BlocProvider<UnlockBloc>(
      create: (_) =>
          getIt<UnlockBloc>()..add(UnlockEvent.gateOpened(target, reason)),
      child: BlocListener<UnlockBloc, UnlockState>(
        listenWhen: (before, after) => before.status != after.status,
        listener: (context, state) {
          if (state.status == UnlockStatus.invalidated) {
            context.read<LockStatusBloc>().add(
              const LockStatusEvent.refreshed(),
            );
          }
          final ask = switch (state.status) {
            UnlockStatus.unavailable ||
            UnlockStatus.cancelled ||
            UnlockStatus.invalidated ||
            UnlockStatus.failed => true,
            _ => false,
          };
          if (ask && !asked) {
            asked = true;
            UnlockSheet.show(
              context,
              target: target,
              bloc: context.read<UnlockBloc>(),
              status: context.read<LockStatusBloc>(),
            );
          }
        },
        child: widget.child,
      ),
    );
  }
}
