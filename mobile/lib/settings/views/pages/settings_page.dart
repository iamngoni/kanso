// Kanso — settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../feedback/state/my_feedback/my_feedback_bloc.dart';
import '../../../injection.dart';
import '../../../push/state/push_setting/push_setting_bloc.dart';
import '../../../shell/models/data/kanso_intent.dart';
import '../../../shell/utils/kanso_shortcut_bindings.dart'
    show ShortcutContext, shortcutBindingsFor;
import '../widgets/settings_pane.dart';

/// Settings: the account and sync, and a row for each category of
/// settings, which opens on a page of its own.
class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<MyFeedbackBloc>(
          create: (_) =>
              getIt<MyFeedbackBloc>()..add(const MyFeedbackEvent.started()),
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
                  KansoRoutes.settingsGeneral,
                  KansoRoutes.settingsAppearance,
                  KansoRoutes.settingsJournal,
                  KansoRoutes.settingsCalendar,
                  KansoRoutes.settingsDiscovery,
                  KansoRoutes.settingsPrivacy,
                  KansoRoutes.settingsAbout,
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
            child: KansoPage(
              resizeToAvoidBottomInset: false,
              backgroundColor: context.colors.app,
              body: const SettingsPane(),
            ),
          ),
        ),
      ),
    );
  }
}
