// Kanso — foreground_reporter.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/account/account_bloc.dart';
import '../../state/device_approval/device_approval_bloc.dart';
import '../../state/sync_status/sync_status_bloc.dart';

/// Tells the app whether it is in front, so that what only matters in front
/// runs only then: the library syncs while the app is in front and stops
/// behind, and the devices that ask to join the account are watched for
/// only while an account is signed in and the app is in front. It draws
/// [child] unchanged.
///
/// An app that is only inactive (the control centre pulled down, the app
/// switcher) is still in front. It needs the app's `AccountBloc`,
/// `SyncStatusBloc` and `DeviceApprovalBloc`.
class ForegroundReporter extends StatefulWidget {
  const ForegroundReporter({required this.child, super.key});

  final Widget child;

  @override
  State<ForegroundReporter> createState() => ForegroundReporterState();
}

/// The state of a [ForegroundReporter]: it watches the app's lifecycle.
class ForegroundReporterState extends State<ForegroundReporter> {
  late final AppLifecycleListener listener;

  /// Whether the app is in front now.
  bool foreground = true;

  @override
  void initState() {
    super.initState();
    final started = WidgetsBinding.instance.lifecycleState;
    foreground =
        started == null ||
        started == AppLifecycleState.resumed ||
        started == AppLifecycleState.inactive;
    listener = AppLifecycleListener(onStateChange: onStateChange);
    // The blocs above are read once the first frame is built.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        report();
      }
    });
  }

  @override
  void dispose() {
    listener.dispose();
    super.dispose();
  }

  /// Takes the lifecycle's new state and reports it if it changed whether
  /// the app is in front.
  void onStateChange(AppLifecycleState state) {
    final now = switch (state) {
      AppLifecycleState.resumed => true,
      AppLifecycleState.inactive => foreground,
      AppLifecycleState.hidden ||
      AppLifecycleState.paused ||
      AppLifecycleState.detached => false,
    };
    if (now != foreground) {
      foreground = now;
      report();
    }
  }

  /// Tells the blocs how things stand.
  void report() {
    context.read<SyncStatusBloc>().add(
      SyncStatusEvent.foregroundChanged(foreground: foreground),
    );
    context.read<DeviceApprovalBloc>().add(
      DeviceApprovalEvent.conditionsChanged(
        signedIn: context.read<AccountBloc>().state.signedIn,
        foreground: foreground,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<AccountBloc, AccountBlocState>(
      listenWhen: (before, after) => before.signedIn != after.signedIn,
      listener: (context, state) => report(),
      child: widget.child,
    );
  }
}
