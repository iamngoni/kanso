// Kanso — lock_item_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../injection.dart';
import '../../../note/views/widgets/attachment_sheet.dart' show byteSize;
import '../../models/data/lock_form_status.dart';
import '../../models/data/lock_mode.dart';
import '../../models/data/lock_target.dart';
import '../../state/lock_setup/lock_setup_bloc.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../utils/seal_progress_rules.dart';
import 'lock_form_sheet.dart';
import 'lock_progress_sheet.dart';

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
        } else if (state.status == LockFormStatus.stopped) {
          // Cancel stopped it: say what is left, then let the sheet go.
          final messenger = ScaffoldMessenger.maybeOf(context);
          final words = note
              ? l10n.lockStoppedMessage
              : l10n.lockStoppedNotebookMessage;
          Navigator.of(context).pop();
          messenger?.showSnackBar(SnackBar(content: Text(words)));
        }
      },
      builder: (context, state) {
        final working = state.status == LockFormStatus.working;
        final summary = state.summary;
        final progress = state.progress;
        // A long lock counts its files on a sheet of its own.
        if (working &&
            ((summary != null &&
                    wantsProgressSheet(
                      files: summary.files,
                      bytes: summary.bytes,
                    )) ||
                (progress != null && progressWantsSheet(progress)))) {
          return LockProgressSheet(
            title: note
                ? l10n.lockProgressTitleNote
                : l10n.lockProgressTitleNotebook,
            files: summary?.files ?? 0,
            bytes: summary?.bytes ?? 0,
            progress: progress,
            cancelling: state.cancelling,
            onCancel: () => context.read<LockSetupBloc>().add(
              const LockSetupEvent.cancelled(),
            ),
          );
        }
        // What the dialog says of the pictures and files it seals.
        final files = <String>[
          if (summary != null && summary.files > 0)
            l10n.lockFilesSealed(summary.files, byteSize(summary.bytes)),
          if (summary != null && summary.shared > 0)
            l10n.lockFilesShared(summary.shared),
        ].join(' ');
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
          message: _joined(<String?>[
            switch (state.mode) {
              LockMode.setPassword => l10n.lockSetMessage,
              LockMode.askPassword => null,
              LockMode.confirm =>
                note
                    ? l10n.lockNoteConfirmMessage
                    : l10n.lockNotebookConfirmMessage,
            },
            files,
          ]),
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

/// The non-empty [parts] as one paragraph, or null when there are none.
String? _joined(List<String?> parts) {
  final text = parts.whereType<String>().where((part) => part.isNotEmpty);
  return text.isEmpty ? null : text.join(' ');
}
