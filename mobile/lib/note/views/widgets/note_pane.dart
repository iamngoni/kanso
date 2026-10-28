// Kanso — note_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart' hide LockState;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../journal/views/widgets/journal_opener.dart';
import '../../../journal/views/widgets/note_journal_actions.dart';
import '../../../lock/views/widgets/privacy_cover_sync.dart';
import '../../../sharing/state/comments/comments_bloc.dart';
import '../../../sharing/state/shared_note_status/shared_note_status_bloc.dart';
import '../../../sharing/views/widgets/comments_button.dart';
import '../../../sharing/views/widgets/share_note_button.dart';
import '../../../sharing/views/widgets/shared_note_status_strip.dart';
import '../../../shell/state/focus_mode/focus_mode_bloc.dart';
import '../../../window/views/widgets/note_window_scope.dart';
import '../../../window/views/widgets/window_actions.dart';
import '../../models/data/note_mode.dart';
import '../../models/data/note_screen_status.dart';
import '../../state/note/note_bloc.dart';
import '../../state/note_discovery/note_discovery_bloc.dart';
import 'note_bottom_bar.dart';
import 'note_info_sheet.dart';
import 'note_lock_menu_button.dart';
import 'note_lock_sync.dart';
import 'note_menu_sheet.dart';
import 'note_mode_shortcuts.dart';
import 'note_mode_toggle.dart';
import 'note_stage.dart';
import 'note_top_bar.dart';

/// A note: a bar, its title, facts and text, or why there is none to show.
/// It reads the `NoteBloc` above it.
///
/// On a phone ([inPane] false) the note has a page of its own, with a way
/// back to the list and a bar of back, forward, search and settings under
/// it. On a tablet it is a pane beside the list, and the bar above it has
/// back and forward.
class NotePane extends StatelessWidget {
  const NotePane({required this.inPane, super.key});

  final bool inPane;

  @override
  Widget build(BuildContext context) {
    final focus = _maybeReadBloc<FocusModeBloc>(context);
    if (focus == null) {
      return _build(context, focusActive: false);
    }
    return BlocBuilder<FocusModeBloc, FocusModeState>(
      buildWhen: (before, after) => before.active != after.active,
      builder: (context, state) => _build(context, focusActive: state.active),
    );
  }

  Widget _build(BuildContext context, {required bool focusActive}) {
    return NoteLockSync(
      // The cover is on exactly while an open locked note is shown.
      child: BlocSelector<NoteBloc, NoteState, bool>(
        selector: (state) =>
            state.status == NoteScreenStatus.ready &&
            state.note?.lock is LockState_Open,
        builder: (context, secure) => PrivacyCoverSync(
          secure: secure,
          child: content(context, focusActive: focusActive),
        ),
      ),
    );
  }

