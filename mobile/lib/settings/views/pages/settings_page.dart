// Kanso — settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../../lock/state/biometrics/biometrics_bloc.dart';
import '../../state/about/about_bloc.dart';
import '../widgets/settings_pane.dart';

/// Settings: where the appearance and the rest are changed.
class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AboutBloc>(
          create: (context) =>
              getIt<AboutBloc>()..add(const AboutEvent.started()),
        ),
        BlocProvider<BiometricsBloc>(
          create: (context) => getIt<BiometricsBloc>(),
        ),
      ],
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const SettingsPane(),
      ),
    );
  }
}
