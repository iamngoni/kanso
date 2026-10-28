// Kanso — app.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'account/views/widgets/device_approval_host.dart';
import 'ai_settings/state/ai_status/ai_status_bloc.dart';
import 'assist/utils/assist_editor_extras.dart';
import 'core/configs/kanso_theme.dart';
import 'core/models/repos/abs/ai_repository.dart';
import 'core/router/kanso_router.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/context_extensions.dart';
import 'core/utils/kanso_scroll_behavior.dart';
import 'core/views/widgets/foreground_reporter.dart';
import 'core/views/widgets/kanso_icon_set_scope.dart';
import 'core/views/widgets/layout_class_reporter.dart';
import 'crash/views/widgets/crash_notice_host.dart';
import 'dictation/views/widgets/dictation_scope.dart';
import 'gen/l10n/app_localizations.dart';
import 'import/views/widgets/opened_documents_host.dart';
import 'injection.dart';
import 'lock/views/widgets/lock_lifecycle_host.dart';
import 'reminders/utils/reminders_coordinator.dart';
import 'reminders/views/widgets/reminders_host.dart';
import 'settings/views/widgets/backup_notice_host.dart';

/// The Kanso app: the themes chosen (drawn from the theme files' tokens),
/// the icon theme, the language and the routes, with the sheet that asks to
/// let another device in and the notices about crashes and device backups
/// over them.
class App extends StatefulWidget {
  const App({required this.router, this.reminders, super.key});

  /// The routes of the app, and how many panes there is room for.
  final KansoRouter router;

  /// Runs the local reminder plan in production; omitted in app shell tests.
  final RemindersCoordinator? reminders;

  @override
  State<App> createState() => _AppState();
}

class _AppState extends State<App> {
  AssistEditorExtras? assistant;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      builder: (context, state) {
        return MaterialApp.router(
          debugShowCheckedModeBanner: false,
          onGenerateTitle: (context) => context.l10n.appName,
          theme: KansoTheme.build(
            state.light,
            fonts: state.fonts,
            textSize: state.textSize,
          ),
          darkTheme: KansoTheme.build(
            state.dark,
            fonts: state.fonts,
            textSize: state.textSize,
          ),
          themeMode: state.themeMode,
          scrollBehavior: const KansoScrollBehavior(),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          routerConfig: widget.router.config,
          builder: (context, child) {
            final status = context.read<AiStatusBloc>();
            assistant ??= AssistEditorExtras(
              ai: getIt<AiRepository>(),
              isEnabled: () => status.state.enabled,
              selectionLabel: context.l10n.assistSelectionAction,
              keyLabel: context.l10n.assistAccessoryLabel,
            );
            final content = KansoIconSetScope(
              iconSet: state.iconSet,
              child: LayoutClassReporter(
                notifier: widget.router.layoutClass,
                child: ForegroundReporter(
                  child: CrashNoticeHost(
                    child: LockLifecycleHost(
                      child: BackupNoticeHost(
                        child: OpenedDocumentsHost(
                          router: widget.router.config,
                          child: DeviceApprovalHost(child: child!),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            );
            final coordinator = widget.reminders;
            final hosted = coordinator == null
                ? content
                : RemindersHost(
                    coordinator: coordinator,
                    router: widget.router.config,
                    child: content,
                  );
            return DictationScope(others: assistant, child: hosted);
          },
        );
      },
    );
  }
}