  Widget content(BuildContext context, {required bool focusActive}) {
    final l10n = context.l10n;
    final gutter = inPane ? KansoSpacing.lg : KansoSpacing.md;
    final history = context.read<NavigationHistoryBloc>();
    // The note's comments, where the page has them.
    CommentsBloc? comments;
    try {
      comments = context.read<CommentsBloc>();
    } on ProviderNotFoundException {
      comments = null;
    }
    // What the page says of the note's sharing, where it has that.
    var hasStatus = true;
    try {
      context.read<SharedNoteStatusBloc>();
    } on ProviderNotFoundException {
      hasStatus = false;
    }
    // A note in a window of its own, on a device that shows more than one.
    final windows = NoteWindowScope.maybeOf(context);
    return JournalOpener(
      child: BlocBuilder<NoteBloc, NoteState>(
        buildWhen: (before, now) =>
            before.mode != now.mode ||
            before.status != now.status ||
            before.note != now.note,
        builder: (context, shown) {
          final note = shown.note;
          final readable =
              shown.status == NoteScreenStatus.ready && note != null;
          final editing = readable && shown.mode == NoteMode.edit;
          // The keyboard has the foot of the screen: the bar of the page
          // gives way to it and the editor's own bar.
          final keyboardUp = MediaQuery.viewInsetsOf(context).bottom > 0;
          // The window intents are found from the focus, which
          // `NoteModeShortcuts` may hold: their actions are above it.
          final body =
              BlocBuilder<NavigationHistoryBloc, NavigationHistoryState>(
                builder: (context, steps) {
                  void goBack() => history.add(
                    const NavigationHistoryEvent.backRequested(),
                  );
                  void goForward() => history.add(
                    const NavigationHistoryEvent.forwardRequested(),
                  );
                  return Column(
                    children: <Widget>[
                      if (!focusActive)
                        NoteTopBar(
                          phone: !inPane,
                          onBack: () => leave(context),
                          onGoBack: steps.canGoBack ? goBack : null,
                          onGoForward: steps.canGoForward ? goForward : null,
                          trailing: <Widget>[
                            if (readable) const NoteJournalActions(),
                            if (readable) ...<Widget>[
                              NoteModeToggle(
                                mode: shown.mode,
                                onChanged: (mode) =>
                                    context.read<NoteBloc>().add(
                                      NoteEvent.modeChanged(mode),
                                    ),
                              ),
                              KansoIconButton(
                                icon: KansoIcons.info,
                                semanticLabel: l10n.noteInfo,
                                onPressed: () => showKansoBottomSheet<void>(
                                  context,
                                  builder: (_) =>
                                      BlocProvider<AiStatusBloc>.value(
                                        value: context.read<AiStatusBloc>(),
                                        child:
                                            BlocProvider<
                                              NoteDiscoveryBloc
                                            >.value(
                                              value: context
                                                  .read<NoteDiscoveryBloc>(),
                                              child: NoteInfoSheet(
                                                note: note,
                                                index: context
                                                    .read<LibraryIndexBloc>()
                                                    .state,
                                              ),
                                            ),
                                      ),
                                ),
                              ),
                            ],
                            if (comments != null && readable)
                              CommentsButton(
                                // Select in the editor: open it first.
                                onSelect: (commentId) {
                                  if (!editing) {
                                    context.read<NoteBloc>().add(
                                      const NoteEvent.modeChanged(
                                        NoteMode.edit,
                                      ),
                                    );
                                  }
                                  comments?.add(
                                    CommentsEvent.selected(commentId),
                                  );
                                },
                              ),
                            const ShareNoteButton(),
                            const NoteLockMenuButton(),
                            // Only a device that shows more than one window
                            // has anything to put in this menu.
                            if (windows != null && windows.supported)
                              KansoIconButton(
                                icon: KansoIcons.ellipsis,
                                semanticLabel: l10n.noteMenu,
                                onPressed: () => showKansoBottomSheet<void>(
                                  context,
                                  builder: (_) => NoteMenuSheet(
                                    noteId: shown.noteId,
                                    windows: windows,
                                  ),
                                ),
                              ),
                          ],
                        ),
                      if (hasStatus && readable && !focusActive)
                        SharedNoteStatusStrip(gutter: gutter),
                      Expanded(
                        child: NoteStage(shown: shown, gutter: gutter),
                      ),
                      if (!inPane && !focusActive && !(editing && keyboardUp))
                        NoteBottomBar(
                          onGoBack: steps.canGoBack ? goBack : null,
                          onGoForward: steps.canGoForward ? goForward : null,
                          onSearch: () => context.go(
                            const BrowseLocation.search().location,
                          ),
                          onSettings: () => context.push(KansoRoutes.settings),
                        ),
                    ],
                  );
                },
              );
          return WindowActions(
            noteId: shown.noteId,
            child: NoteModeShortcuts(
              child: ColoredBox(
                color: context.colors.editor,
                child: body,
              ),
            ),
          );
        },
      ),
    );
  }

  /// Back to the list the note was opened from.
  void leave(BuildContext context) {
    if (context.canPop()) {
      context.pop();
    } else {
      context.go(KansoRoutes.notes);
    }
  }
}

T? _maybeReadBloc<T>(BuildContext context) {
  try {
    return context.read<T>();
  } on ProviderNotFoundException {
    return null;
  }
}
