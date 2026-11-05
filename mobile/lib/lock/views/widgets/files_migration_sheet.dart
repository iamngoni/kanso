// Kanso — files_migration_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../state/files_migration/files_migration_bloc.dart';
import '../../utils/files_migration_words.dart';
import 'lock_form_sheet.dart';
import 'lock_progress_sheet.dart';

/// The sheet that seals the attachments of notes locked before attachments
/// were sealed: the prompt ("Seal now" or "Later"), the lock password when
/// the notes are closed, then the progress with a Cancel that stops it
/// between two files.
///
/// It reads the `FilesMigrationBloc` above it and closes itself when the
/// bloc leaves the stages it shows (the host says what happened).
class FilesMigrationSheet extends StatefulWidget {
  const FilesMigrationSheet({super.key});

  /// Shows the sheet over the whole app, for [bloc]. A swipe down is
  /// "Later".
  static Future<void> show(
    BuildContext context,
    FilesMigrationBloc bloc,
  ) async {
    await showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<FilesMigrationBloc>.value(
        value: bloc,
        child: const FilesMigrationSheet(),
      ),
    );
    if (bloc.state.stage == MigrationStage.ask ||
        bloc.state.stage == MigrationStage.password ||
        bloc.state.stage == MigrationStage.wrongPassword ||
        bloc.state.stage == MigrationStage.failed) {
      bloc.add(const FilesMigrationEvent.postponed());
    }
  }

  @override
  State<FilesMigrationSheet> createState() => _FilesMigrationSheetState();
}

class _FilesMigrationSheetState extends State<FilesMigrationSheet> {
  final TextEditingController password = TextEditingController();

  @override
  void dispose() {
    password.dispose();
    super.dispose();
  }

  void submit(BuildContext context, FilesMigrationState state) {
    final bloc = context.read<FilesMigrationBloc>();
    final typed = password.text;
    bloc.add(
      state.action == MigrationAction.undo
          ? FilesMigrationEvent.undone(password: typed)
          : FilesMigrationEvent.sealed(password: typed),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<FilesMigrationBloc, FilesMigrationState>(
      listenWhen: (before, now) => before.stage != now.stage,
      listener: (context, state) {
        if (state.stage == MigrationStage.idle ||
            state.stage == MigrationStage.remind) {
          Navigator.of(context).pop();
        }
      },
      builder: (context, state) {
        final bloc = context.read<FilesMigrationBloc>();
        final info = state.info;
        switch (state.stage) {
          case MigrationStage.working:
            if (state.action == MigrationAction.undo) {
              return LockFormSheet(
                title: l10n.lockMigrationStoppedTitle,
                message: l10n.lockMigrationUndo,
                primaryLabel: l10n.lockMigrationUndo,
                onPrimary: null,
                working: true,
              );
            }
            return LockProgressSheet(
              title: l10n.lockMigrationSealing,
              files: info?.files ?? 0,
              bytes: info?.bytes ?? 0,
              progress: state.progress,
              cancelling: state.cancelling,
              onCancel: () => bloc.add(const FilesMigrationEvent.cancelled()),
            );
          case MigrationStage.password:
          case MigrationStage.wrongPassword:
            final undo = state.action == MigrationAction.undo;
            return LockFormSheet(
              title: l10n.lockMigrationTitle,
              message: l10n.lockMigrationPasswordMessage,
              fields: <Widget>[
                KansoTextField(
                  label: l10n.lockFieldPassword,
                  controller: password,
                  obscure: true,
                  autofocus: true,
                  error: state.stage == MigrationStage.wrongPassword
                      ? l10n.lockWrongPassword
                      : null,
                  onSubmitted: (_) => submit(context, state),
                ),
              ],
              primaryLabel: undo
                  ? l10n.lockMigrationUndo
                  : l10n.lockMigrationSealNow,
              onPrimary: () => submit(context, state),
              cancelLabel: l10n.lockMigrationLater,
              onCancel: () => bloc.add(const FilesMigrationEvent.postponed()),
            );
          case MigrationStage.done:
            final outcome = state.outcome;
            return LockFormSheet(
              title: l10n.lockMigrationTitle,
              message: outcome == null
                  ? l10n.lockMigrationUndone(state.undone)
                  : migrationDoneMessage(l10n, outcome),
              noticeIsError: false,
              primaryLabel: l10n.lockMigrationClose,
              onPrimary: () => bloc.add(const FilesMigrationEvent.dismissed()),
              showCancel: false,
            );
          case MigrationStage.failed:
            return LockFormSheet(
              title: l10n.lockMigrationTitle,
              notice: l10n.lockInterruptedFailed,
              primaryLabel: l10n.lockMigrationSealNow,
              onPrimary: () =>
                  bloc.add(const FilesMigrationEvent.sealRequested()),
              cancelLabel: l10n.lockMigrationLater,
              onCancel: () => bloc.add(const FilesMigrationEvent.postponed()),
            );
          case MigrationStage.checking:
          case MigrationStage.idle:
          case MigrationStage.remind:
          case MigrationStage.ask:
            if (info == null) {
              return const SizedBox.shrink();
            }
            if (migrationStopped(info)) {
              return LockFormSheet(
                title: l10n.lockMigrationStoppedTitle,
                message: l10n.lockMigrationStoppedMessage(info.stopped),
                fields: <Widget>[
                  KansoButton(
                    label: l10n.lockMigrationUndo,
                    variant: KansoButtonVariant.quiet,
                    onPressed: () =>
                        bloc.add(const FilesMigrationEvent.undoRequested()),
                  ),
                ],
                primaryLabel: l10n.lockMigrationFinish,
                onPrimary: () =>
                    bloc.add(const FilesMigrationEvent.sealRequested()),
                cancelLabel: l10n.lockMigrationLater,
                onCancel: () => bloc.add(const FilesMigrationEvent.postponed()),
              );
            }
            return LockFormSheet(
              title: l10n.lockMigrationTitle,
              message: migrationPromptMessage(l10n, info),
              primaryLabel: l10n.lockMigrationSealNow,
              onPrimary: () =>
                  bloc.add(const FilesMigrationEvent.sealRequested()),
              cancelLabel: l10n.lockMigrationLater,
              onCancel: () => bloc.add(const FilesMigrationEvent.postponed()),
            );
        }
      },
    );
  }
}
