// Kanso — push_lifecycle_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/push_registration/push_registration_bloc.dart';

/// Tells the [PushRegistrationBloc] each time the app comes back to the
/// front, so a registration that failed is tried again. It needs the bloc
/// above it.
class PushLifecycleHost extends StatefulWidget {
  const PushLifecycleHost({required this.child, super.key});

  final Widget child;

  @override
  State<PushLifecycleHost> createState() => PushLifecycleHostState();
}

/// The state of a [PushLifecycleHost]: it watches the app's lifecycle.
class PushLifecycleHostState extends State<PushLifecycleHost>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      context.read<PushRegistrationBloc>().add(
        const PushRegistrationEvent.resumed(),
      );
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
