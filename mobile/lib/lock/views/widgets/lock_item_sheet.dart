// Kanso — lock_item_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../injection.dart';
import '../../models/data/lock_form_status.dart';
import '../../models/data/lock_mode.dart';
import '../../models/data/lock_target.dart';
import '../../state/lock_setup/lock_setup_bloc.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import 'lock_form_sheet.dart';

/// Locks a note or a notebook the desktop's three ways: "Set a lock
/// password" the first time (typed twice), the lock password when the notes
/// are closed, and a plain confirmation when they are open.
class LockItemSheet extends StatefulWidget {
  const LockItemSheet({required this.target, super.key});

  final LockTarget target;

  /// Shows the sheet for [target] over the whole app.
  static Future<void> show(
    BuildContext context,
    LockTarget target, {
    required LockStatusBloc status,
  }) async {
    final bloc = getIt<LockSetupBloc>()..add(LockSetupEvent.started(target));
    try {
      await showKansoBottomSheet<void>(
        context,
        builder: (_) => MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<LockSetupBloc>.value(value: bloc),
            BlocProvider<LockStatusBloc>.value(value: status),
          ],
          child: LockItemSheet(target: target),
        ),
      );
    } finally {
      await bloc.close();
    }
  }

  @override
  State<LockItemSheet> createState() => _LockItemSheetState();
}

class _LockItemSheetState extends State<LockItemSheet> {
  final TextEditingController password = TextEditingController();
  final TextEditingController again = TextEditingController();

  @override
  void dispose() {
    password.dispose();
    again.dispose();
    super.dispose();
  }

  void submit(BuildContext context, LockSetupState state) {
    final bloc = context.read<LockSetupBloc>();
    if (state.mode == LockMode.confirm) {
      bloc.add(const LockSetupEvent.confirmed());
    } else {
      bloc.add(LockSetupEvent.submitted(password.text, again.text));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final note = widget.target is NoteLockTarget;
    return BlocConsumer<LockSetupBloc, LockSetupState>(
      listener: (context, state) {
        if (state.status == LockFormStatus.done) {
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed());
          Navigator.of(context).pop();
        }
      },
      builder: (context, state) {
        final working = state.status == LockFormStatus.working;
        final ready = state.status != LockFormStatus.loading;
        final setting = state.mode == LockMode.setPassword;
        return LockFormSheet(
          title: switch (state.mode) {
            LockMode.setPassword => l10n.lockSetTitle,
            LockMode.askPassword =>
              note ? l10n.lockNoteAskTitle : l10n.lockNotebookAskTitle,
            LockMode.confirm =>
              note ? l10n.lockNoteConfirmTitle : l10n.lockNotebookConfirmTitle,
          },
          message: switch (state.mode) {
            LockMode.setPassword => l10n.lockSetMessage,
            LockMode.askPassword => null,
            LockMode.confirm =>
              note
                  ? l10n.lockNoteConfirmMessage
                  : l10n.lockNotebookConfirmMessage,
          },
          fields: <Widget>[
            if (state.mode != LockMode.confirm)
              KansoTextField(
                label: l10n.lockFieldPassword,
                help: setting ? l10n.lockFieldPasswordHint : null,
                controller: password,
                obscure: true,
                autofocus: true,
                enabled: !working,
                textInputAction: setting
                    ? TextInputAction.next
                    : TextInputAction.done,
                error: switch (state.status) {
                  LockFormStatus.wrongPassword => l10n.lockWrongPassword,
                  LockFormStatus.tooShort => l10n.lockTooShort,
                  _ => null,
                },
                onSubmitted: setting ? null : (_) => submit(context, state),
              ),
            if (setting)
              KansoTextField(
                label: l10n.lockFieldAgain,
                controller: again,
                obscure: true,
                enabled: !working,
                error: state.status == LockFormStatus.mismatch
                    ? l10n.lockMismatch
                    : null,
                onSubmitted: (_) => submit(context, state),
              ),
          ],
          notice: state.status == LockFormStatus.failed
              ? l10n.lockActionFailed
              : null,
          primaryLabel: l10n.lockSetButton,
          working: working,
          onPrimary: ready ? () => submit(context, state) : null,
        );
      },
    );
  }
}
