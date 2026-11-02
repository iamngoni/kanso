// Kanso — review_prompt_host.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../bridge/api/types.dart';
import '../../services/review_prompt.dart';
import '../../state/sync_status/sync_status_bloc.dart';
import '../../utils/review_eligibility.dart';

/// How long the person must stay on a list, with the keyboard down, after a
/// good moment before the app asks: long enough for a page to finish
/// sliding back, and to see they are not about to write again.
const Duration reviewSettle = Duration(milliseconds: 1500);

/// Whether [state] says sync or signing in went wrong: a failed sync, one
/// the person asked for that failed, an account whose session ended, or a
/// problem with the account or its keys. Being offline, paused or waiting
/// to try again is not trouble.
bool isReviewSyncTrouble(SyncStatusState state) {
  if (state.error != null) {
    return true;
  }
  final status = state.status;
  if (status == null) {
    return false;
  }
  if (status.phase == SyncPhase.failed ||
      status.account is SyncAccount_SessionEnded) {
    return true;
  }
  return switch (status.problem) {
    SyncProblemKind.signedOut ||
    SyncProblemKind.accountDisabled ||
    SyncProblemKind.keysDontMatch ||
    SyncProblemKind.other => true,
    _ => false,
  };
}

/// [child], with the review prompt told when the app is opened, when sync
/// completes or goes wrong, and asked to ask once a good moment is pending
/// and the person is back on a list with the keyboard down, in front.
///
/// Mounted for production release builds only, like the metrics. It needs
/// the app's `SyncStatusBloc` above it.
class ReviewPromptHost extends StatefulWidget {
  const ReviewPromptHost({
    required this.prompt,
    required this.router,
    required this.child,
    this.settle = reviewSettle,
    super.key,
  });

  final ReviewPrompt prompt;

  /// The app's routes, to know which screen is showing.
  final GoRouter router;

  final Widget child;

  /// How long to wait on a list before asking.
  final Duration settle;

  @override
  State<ReviewPromptHost> createState() => _ReviewPromptHostState();
}

class _ReviewPromptHostState extends State<ReviewPromptHost>
    with WidgetsBindingObserver {
  Timer? _timer;
  bool _inFront = true;
  double _keyboard = 0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    widget.router.routerDelegate.addListener(_schedule);
    widget.prompt.moments.addListener(_schedule);
    unawaited(widget.prompt.opened());
    // The status may have been read before this was mounted, which the
    // listener below would never see.
    _onSync(context.read<SyncStatusBloc>().state);
  }

  void _onSync(SyncStatusState state) {
    if (isReviewSyncTrouble(state)) {
      unawaited(widget.prompt.trouble());
    } else if (state.status?.phase == SyncPhase.synced) {
      unawaited(widget.prompt.synced());
    }
  }

  @override
  void didUpdateWidget(ReviewPromptHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.router != widget.router) {
      oldWidget.router.routerDelegate.removeListener(_schedule);
      widget.router.routerDelegate.addListener(_schedule);
    }
    if (oldWidget.prompt != widget.prompt) {
      oldWidget.prompt.moments.removeListener(_schedule);
      widget.prompt.moments.addListener(_schedule);
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final keyboard = MediaQuery.viewInsetsOf(context).bottom;
    if (keyboard != _keyboard) {
      _keyboard = keyboard;
      _schedule();
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    final inFront = state == AppLifecycleState.resumed;
    if (inFront && !_inFront) {
      // A new day may have started while the app was away.
      unawaited(widget.prompt.opened());
    }
    _inFront = inFront;
    _schedule();
  }

  @override
  void dispose() {
    _timer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    widget.router.routerDelegate.removeListener(_schedule);
    widget.prompt.moments.removeListener(_schedule);
    super.dispose();
  }

  /// Whether the person is on a list screen, with the keyboard down, in
  /// front, whatever is laid over it.
  bool get _onListScreen {
    if (!_inFront || _keyboard > 0) {
      return false;
    }
    final path = widget.router.routerDelegate.currentConfiguration.uri.path;
    return isReviewListLocation(path);
  }

  /// Whether a sheet, dialog or other route is on top of the screen.
  bool get _covered {
    final navigator = widget.router.routerDelegate.navigatorKey.currentState;
    return navigator != null && navigator.canPop();
  }

  /// Whether the person is on a list, uncovered, with the keyboard down,
  /// in front.
  bool get _onList => _onListScreen && !_covered;

  /// Waits [ReviewPromptHost.settle] from now, then asks if the person is
  /// still where it may. Anything changing starts the wait again, and so
  /// does finding a sheet or dialog on top (closing one tells nobody).
  void _schedule() {
    _timer?.cancel();
    _timer = null;
    if (!widget.prompt.hasPendingMoment || !_onListScreen) {
      return;
    }
    _timer = Timer(widget.settle, () {
      _timer = null;
      if (!mounted) {
        return;
      }
      if (_onList) {
        unawaited(widget.prompt.askIfDue());
      } else {
        _schedule();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<SyncStatusBloc, SyncStatusState>(
      listenWhen: (previous, current) =>
          (isReviewSyncTrouble(current) && !isReviewSyncTrouble(previous)) ||
          (current.status?.phase == SyncPhase.synced &&
              previous.status?.phase != SyncPhase.synced),
      listener: (context, state) => _onSync(state),
      child: widget.child,
    );
  }
}
