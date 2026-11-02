// Kanso — presentation_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page_route.dart';
import '../../../injection.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../state/preview/preview_bloc.dart';
import '../widgets/presentation_deck.dart';
import '../widgets/preview/preview_style.dart';

/// A note presented: its slides full screen, drawn from the same blocks as
/// its preview, with the system's bars out of the way. It stays with the note
/// as it changes, and closes when the note can't be read any more.
class PresentationPage extends StatefulWidget {
  const PresentationPage({required this.noteId, super.key});

  final String noteId;

  /// Presents the note [noteId] over everything else.
  static Future<void> show(BuildContext context, {required String noteId}) {
    return Navigator.of(context, rootNavigator: true).push<void>(
      KansoPageRoute<void>(
        fullscreenDialog: true,
        builder: (_) => PresentationPage(noteId: noteId),
      ),
    );
  }

  @override
  State<PresentationPage> createState() => _PresentationPageState();
}

class _PresentationPageState extends State<PresentationPage> {
  PreviewBloc? _preview;

  /// Notes were locked, and the note is being drawn again to see if it still
  /// can be: any failure now closes the slides rather than leave them showing.
  bool _locking = false;

  /// Whether [state] means the note can't be shown. A failed draw of a note
  /// that is on show keeps its slides, as the preview does.
  bool _unreadable(PreviewState state) {
    final error = state.error;
    return error != null &&
        (_locking || state.blocks == null || error is LockedError);
  }

  /// Takes this page away, wherever it is among the routes.
  void _remove() {
    final route = ModalRoute.of(context);
    if (route != null) {
      Navigator.of(context).removeRoute(route);
    }
  }

  @override
  void initState() {
    super.initState();
    unawaited(
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky),
    );
  }

  @override
  void dispose() {
    unawaited(
      SystemChrome.setEnabledSystemUIMode(
        SystemUiMode.manual,
        overlays: SystemUiOverlay.values,
      ),
    );
    unawaited(_preview?.close());
    super.dispose();
  }

  PreviewThemes _themes(AppearanceState appearance) {
    final dark = context.brightness == Brightness.dark;
    final selection = appearance.selection;
    return (
      ui: selection.uiFor(systemDark: dark),
      syntax: selection.syntaxFor(systemDark: dark),
      preview: selection.previewFor(systemDark: dark),
      dark: dark,
    );
  }

  /// The size the core draws at. A slide's text scaler sizes up what it draws
  /// as it does the text, so it isn't scaled here too.
  double _size(AppearanceState appearance) =>
      previewBaseTextSize * appearance.previewTextSize;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final appearance = context.watch<AppearanceBloc>().state;
    final themes = _themes(appearance);
    final size = _size(appearance);
    final preview = _preview;
    if (preview == null) {
      _preview = getIt<PreviewBloc>()
        ..add(PreviewEvent.started(widget.noteId, themes, size: size));
      return;
    }
    preview
      ..add(PreviewEvent.themeChanged(themes))
      ..add(PreviewEvent.styleChanged(size: size));
  }

  @override
  Widget build(BuildContext context) {
    final dark = context.brightness == Brightness.dark;
    final appearance = context.watch<AppearanceBloc>().state;
    final palette = (dark ? appearance.dark : appearance.light).preview;
    void close() => Navigator.of(context).pop();
    return ColoredBox(
      color: previewColor(palette.background),
      child: BlocProvider<PreviewBloc>.value(
        value: _preview!,
        child: MultiBlocListener(
          listeners: [
            // Notes locked: what the core shows now replaces the slides.
            BlocListener<LockStatusBloc, LockStatusState>(
              listenWhen: (before, now) => before.unlocked && !now.unlocked,
              listener: (_, _) {
                _locking = true;
                _preview?.add(const PreviewEvent.changesReceived());
              },
            ),
            // The note can't be read any more.
            BlocListener<PreviewBloc, PreviewState>(
              listenWhen: (before, now) => before.error != now.error,
              listener: (_, state) {
                if (_unreadable(state)) {
                  _remove();
                }
              },
            ),
          ],
          child: BlocBuilder<PreviewBloc, PreviewState>(
            buildWhen: (before, now) => before.blocks != now.blocks,
            builder: (context, state) {
              final blocks = state.blocks;
              if (blocks == null) {
                return const SizedBox.expand();
              }
              return PresentationDeck(
                slides: slidesOf(blocks),
                palette: palette,
                onClose: close,
              );
            },
          ),
        ),
      ),
    );
  }
}
