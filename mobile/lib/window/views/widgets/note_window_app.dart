// Kanso — note_window_app.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../assist/utils/assist_editor_extras.dart';
import '../../../core/configs/kanso_theme.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_scroll_behavior.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../dictation/views/widgets/dictation_scope.dart';
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
            return MaterialApp(
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
              builder: (context, child) {
                final status = context.read<AiStatusBloc>();
                assistant ??= AssistEditorExtras(
                  ai: getIt<AiRepository>(),
                  isEnabled: () => status.state.enabled,
                  selectionLabel: context.l10n.assistSelectionAction,
                  keyLabel: context.l10n.assistAccessoryLabel,
                );
                return DictationScope(
                  others: assistant,
                  child: child ?? const SizedBox.shrink(),
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
