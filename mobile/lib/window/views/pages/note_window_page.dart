// Kanso — note_window_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../editor/models/data/editor_destination.dart';
import '../../../editor/views/widgets/editor_destination_scope.dart';
import '../../../note/models/data/note_screen_status.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../note/views/widgets/note_info_sheet.dart';
import '../../../note/views/widgets/note_mode_shortcuts.dart';
import '../../../note/views/widgets/note_mode_toggle.dart';
import '../../../note/views/widgets/note_stage.dart';
import '../../../note/views/widgets/note_theme_sync.dart';
import '../../../shell/models/data/kanso_intent.dart';
import '../../models/data/note_window_request.dart';
import '../../models/data/note_window_status.dart';
import '../../models/repos/abs/note_window_repository.dart';
import '../../state/note_window/note_window_bloc.dart';
import '../../utils/note_window_title.dart';
import '../widgets/note_trash_banner.dart';
import '../widgets/note_window_menu_sheet.dart';
import '../widgets/window_actions.dart';

/// A note in a window of its own: a slim bar (the mode, the note's facts, a
/// menu), the trash's banner when the note is in the trash, and the note
/// itself, edited or read, in the editor of the main window's note pane.
/// There is no library, list or tabs here.
///
/// It reads the `NoteBloc`, `NoteWindowBloc` and `LibraryIndexBloc` above it.
/// What it can't do itself (a new note, a search, a link to another note) it
/// hands to the main window through [windows]. It names its window after the
/// note, and closes it when the note is deleted for good.
class NoteWindowPage extends StatefulWidget {
  const NoteWindowPage({required this.windows, super.key});

  /// The space either side of the text.
  static const double gutter = KansoSpacing.lg;

  final NoteWindowRepository windows;

  @override
  State<NoteWindowPage> createState() => _NoteWindowPageState();
}

class _NoteWindowPageState extends State<NoteWindowPage> {
  @override
  void initState() {
    super.initState();
    // The listener below sees changes only, and the note is often already
    // read when this page is first built (a test reads it before mounting):
    // name the window from the state it has at once.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        _follow(context, context.read<NoteWindowBloc>().state);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final windows = widget.windows;
    final themeId = syntaxThemeId(context);
    return NoteThemeSync(
      themeId: themeId,
      child: BlocListener<NoteWindowBloc, NoteWindowState>(
        listener: _follow,
        child: EditorDestinationScope(
          onDestination: (context, destination) {
            final request = switch (destination) {
              NoteDestination(:final noteId) => NoteWindowRequest.openNote(
                noteId,
              ),
              NewNoteDestination(:final title) =>
                NoteWindowRequest.newNoteTitled(title),
            };
            unawaited(windows.hand(request));
          },
          child: BlocBuilder<NoteWindowBloc, NoteWindowState>(
            buildWhen: (before, now) => before.noteId != now.noteId,
            builder: (context, window) {
              return WindowActions(
                noteId: window.noteId,
                child: Actions(
                  actions: <Type, Action<Intent>>{
                    NewNoteIntent: CallbackAction<NewNoteIntent>(
                      onInvoke: (_) {
                        unawaited(
                          windows.hand(const NoteWindowRequest.newNote()),
                        );
                        return null;
                      },
                    ),
                    SearchAllIntent: CallbackAction<SearchAllIntent>(
                      onInvoke: (_) {
                        unawaited(
                          windows.hand(const NoteWindowRequest.search()),
                        );
                        return null;
                      },
                    ),
                  },
                  child: NoteModeShortcuts(
                    child: Scaffold(
                      resizeToAvoidBottomInset: false,
                      backgroundColor: context.colors.editor,
                      body: _Body(windows: windows),
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }

  /// Names the window after the note, and closes it when the note is gone.
  void _follow(BuildContext context, NoteWindowState state) {
    switch (state.status) {
      case NoteWindowStatus.gone:
        unawaited(widget.windows.closeThisWindow());
      case NoteWindowStatus.ready:
        final l10n = context.l10n;
        unawaited(
          widget.windows.setTitle(
            noteWindowTitle(
              title: state.title,
              locked: state.locked,
              untitled: l10n.noteUntitled,
              lockedTitle: l10n.noteWindowLockedTitle,
            ),
          ),
        );
      case NoteWindowStatus.loading:
        break;
    }
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.windows});

  final NoteWindowRepository windows;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<NoteBloc, NoteState>(
      buildWhen: (before, now) =>
          before.mode != now.mode ||
          before.status != now.status ||
          before.note != now.note,
      builder: (context, shown) {
        final note = shown.note;
        final readable = shown.status == NoteScreenStatus.ready && note != null;
        return Column(
          children: <Widget>[
            KansoTopBar(
              bordered: false,
              trailing: <Widget>[
                if (readable) ...<Widget>[
                  NoteModeToggle(
                    mode: shown.mode,
                    onChanged: (mode) => context.read<NoteBloc>().add(
                      NoteEvent.modeChanged(mode),
                    ),
                  ),
                  KansoIconButton(
                    icon: KansoIcons.info,
                    semanticLabel: l10n.noteInfo,
                    onPressed: () => showKansoBottomSheet<void>(
                      context,
                      builder: (_) => NoteInfoSheet(
                        note: note,
                        index: context.read<LibraryIndexBloc>().state,
                      ),
                    ),
                  ),
                ],
                KansoIconButton(
                  icon: KansoIcons.ellipsis,
                  semanticLabel: l10n.noteMenu,
                  onPressed: () => showKansoBottomSheet<void>(
                    context,
                    builder: (_) => NoteWindowMenuSheet(windows: windows),
                  ),
                ),
              ],
            ),
            BlocBuilder<NoteWindowBloc, NoteWindowState>(
              buildWhen: (before, now) => before.trashed != now.trashed,
              builder: (context, window) {
                if (!window.trashed) {
                  return const SizedBox.shrink();
                }
                return NoteTrashBanner(
                  onRestore: () => context.read<NoteWindowBloc>().add(
                    const NoteWindowEvent.restoreRequested(),
                  ),
                );
              },
            ),
            Expanded(
              child: NoteStage(shown: shown, gutter: NoteWindowPage.gutter),
            ),
          ],
        );
      },
    );
  }
}
