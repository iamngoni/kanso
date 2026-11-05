// Kanso — files_migration_host.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/files_migration/files_migration_bloc.dart';
import 'files_migration_notice.dart';
import 'files_migration_sheet.dart';

/// At the first launch after the update, asks to seal the attachments of
/// notes locked before they were sealed ("Seal now" or "Later"); after
/// "Later" it reminds quietly, once per launch, until it is done. It says how
/// a pass went in the sheet.
///
/// It makes its own `FilesMigrationBloc`; where none is registered (a test
/// that does not care) it shows [child] alone.
class FilesMigrationHost extends StatefulWidget {
  const FilesMigrationHost({required this.child, super.key});

  final Widget child;

  /// How long a notice stays.
  static const Duration noticeTime = Duration(seconds: 10);

  @override
  State<FilesMigrationHost> createState() => _FilesMigrationHostState();
}

/// What the notice says, and the action it offers.
class _Say {
  const _Say(this.message, {this.sealNow = false});

  final String message;
  final bool sealNow;
}

class _FilesMigrationHostState extends State<FilesMigrationHost> {
  FilesMigrationBloc? bloc;
  bool sheetOpen = false;
  _Say? say;
  Timer? timer;

  @override
  void initState() {
    super.initState();
    if (getIt.isRegistered<FilesMigrationBloc>()) {
      bloc = getIt<FilesMigrationBloc>()
        ..add(const FilesMigrationEvent.checked(launch: true));
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    unawaited(bloc?.close());
    super.dispose();
  }

  void openSheet(FilesMigrationBloc bloc) {
    if (sheetOpen) {
      return;
    }
    sheetOpen = true;
    unawaited(
      FilesMigrationSheet.show(
        context,
        bloc,
      ).whenComplete(() => sheetOpen = false),
    );
  }

  void tell(_Say next) {
    timer?.cancel();
    setState(() => say = next);
    timer = Timer(FilesMigrationHost.noticeTime, close);
  }

  void close() {
    timer?.cancel();
    if (mounted && say != null) {
      setState(() => say = null);
    }
  }

  @override
  Widget build(BuildContext context) {
    final bloc = this.bloc;
    if (bloc == null) {
      return widget.child;
    }
    final l10n = context.l10n;
    final shown = say;
    return BlocProvider<FilesMigrationBloc>.value(
      value: bloc,
      child: BlocListener<FilesMigrationBloc, FilesMigrationState>(
        listenWhen: (before, now) => before.stage != now.stage,
        listener: (context, state) {
          switch (state.stage) {
            case MigrationStage.ask:
              close();
              openSheet(bloc);
            case MigrationStage.remind:
              tell(
                _Say(
                  l10n.lockMigrationReminder(state.info?.notes ?? 0),
                  sealNow: true,
                ),
              );
              bloc.add(const FilesMigrationEvent.dismissed());
            case MigrationStage.checking ||
                MigrationStage.idle ||
                MigrationStage.password ||
                MigrationStage.wrongPassword ||
                MigrationStage.working ||
                MigrationStage.done ||
                MigrationStage.failed:
              break;
          }
        },
        child: Stack(
          children: <Widget>[
            Positioned.fill(child: widget.child),
            if (shown != null)
              PositionedDirectional(
                start: KansoSpacing.md,
                end: KansoSpacing.md,
                // Above the undo notice's place.
                bottom:
                    MediaQuery.paddingOf(context).bottom +
                    KansoSpacing.md +
                    KansoSpacing.xxl,
                child: Align(
                  alignment: Alignment.bottomCenter,
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(
                      maxWidth: KansoSizes.noticeWidth,
                    ),
                    child: FilesMigrationNotice(
                      message: shown.message,
                      closeLabel: l10n.lockFilesNoticeDismiss,
                      onClose: close,
                      actionLabel: shown.sealNow
                          ? l10n.lockMigrationSealNow
                          : null,
                      onAction: shown.sealNow
                          ? () {
                              close();
                              bloc.add(const FilesMigrationEvent.opened());
                            }
                          : null,
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
