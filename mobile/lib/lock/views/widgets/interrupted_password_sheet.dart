// Kanso — interrupted_password_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../state/interrupted_lock/interrupted_lock_bloc.dart';
import 'lock_form_sheet.dart';

/// Asks for the lock password so a lock that stopped can be finished or
/// undone when the notes are closed. It reads the `InterruptedLockBloc`
/// above it and closes itself once the work is done.
class InterruptedPasswordSheet extends StatefulWidget {
  const InterruptedPasswordSheet({super.key});

  /// Shows the sheet over the whole app, for [bloc].
  static Future<void> show(BuildContext context, InterruptedLockBloc bloc) =>
      showKansoBottomSheet<void>(
        context,
        builder: (_) => BlocProvider<InterruptedLockBloc>.value(
          value: bloc,
          child: const InterruptedPasswordSheet(),
        ),
      );

  @override
  State<InterruptedPasswordSheet> createState() =>
      _InterruptedPasswordSheetState();
}

class _InterruptedPasswordSheetState extends State<InterruptedPasswordSheet> {
  final TextEditingController password = TextEditingController();

  @override
  void dispose() {
    password.dispose();
    super.dispose();
  }

  void submit(BuildContext context, InterruptedLockState state) {
    final bloc = context.read<InterruptedLockBloc>();
    final typed = password.text;
    if (state.action == InterruptedAction.undo) {
      bloc.add(InterruptedLockEvent.undone(password: typed));
    } else {
      bloc.add(InterruptedLockEvent.finished(password: typed));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<InterruptedLockBloc, InterruptedLockState>(
      listenWhen: (before, now) => before.stage != now.stage,
      listener: (context, state) {
        if (state.stage == InterruptedStage.none ||
            state.stage == InterruptedStage.stopped ||
            state.stage == InterruptedStage.failed) {
          Navigator.of(context).pop();
        }
      },
      builder: (context, state) {
        final working = state.stage == InterruptedStage.working;
        return LockFormSheet(
          title: l10n.lockInterruptedPasswordTitle,
          message: l10n.lockInterruptedPasswordMessage,
          fields: <Widget>[
            KansoTextField(
              label: l10n.lockFieldPassword,
              controller: password,
              obscure: true,
              autofocus: true,
              enabled: !working,
              error: state.stage == InterruptedStage.wrongPassword
                  ? l10n.lockWrongPassword
                  : null,
              onSubmitted: (_) => submit(context, state),
            ),
          ],
          primaryLabel: l10n.lockInterruptedContinue,
          working: working,
          onPrimary: () => submit(context, state),
          // The bar goes back to Finish and Undo, which closes this sheet.
          onCancel: () => context.read<InterruptedLockBloc>().add(
            const InterruptedLockEvent.passwordDeclined(),
          ),
        );
      },
    );
  }
}
