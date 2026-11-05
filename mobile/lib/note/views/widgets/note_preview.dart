// Kanso — note_preview.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_page_route.dart';
import '../../../injection.dart';
import '../../../pdf_reader/views/pages/pdf_reader_page.dart';
import '../../../recording/models/repos/abs/player_repository.dart';
import '../../../recording/state/player/player_bloc.dart';
import '../../../recording/views/widgets/recording_player_card.dart';
import '../../models/data/preview_action.dart';
import '../../models/data/source_line.dart';
import '../../state/preview/preview_bloc.dart';
import 'attachment_sheet.dart';
import 'note_source_text.dart';
import 'pdf_highlights_sheet.dart';
import 'preview/native_preview.dart';

/// A note's text in preview: the core's blocks, drawn by the app in the
/// fitted preview palette and the note's fonts, in the note's scroll. Every
/// tap on an address in it goes to [PreviewBloc].
///
/// Until the blocks come, and where no [PreviewBloc] can be made (widget
/// tests that don't register one), it is the note's source text, as the
/// editor shows it.
class NotePreview extends StatelessWidget {
  const NotePreview({
    required this.note,
    required this.lines,
    this.onEditAt,
    super.key,
  });

  final NoteView note;

  /// The text cut into lines, for the source text.
  final List<SourceLine> lines;

  /// Asks to edit at an offset in the text; a double tap on the preview
  /// asks to edit at its start.
  final ValueChanged<int>? onEditAt;

  @override
  Widget build(BuildContext context) {
    final source = NoteSourceText(
      text: note.body ?? '',
      lines: lines,
      onEditAt: onEditAt,
    );
    if (!getIt.isRegistered<PreviewBloc>()) {
      return source;
    }
    final dark = context.brightness == Brightness.dark;
    final appearance = context.watch<AppearanceBloc>().state;
    final selection = appearance.selection;
    final PreviewThemes themes = (
      ui: selection.uiFor(systemDark: dark),
      syntax: selection.syntaxFor(systemDark: dark),
      preview: selection.previewFor(systemDark: dark),
      dark: dark,
    );
    final size = previewBaseTextSize * appearance.previewTextSize;
    return BlocProvider<PreviewBloc>(
      key: ValueKey<String>(note.id),
      create: (context) =>
          getIt<PreviewBloc>()
            ..add(PreviewEvent.started(note.id, themes, size: size)),
      child: _PreviewInputs(
        themes: themes,
        size: size,
        child: PdfNotesScope(
          noteId: note.id,
          child: _PlayerHost(
            noteId: note.id,
            body: note.body ?? '',
            builder: (player) => PreviewPage(
              palette: (dark ? appearance.dark : appearance.light).preview,
              source: source,
              onEdit: onEditAt == null ? null : () => onEditAt!(0),
              player: player,
            ),
          ),
        ),
      ),
    );
  }
}

/// Gives the note's preview its recording player (issue #38): a [PlayerBloc]
/// for this note, made when the note opens and closed when it goes, which
/// stops the sound. The note's text changing (an edit, or sync bringing the
/// audio line) has it look for the recording again. Without a player
/// repository (widget tests that don't register one) the note has no player.
class _PlayerHost extends StatefulWidget {
  const _PlayerHost({
    required this.noteId,
    required this.body,
    required this.builder,
  });

  final String noteId;
  final String body;

  /// The page, given the note's player (null where there is none).
  final Widget Function(PlayerBloc? player) builder;

  @override
  State<_PlayerHost> createState() => _PlayerHostState();
}

class _PlayerHostState extends State<_PlayerHost> {
  PlayerBloc? _bloc;

  @override
  void initState() {
    super.initState();
    if (getIt.isRegistered<PlayerRepository>()) {
      _bloc = getIt<PlayerBloc>()..add(PlayerEvent.started(widget.noteId));
    }
  }

  @override
  void didUpdateWidget(_PlayerHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.noteId != oldWidget.noteId) {
      // The same page now shows another note: its recording, not the last
      // one's (starting again closes the old player, which stops it).
      _bloc?.add(PlayerEvent.started(widget.noteId));
    } else if (widget.body != oldWidget.body) {
      _bloc?.add(const PlayerEvent.refreshed());
    }
  }

  @override
  void dispose() {
    unawaited(_bloc?.close());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return widget.builder(_bloc);
  }
}

/// Tells [PreviewBloc] when the themes in force or the text size change, so
/// the note's code, math and drawings are drawn again in them.
class _PreviewInputs extends StatefulWidget {
  const _PreviewInputs({
    required this.themes,
    required this.size,
    required this.child,
  });

  final PreviewThemes themes;
  final double size;
  final Widget child;

  @override
  State<_PreviewInputs> createState() => _PreviewInputsState();
}

