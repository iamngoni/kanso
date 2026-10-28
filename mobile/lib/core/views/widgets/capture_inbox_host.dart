// Kanso — capture_inbox_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/capture_inbox/capture_inbox_bloc.dart';

/// [child] with the app's coming back to the front watched: each time it
/// does, what other apps shared is saved. It needs a [CaptureInboxBloc]
/// above it.
class CaptureInboxHost extends StatefulWidget {
  const CaptureInboxHost({required this.child, super.key});

  final Widget child;

  @override
  State<CaptureInboxHost> createState() => CaptureInboxHostState();
}

class CaptureInboxHostState extends State<CaptureInboxHost> {
  late final AppLifecycleListener listener;

  @override
  void initState() {
    super.initState();
    listener = AppLifecycleListener(
      onResume: () => context.read<CaptureInboxBloc>().add(
        const CaptureInboxEvent.saveRequested(),
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
