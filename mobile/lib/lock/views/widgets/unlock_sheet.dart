// Kanso — unlock_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../injection.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/unlock_status.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../state/unlock/unlock_bloc.dart';
import 'biometric_offer_sheet.dart';
import 'lock_form_sheet.dart';

/// Asks for the lock password to open a note ("Unlock note") or a notebook
/// ("Unlock notebook"). A wrong password says so and stays; the right one
/// closes the sheet, and the first time, a biometric is offered.
class UnlockSheet extends StatefulWidget {
  const UnlockSheet({required this.target, super.key});

  final LockTarget target;

  /// Shows the sheet for [target], working with [bloc] (which the caller
  /// closes), over the whole app. Whether the notes were opened.
  static Future<bool> show(
    BuildContext context, {
    required LockTarget target,
    required UnlockBloc bloc,
    required LockStatusBloc status,
  }) async {
    final offer = await showKansoBottomSheet<bool>(
      context,
      builder: (_) => MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<UnlockBloc>.value(value: bloc),
          BlocProvider<LockStatusBloc>.value(value: status),
        ],
        child: UnlockSheet(target: target),
      ),
    );
    if ((offer ?? false) && context.mounted) {
      await BiometricOfferSheet.show(context, status: status);
    }
    return offer != null;
  }

  /// [show] with a bloc of its own, for a sheet opened from a menu.
  static Future<bool> open(
    BuildContext context, {
    required LockTarget target,
    required LockStatusBloc status,
  }) async {
    final bloc = getIt<UnlockBloc>();
    try {
      return await show(context, target: target, bloc: bloc, status: status);
    } finally {
      await bloc.close();
    }
  }

  @override
  State<UnlockSheet> createState() => _UnlockSheetState();
}

class _UnlockSheetState extends State<UnlockSheet> {
  final TextEditingController password = TextEditingController();

  @override
  void dispose() {
    password.dispose();
    super.dispose();
  }

  void submit(BuildContext context) {
    context.read<UnlockBloc>().add(
      UnlockEvent.passwordSubmitted(widget.target, password.text),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final target = widget.target;
    return BlocConsumer<UnlockBloc, UnlockState>(
      listener: (context, state) {
        if (state.status == UnlockStatus.opened) {
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed());
          Navigator.of(context).pop(state.offerBiometrics);
        }
      },
      builder: (context, state) {
        final working = state.status == UnlockStatus.working;
        final error = switch (state.status) {
          UnlockStatus.wrongPassword => l10n.lockWrongPassword,
          UnlockStatus.failed => l10n.lockActionFailed,
          _ => null,
        };
        return LockFormSheet(
          title: switch (target) {
            NoteLockTarget() => l10n.lockUnlockNoteTitle,
            NotebookLockTarget() => l10n.lockUnlockNotebookTitle,
          },
          message: switch (target) {
            NoteLockTarget() => null,
            NotebookLockTarget(:final name) => l10n.lockUnlockNotebookMessage(
              name,
            ),
          },
          fields: <Widget>[
            KansoTextField(
              label: l10n.lockFieldPassword,
              controller: password,
              obscure: true,
              autofocus: true,
              enabled: !working,
              error: error,
              onSubmitted: (_) => submit(context),
            ),
          ],
          primaryLabel: l10n.lockUnlockButton,
          working: working,
          onPrimary: () => submit(context),
        );
      },
    );
  }
}
