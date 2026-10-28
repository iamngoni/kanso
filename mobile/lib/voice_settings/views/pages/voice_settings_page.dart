// Kanso — voice_settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/voice_settings/voice_settings_bloc.dart';
import '../widgets/voice_settings_pane.dart';

/// Settings › Voice, at `/settings/voice`.
class VoiceSettingsPage extends StatelessWidget {
  const VoiceSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<VoiceSettingsBloc>(
      create: (context) =>
          getIt<VoiceSettingsBloc>()..add(const VoiceSettingsEvent.started()),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const VoiceSettingsPane(),
      ),
    );
  }
}
