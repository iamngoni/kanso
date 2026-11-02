// Kanso — undo_notice_timeout.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/undo_notice/undo_notice_bloc.dart';

/// Tells the `UndoNoticeBloc` that the time of the notice numbered [serial]
/// is up, a few seconds after this is built, and not if it is taken away
/// first. A person who uses a screen reader gets twice as long.
class UndoNoticeTimeout extends StatefulWidget {
  const UndoNoticeTimeout({
    required this.serial,
    required this.child,
    this.stays,
    super.key,
  });

  final int serial;

  /// How long the notice stays, when that is not [undoNoticeDuration].
  final Duration? stays;
  final Widget child;

  @override
  State<UndoNoticeTimeout> createState() => UndoNoticeTimeoutState();
}

/// The state of an [UndoNoticeTimeout]: the timer, which goes with it.
class UndoNoticeTimeoutState extends State<UndoNoticeTimeout> {
  Timer? timer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    timer?.cancel();
    final slow = MediaQuery.accessibleNavigationOf(context);
    final bloc = context.read<UndoNoticeBloc>();
    final serial = widget.serial;
    timer = Timer(
      (widget.stays ?? undoNoticeDuration) * (slow ? 2 : 1),
      () => bloc.add(UndoNoticeEvent.timedOut(serial)),
    );
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
