// Kanso — settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../../lock/state/biometrics/biometrics_bloc.dart';
import '../../../push/state/push_setting/push_setting_bloc.dart';
import '../../../reminders/state/reminders_settings/reminders_settings_bloc.dart';
import '../../../reminders/views/widgets/reminders_permission_sync.dart';
import '../../../shell/models/data/kanso_intent.dart';
import '../../../shell/utils/kanso_shortcut_bindings.dart'
    show ShortcutContext, shortcutBindingsFor;
import '../../../templates/state/daily_template/daily_template_bloc.dart';
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
        BlocProvider<DailyTemplateBloc>(
          create: (_) =>
              getIt<DailyTemplateBloc>()
                ..add(const DailyTemplateEvent.started()),
        ),
        BlocProvider<RemindersSettingsBloc>(
          create: (_) =>
              getIt<RemindersSettingsBloc>()
                ..add(const RemindersSettingsEvent.started()),
        ),
        BlocProvider<PushSettingBloc>(
          create: (context) =>
              getIt<PushSettingBloc>()..add(const PushSettingEvent.started()),
        ),
      ],
      child: Shortcuts(
        shortcuts: shortcutBindingsFor(
          const <ShortcutContext>[
            ShortcutContext.settings,
            ShortcutContext.settingsNav,
          ],
          defaultTargetPlatform,
        ),
        child: Actions(
          actions: <Type, Action<Intent>>{
            CloseSettingsIntent: CallbackAction<CloseSettingsIntent>(
              onInvoke: (_) {
                context.pop();
                return null;
              },
            ),
            SettingsPageStepIntent: CallbackAction<SettingsPageStepIntent>(
              onInvoke: (intent) {
                const pages = <String>[
                  KansoRoutes.settings,
                  KansoRoutes.settingsNotebook,
                  KansoRoutes.settingsTheme,
                  KansoRoutes.settingsIcons,
                  KansoRoutes.settingsFonts,
                  KansoRoutes.settingsCrashReports,
                  KansoRoutes.settingsBackups,
                  KansoRoutes.settingsLicences,
                ];
                final current = GoRouterState.of(context).uri.path;
                final at = pages.indexOf(current);
                if (at >= 0) {
                  final step = intent.forward ? 1 : -1;
                  context.go(pages[(at + step + pages.length) % pages.length]);
                }
                return null;
              },
            ),
          },
          child: Focus(
            autofocus: true,
            child: RemindersPermissionSync(
              child: Scaffold(
                resizeToAvoidBottomInset: false,
                backgroundColor: context.colors.app,
                body: const SettingsPane(),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
