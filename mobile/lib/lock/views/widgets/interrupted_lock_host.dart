// Kanso — interrupted_lock_host.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/interrupted_lock/interrupted_lock_bloc.dart';
import 'interrupted_lock_bar.dart';
import 'interrupted_password_sheet.dart';

/// Looks, whenever the note [noteId] changes to [revision], whether a lock
/// stopped on it, and shows the bar that finishes or undoes it. It asks for
/// the lock password when the notes are closed, and says once that notes
/// locked before this update seal their files when opened.
class InterruptedLockHost extends StatefulWidget {
  const InterruptedLockHost({
    required this.noteId,
    required this.revision,
    required this.gutter,
    super.key,
  });

  final String noteId;
  final String revision;
  final double gutter;

  @override
  State<InterruptedLockHost> createState() => _InterruptedLockHostState();
}

class _InterruptedLockHostState extends State<InterruptedLockHost> {
  late final InterruptedLockBloc bloc = getIt<InterruptedLockBloc>()
    ..add(InterruptedLockEvent.checked(widget.noteId));

  @override
  void didUpdateWidget(InterruptedLockHost old) {
    super.didUpdateWidget(old);
    if (old.noteId != widget.noteId || old.revision != widget.revision) {
      bloc.add(InterruptedLockEvent.checked(widget.noteId));
    }
  }

  @override
  void dispose() {
    unawaited(bloc.close());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<InterruptedLockBloc>.value(
      value: bloc,
      child: MultiBlocListener(
        listeners: <BlocListener<dynamic, dynamic>>[
          BlocListener<InterruptedLockBloc, InterruptedLockState>(
            listenWhen: (before, now) =>
                before.stage != now.stage &&
                now.stage == InterruptedStage.needsPassword,
            listener: (context, state) =>
                unawaited(InterruptedPasswordSheet.show(context, bloc)),
          ),
          BlocListener<InterruptedLockBloc, InterruptedLockState>(
            listenWhen: (before, now) => !before.filesNotice && now.filesNotice,
            listener: (context, state) {
              final l10n = context.l10n;
              final messenger = ScaffoldMessenger.maybeOf(context);
              messenger?.showSnackBar(
                SnackBar(
                  duration: const Duration(days: 1),
                  showCloseIcon: true,
                  content: Text(l10n.lockFilesNotice),
                ),
              );
              bloc.add(const InterruptedLockEvent.noticeShown());
            },
          ),
        ],
        child: InterruptedLockBar(gutter: widget.gutter),
      ),
    );
  }
}
