// Kanso — lock_lifecycle_host.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/lock_status/lock_status_bloc.dart';

/// Tells the `LockStatusBloc` above it when the app changes lifecycle
/// state, so that the locked notes close when the app leaves the screen.
class LockLifecycleHost extends StatefulWidget {
  const LockLifecycleHost({required this.child, super.key});

  final Widget child;

  @override
  State<LockLifecycleHost> createState() => _LockLifecycleHostState();
}

class _LockLifecycleHostState extends State<LockLifecycleHost> {
  late final AppLifecycleListener listener;

  @override
  void initState() {
    super.initState();
    listener = AppLifecycleListener(
      onStateChange: (lifecycle) => context.read<LockStatusBloc>().add(
        LockStatusEvent.lifecycleChanged(lifecycle),
      ),
    );
  }

  @override
  void dispose() {
    listener.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
