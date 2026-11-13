// Kanso — capture_over_app.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_theme.dart';
import '../../../core/configs/kanso_theme_scope.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_scroll_behavior.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_icon_set_scope.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../models/repos/abs/capture_window_repository.dart';
import '../../state/capture_over_app/capture_over_app_cubit.dart';
import '../../state/capture_task/capture_task_cubit.dart';
import 'capture_over_app_host.dart';

/// The app of the capture sheet over another app (`captureMain`): the
/// themes, icons and language chosen in Kanso, and the sheet, with no
/// routes and nothing painted under it.
class CaptureOverApp extends StatelessWidget {
  const CaptureOverApp({
    required this.window,
    required this.appearance,
    required this.tasks,
    required this.media,
    super.key,
  });

  final CaptureWindowRepository window;

  /// Read before the app runs, so that the sheet opens in the chosen themes.
  final AppearanceBloc appearance;

  /// Makes the Task mode's cubit.
  final CaptureTaskCubit Function() tasks;

  /// Picks and saves for Note mode.
  final CaptureOverAppCubit media;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AppearanceBloc>.value(
      value: appearance,
      child: BlocBuilder<AppearanceBloc, AppearanceState>(
        builder: (context, state) {
          return WidgetsApp(
            debugShowCheckedModeBanner: false,
            color: state.light.colors.button,
            onGenerateTitle: (context) => context.l10n.appName,
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            // The one page is see-through: the app beneath shows.
            pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(
              settings: settings,
              opaque: false,
              transitionDuration: Duration.zero,
              reverseTransitionDuration: Duration.zero,
              pageBuilder: (context, _, _) => builder(context),
            ),
            builder: (context, child) {
              final brightness = switch (state.themeMode) {
                ThemeMode.light => Brightness.light,
                ThemeMode.dark => Brightness.dark,
                ThemeMode.system => MediaQuery.platformBrightnessOf(context),
              };
              final theme = KansoTheme.build(
                brightness == Brightness.dark ? state.dark : state.light,
                fonts: state.fonts,
                textSize: state.textSize,
                previewTextSize: state.previewTextSize,
                interfaceTextSize: state.interfaceTextSize,
              );
              return AnnotatedRegion<SystemUiOverlayStyle>(
                value: SystemUiOverlayStyle(
                  statusBarColor: const Color(0x00000000),
                  systemNavigationBarColor: const Color(0x00000000),
                  statusBarIconBrightness: theme.brightness == Brightness.dark
                      ? Brightness.light
                      : Brightness.dark,
                ),
                child: KansoIconSetScope(
                  iconSet: state.iconSet,
                  child: ScrollConfiguration(
                    behavior: const KansoScrollBehavior(),
                    child: KansoThemeTransition(
                      theme: theme,
                      child: DefaultTextStyle(
                        style: theme.typography.body,
                        child: DefaultSelectionStyle(
                          selectionColor: theme.colors.accent.withValues(
                            alpha: 0.3,
                          ),
                          cursorColor: theme.colors.accent,
                          child: KansoFeedbackHost(
                            child: child ?? const SizedBox.shrink(),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              );
            },
            home: CaptureOverAppHost(
              window: window,
              tasks: tasks,
              media: media,
            ),
          );
        },
      ),
    );
  }
}
