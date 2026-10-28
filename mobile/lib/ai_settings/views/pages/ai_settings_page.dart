// Kanso — ai_settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/ai_settings/ai_settings_bloc.dart';
import '../widgets/ai_settings_pane.dart';

/// Settings › AI.
class AiSettingsPage extends StatelessWidget {
  const AiSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AiSettingsBloc>(
      create: (context) =>
          getIt<AiSettingsBloc>()..add(const AiSettingsEvent.started()),
      child: Scaffold(
        resizeToAvoidBottomInset: true,
        backgroundColor: context.colors.app,
        body: const AiSettingsPane(),
      ),
    );
  }
}