class _PreviewInputsState extends State<_PreviewInputs> {
  @override
  void didUpdateWidget(_PreviewInputs oldWidget) {
    super.didUpdateWidget(oldWidget);
    final bloc = context.read<PreviewBloc>();
    if (widget.themes != oldWidget.themes) {
      bloc.add(PreviewEvent.themeChanged(widget.themes));
    }
    if (widget.size != oldWidget.size) {
      bloc.add(PreviewEvent.styleChanged(size: widget.size));
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

/// The blocks of [PreviewBloc], drawn natively, and what the screen does for
/// a tap in them. It is a sliver, built lazily; a [SelectionArea] round the
/// scroll it is in lets its text be selected across blocks.
class PreviewPage extends StatefulWidget {
  const PreviewPage({
    required this.palette,
    required this.source,
    this.onEdit,
    this.player,
    super.key,
  });

  /// The fitted preview palette the blocks are drawn in.
  final PaletteDto palette;

  /// What shows until the blocks are drawn: the note's source text.
  final Widget source;

  final VoidCallback? onEdit;

  /// The note's recording player; the card and the timestamp taps are its.
  final PlayerBloc? player;

  @override
  State<PreviewPage> createState() => _PreviewPageState();
}

class _PreviewPageState extends State<PreviewPage> {
  final GlobalKey<NativePreviewState> _preview =
      GlobalKey<NativePreviewState>();

  /// Scrolls the note so that the heading [anchor] is at the top.
  Future<void> scrollTo(String anchor) async {
    await _preview.currentState?.scrollTo(anchor);
  }

  void act(BuildContext context, PreviewAction action) {
    switch (action) {
      case PreviewOpenNote(:final noteId):
        context.go(
          BrowseLocation.place(const ListPlace.all(), noteId: noteId).location,
        );
      case PreviewOpenTag(:final name):
        final tags = context.read<LibraryIndexBloc>().state.tags;
        final tag = tags.where((tag) => tag.name == name).firstOrNull;
        if (tag != null) {
          context.go(BrowseLocation.place(ListPlace.tag(id: tag.id)).location);
        }
      case PreviewScrollTo(:final anchor):
        unawaited(scrollTo(anchor));
      case PreviewShowAttachment(:final file):
        unawaited(
          showAttachment(
            context,
            file,
            noteId: context.read<PreviewBloc>().state.noteId,
          ),
        );
      case PreviewOpenPdf(
        :final noteId,
        :final file,
        :final name,
        :final page,
        :final highlight,
      ):
        unawaited(
          Navigator.of(context, rootNavigator: true).push<void>(
            KansoPageRoute<void>(
              builder: (_) => PdfReaderPage(
                noteId: noteId,
                file: file,
                name: name,
                page: page,
                highlight: highlight,
              ),
            ),
          ),
        );
      case PreviewCopyCode(:final code):
        unawaited(Clipboard.setData(ClipboardData(text: code)));
      case PreviewPlayMoment(:final fileId, :final seconds):
        widget.player?.add(PlayerEvent.momentTapped(fileId, seconds));
    }
  }

  /// [preview] under the recording's player card, if the note has a player.
  /// The card is the first thing in the note and scrolls with it.
  Widget _withPlayer(BuildContext context, Widget preview) {
    final player = widget.player;
    if (player == null) {
      return preview;
    }
    return SliverMainAxisGroup(
      slivers: <Widget>[
        SliverToBoxAdapter(
          child: RecordingPlayerCard(
            bloc: player,
            onSyncNow: () => context.read<SyncStatusBloc>().add(
              const SyncStatusEvent.syncRequested(),
            ),
          ),
        ),
        preview,
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return MultiBlocListener(
      listeners: <BlocListener<PreviewBloc, PreviewState>>[
        BlocListener<PreviewBloc, PreviewState>(
          listenWhen: (before, now) =>
              now.action != null && before.action != now.action,
          listener: (context, state) {
            final action = state.action!;
            context.read<PreviewBloc>().add(const PreviewEvent.actionHandled());
            act(context, action);
          },
        ),
        BlocListener<PreviewBloc, PreviewState>(
          listenWhen: (before, now) =>
              now.notice != null && before.notice != now.notice,
          listener: (context, state) {
            final notice = state.notice!;
            context.read<PreviewBloc>().add(
              const PreviewEvent.noticeDismissed(),
            );
            KansoFeedback.show(
              context,
              notice is LockedError
                  ? l10n.previewLinkLocked
                  : l10n.previewLinkFailed,
              tone: KansoFeedbackTone.error,
            );
          },
        ),
      ],
      child: BlocBuilder<PreviewBloc, PreviewState>(
        buildWhen: (before, now) =>
            before.error != now.error || before.blocks != now.blocks,
        builder: (context, state) {
          final blocks = state.blocks;
          if (blocks == null) {
            if (state.error != null) {
              return SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 24),
                  child: Text(
                    l10n.previewFailed,
                    style: context.typography.secondary,
                  ),
                ),
              );
            }
            return widget.source;
          }
          final preview = NativePreview(
            key: _preview,
            blocks: blocks,
            palette: widget.palette,
            onEdit: widget.onEdit,
            onLink: (url) =>
                context.read<PreviewBloc>().add(PreviewEvent.linkTapped(url)),
          );
          return _withPlayer(context, preview);
        },
      ),
    );
  }
}
