// Kanso — note_window_app.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../assist/utils/assist_editor_extras.dart';
import '../../../canvas/models/repos/abs/canvas_repository.dart';
import '../../../canvas/utils/sketch_editor_extras.dart';
import '../../../core/configs/kanso_theme.dart';
import '../../../core/configs/kanso_theme_scope.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_scroll_behavior.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_page_route.dart';
import '../../../dictation/utils/combined_editor_extras.dart';
import '../../../dictation/views/widgets/dictation_scope.dart';
import '../../../editor_extras/models/data/editor_extras.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../../injection.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../models/repos/abs/note_window_repository.dart';
import '../../state/note_window/note_window_bloc.dart';
import '../pages/note_window_page.dart';
import 'note_window_scope.dart';

/// The app of a note's own window: the themes chosen, the language, and the
/// note, with no routes. The library index (the notebooks and tags a note's
/// facts are looked up in) and the appearance are shared by the window.
class NoteWindowApp extends StatefulWidget {
  const NoteWindowApp({
    required this.noteId,
    required this.windows,
    required this.appearance,
    super.key,
  });

  /// The note the window shows.
  final String noteId;

  final NoteWindowRepository windows;

  /// Read before the window runs, so that its first frame is already in the
  /// chosen themes.
  final AppearanceBloc appearance;

  @override
  State<NoteWindowApp> createState() => _NoteWindowAppState();
}

class _NoteWindowAppState extends State<NoteWindowApp> {
  AssistEditorExtras? assistant;
  EditorExtras? others;

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: widget.appearance),
        BlocProvider<AiStatusBloc>(
          create: (_) =>
              getIt<AiStatusBloc>()..add(const AiStatusEvent.started()),
        ),
        BlocProvider<LibraryIndexBloc>(
          create: (_) =>
              getIt<LibraryIndexBloc>()..add(const LibraryIndexEvent.started()),
        ),
      ],
      child: NoteWindowScope(
        repository: widget.windows,
        child: BlocBuilder<AppearanceBloc, AppearanceState>(
          builder: (context, state) {
            return WidgetsApp(
              debugShowCheckedModeBanner: false,
              color: state.light.colors.button,
              onGenerateTitle: (context) => context.l10n.appName,
              localizationsDelegates: AppLocalizations.localizationsDelegates,
              supportedLocales: AppLocalizations.supportedLocales,
              pageRouteBuilder: <T>(settings, builder) => KansoPageRoute<T>(
                settings: settings,
                builder: builder,
              ),
              builder: (context, child) {
                final status = context.read<AiStatusBloc>();
                assistant ??= AssistEditorExtras(
                  ai: getIt<AiRepository>(),
                  isEnabled: () => status.state.enabled,
                  selectionLabel: context.l10n.assistSelectionAction,
                  keyLabel: context.l10n.assistAccessoryLabel,
                );
                others ??= CombinedEditorExtras(
                  assistant!,
                  SketchEditorExtras(
                    repository: getIt<CanvasRepository>(),
                    label: context.l10n.sketchInsertKey,
                  ),
                );
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
                return ScrollConfiguration(
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
                          child: DictationScope(
                            others: others,
                            child: child ?? const SizedBox.shrink(),
                          ),
                        ),
                      ),
                    ),
                  ),
                );
              },
              home: _NoteWindowHost(
                noteId: widget.noteId,
                windows: widget.windows,
              ),
            );
          },
        ),
      ),
    );
  }
}

/// Makes the BLoCs of the window's note, in the theme the screen is in.
class _NoteWindowHost extends StatelessWidget {
  const _NoteWindowHost({required this.noteId, required this.windows});

  final String noteId;
  final NoteWindowRepository windows;

  @override
  Widget build(BuildContext context) {
    final themeId = syntaxThemeId(context);
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<NoteBloc>(
          create: (_) =>
              getIt<NoteBloc>()..add(NoteEvent.started(noteId, themeId)),
        ),
        BlocProvider<NoteWindowBloc>(
          create: (_) =>
              getIt<NoteWindowBloc>()..add(NoteWindowEvent.started(noteId)),
        ),
      ],
      child: NoteWindowPage(windows: windows),
    );
  }
}
