// Kanso — recovery_key_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/recovery_key/recovery_key_bloc.dart';
import '../widgets/recovery_key_pane.dart';

/// Settings › Account › Your recovery key: the key of an account just made
/// or changed, to write down. With the [recoveryKey] the flow that made it
/// holds; with none, the one the library kept because it was never confirmed
/// saved.
class RecoveryKeyPage extends StatelessWidget {
  const RecoveryKeyPage({this.recoveryKey, super.key});

  final RecoveryKeyText? recoveryKey;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<RecoveryKeyBloc>(
      create: (context) =>
          getIt<RecoveryKeyBloc>()
            ..add(RecoveryKeyEvent.started(key: recoveryKey)),
      child: BlocListener<RecoveryKeyBloc, RecoveryKeyState>(
        listenWhen: (before, after) => !before.done && after.done,
        listener: (context, state) {
          if (context.canPop()) {
            context.pop();
          }
        },
        child: Scaffold(
          backgroundColor: context.colors.app,
          body: const RecoveryKeyPane(),
        ),
      ),
    );
  }
}
