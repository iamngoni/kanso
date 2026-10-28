// Kanso — change_password_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../injection.dart';
import '../../models/data/lock_form_status.dart';
import '../../state/change_password/change_password_bloc.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import 'lock_form_sheet.dart';

/// Changes the lock password: the current one, and the new one typed twice.
/// When some locked notes didn't open with the current password and kept it,
/// the sheet says how many before it is closed.
class ChangePasswordSheet extends StatefulWidget {
  const ChangePasswordSheet({super.key});

  /// Shows the sheet over the whole app.
  static Future<void> show(
    BuildContext context, {
    required LockStatusBloc status,
  }) async {
    final bloc = getIt<ChangePasswordBloc>();
    try {
      await showKansoBottomSheet<void>(
        context,
        builder: (_) => MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<ChangePasswordBloc>.value(value: bloc),
            BlocProvider<LockStatusBloc>.value(value: status),
          ],
          child: const ChangePasswordSheet(),
        ),
      );
    } finally {
      await bloc.close();
    }
  }

  @override
  State<ChangePasswordSheet> createState() => _ChangePasswordSheetState();
}

class _ChangePasswordSheetState extends State<ChangePasswordSheet> {
  final TextEditingController current = TextEditingController();
  final TextEditingController next = TextEditingController();
  final TextEditingController again = TextEditingController();

  @override
  void dispose() {
    current.dispose();
    next.dispose();
    again.dispose();
    super.dispose();
  }

  void submit(BuildContext context) {
    context.read<ChangePasswordBloc>().add(
      ChangePasswordEvent.submitted(current.text, next.text, again.text),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<ChangePasswordBloc, ChangePasswordState>(
      listener: (context, state) {
        if (state.status == LockFormStatus.done) {
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed());
          if (state.kept == 0) {
            Navigator.of(context).pop();
          }
        }
      },
      builder: (context, state) {
        final working = state.status == LockFormStatus.working;
        final done = state.status == LockFormStatus.done;
        return LockFormSheet(
          title: l10n.lockChangeTitle,
          message: l10n.lockChangeMessage,
          fields: done
              ? const <Widget>[]
              : <Widget>[
                  KansoTextField(
                    label: l10n.lockFieldCurrent,
                    controller: current,
                    obscure: true,
                    autofocus: true,
                    enabled: !working,
                    textInputAction: TextInputAction.next,
                    error: state.status == LockFormStatus.wrongPassword
                        ? l10n.lockWrongPassword
                        : null,
                  ),
                  KansoTextField(
                    label: l10n.lockFieldNew,
                    help: l10n.lockFieldPasswordHint,
                    controller: next,
                    obscure: true,
                    enabled: !working,
                    textInputAction: TextInputAction.next,
                    error: state.status == LockFormStatus.tooShort
                        ? l10n.lockTooShort
                        : null,
                  ),
                  KansoTextField(
                    label: l10n.lockFieldAgain,
                    controller: again,
                    obscure: true,
                    enabled: !working,
                    error: state.status == LockFormStatus.mismatch
                        ? l10n.lockNewMismatch
                        : null,
                    onSubmitted: (_) => submit(context),
                  ),
                ],
          notice: switch (state.status) {
            LockFormStatus.done => l10n.lockChangeKept(state.kept),
            LockFormStatus.failed => l10n.lockActionFailed,
            _ => null,
          },
          noticeIsError: state.status != LockFormStatus.done,
          primaryLabel: done ? l10n.lockDone : l10n.lockChangeButton,
          working: working,
          onPrimary: done
              ? () => Navigator.of(context).pop()
              : () => submit(context),
        );
      },
    );
  }
}
