// Kanso — reminders_host.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/router/kanso_routes.dart';
import '../../utils/reminders_coordinator.dart';

/// Runs the [RemindersCoordinator] for the life of the app: it starts it,
/// has it plan again and take the answers waiting whenever the app comes
/// back to the foreground, and opens the task a tapped reminder is for.
class RemindersHost extends StatefulWidget {
  const RemindersHost({
    required this.coordinator,
    required this.router,
    required this.child,
    super.key,
  });

  final RemindersCoordinator coordinator;

  /// The router that opens the note of a reminder that was tapped.
  final GoRouter router;
  final Widget child;

  @override
  State<RemindersHost> createState() => _RemindersHostState();
}

class _RemindersHostState extends State<RemindersHost>
    with WidgetsBindingObserver {
  StreamSubscription<TaskPlace>? _opened;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _opened = widget.coordinator.opened.listen(_open);
    unawaited(widget.coordinator.start());
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    unawaited(_opened?.cancel());
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(widget.coordinator.resumed());
    }
  }

  /// Shows the note of the task; the editor puts the cursor at its line.
  void _open(TaskPlace place) {
    final here = BrowseLocation.parse(
      widget.router.routerDelegate.currentConfiguration.uri,
    );
    final target = (here ?? const BrowseLocation.place(ListPlace.all()))
        .withNote(place.noteId);
    final uri = Uri.parse(target.location);
    widget.router.go(
      uri
          .replace(
            queryParameters: <String, String>{
              ...uri.queryParameters,
              KansoRoutes.taskLineParameter: place.line.toString(),
            },
          )
          .toString(),
    );
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
