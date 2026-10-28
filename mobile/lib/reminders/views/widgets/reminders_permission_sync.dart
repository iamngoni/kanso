// Kanso — reminders_permission_sync.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/reminders_settings/reminders_settings_bloc.dart';

/// Reads the system's notification permission again when the app comes
/// back to the foreground, so the reminders row follows a change the person
/// made in the system's settings.
class RemindersPermissionSync extends StatefulWidget {
  const RemindersPermissionSync({required this.child, super.key});

  final Widget child;

  @override
  State<RemindersPermissionSync> createState() =>
      _RemindersPermissionSyncState();
}

class _RemindersPermissionSyncState extends State<RemindersPermissionSync>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      context.read<RemindersSettingsBloc>().add(
        const RemindersSettingsEvent.resumed(),
      );
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
