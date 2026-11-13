// Kanso — settings_category_pages.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../capture/state/capture_task/capture_task_cubit.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../../lock/state/biometrics/biometrics_bloc.dart';
import '../../../reminders/state/reminders_settings/reminders_settings_bloc.dart';
import '../../../reminders/views/widgets/reminders_permission_sync.dart';
import '../../../templates/state/daily_template/daily_template_bloc.dart';
import '../../state/about/about_bloc.dart';
import '../../state/word_goal_settings/word_goal_settings_bloc.dart';
import '../widgets/about_settings_pane.dart';
import '../widgets/appearance_settings_pane.dart';
import '../widgets/discovery_settings_pane.dart';
import '../widgets/general_settings_pane.dart';
import '../widgets/journal_settings_pane.dart';
import '../widgets/privacy_settings_pane.dart';

/// The pages Settings opens for each of its categories. Each makes the
/// blocs only its own sections read; the rest are the app's.
class _CategoryPage extends StatelessWidget {
  const _CategoryPage({required this.pane});

  final Widget pane;

  @override
  Widget build(BuildContext context) {
    return KansoPage(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: pane,
    );
  }
}

/// Settings › General.
class GeneralSettingsPage extends StatelessWidget {
  const GeneralSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<WordGoalSettingsBloc>(
      create: (_) =>
          getIt<WordGoalSettingsBloc>()
            ..add(const WordGoalSettingsEvent.started()),
      child: const _CategoryPage(pane: GeneralSettingsPane()),
    );
  }
}

/// Settings › Appearance.
class AppearanceSettingsPage extends StatelessWidget {
  const AppearanceSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const _CategoryPage(pane: AppearanceSettingsPane());
  }
}

/// Settings › Daily notes and reminders.
class JournalSettingsPage extends StatelessWidget {
  const JournalSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<DailyTemplateBloc>(
          create: (_) =>
              getIt<DailyTemplateBloc>()
                ..add(const DailyTemplateEvent.started()),
        ),
        BlocProvider<CaptureTaskCubit>(
          create: (_) => getIt<CaptureTaskCubit>()..started(),
        ),
        BlocProvider<RemindersSettingsBloc>(
          create: (_) =>
              getIt<RemindersSettingsBloc>()
                ..add(const RemindersSettingsEvent.started()),
        ),
      ],
      child: const RemindersPermissionSync(
        child: _CategoryPage(pane: JournalSettingsPane()),
      ),
    );
  }
}

/// Settings › Discovery.
class DiscoverySettingsPage extends StatelessWidget {
  const DiscoverySettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const _CategoryPage(pane: DiscoverySettingsPane());
  }
}

/// Settings › Privacy and security.
class PrivacySettingsPage extends StatelessWidget {
  const PrivacySettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<BiometricsBloc>(
      create: (_) => getIt<BiometricsBloc>(),
      child: const _CategoryPage(pane: PrivacySettingsPane()),
    );
  }
}

/// Settings › About.
class AboutSettingsPage extends StatelessWidget {
  const AboutSettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AboutBloc>(
      create: (_) => getIt<AboutBloc>()..add(const AboutEvent.started()),
      child: const _CategoryPage(pane: AboutSettingsPane()),
    );
  }
}
