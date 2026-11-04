// Kanso — feedback_host.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../../injection.dart';
import '../../models/repos/abs/feedback_repository.dart';

/// [child], with the feedback outbox looked after: at launch the server is
/// asked how the reports stand (at most every 6 hours) and what is waiting
/// is sent, and whenever the app comes back to the front what is waiting is
/// sent again. Nothing polls in the background.
class FeedbackHost extends StatefulWidget {
  const FeedbackHost({required this.child, this.repository, super.key});

  final Widget child;

  /// The reports' repository; the app's own when null (tests pass theirs).
  final FeedbackRepository? repository;

  @override
  State<FeedbackHost> createState() => _FeedbackHostState();
}

class _FeedbackHostState extends State<FeedbackHost>
    with WidgetsBindingObserver {
  FeedbackRepository? get _feedback =>
      widget.repository ??
      (getIt.isRegistered<FeedbackRepository>()
          ? getIt<FeedbackRepository>()
          : null);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(_launch()));
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  Future<void> _launch() async {
    final feedback = _feedback;
    if (feedback == null) {
      return;
    }
    if (feedback.refreshDue) {
      await feedback.entries(refresh: true);
    } else {
      await feedback.flush();
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(_feedback?.flush());
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
