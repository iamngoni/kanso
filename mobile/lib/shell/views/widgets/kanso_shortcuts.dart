// Kanso — kanso_shortcuts.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/layout_class.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/neighbour_note.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../library/views/widgets/new_notebook_sheet.dart';
import '../../models/data/kanso_intent.dart';
import '../../state/library_pane/library_pane_bloc.dart';
import '../../state/note_creation/note_creation_bloc.dart';
import '../../utils/kanso_shortcut_bindings.dart';

/// The keyboard shortcuts for moving around on a tablet (see
/// [kansoShortcutBindings]) around [child].
///
/// It must be inside the library's drawer, where there is one, to open it:
/// [layout] says whether the library is a pane ([LayoutClass.triple]) or a
/// drawer. [location] is where the person is.
class KansoShortcuts extends StatelessWidget {
  const KansoShortcuts({
    required this.layout,
    required this.location,
    required this.child,
    this.platform,
    super.key,
  });

  final LayoutClass layout;
  final BrowseLocation? location;
  final Widget child;

  /// The platform whose keys apply; the device's when null.
  final TargetPlatform? platform;

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: kansoShortcutBindings(platform ?? defaultTargetPlatform),
      child: Actions(
        actions: <Type, Action<Intent>>{
          BackIntent: CallbackAction<BackIntent>(
            onInvoke: (_) {
              context.read<NavigationHistoryBloc>().add(
                const NavigationHistoryEvent.backRequested(),
              );
              return null;
            },
          ),
          ForwardIntent: CallbackAction<ForwardIntent>(
            onInvoke: (_) {
              context.read<NavigationHistoryBloc>().add(
                const NavigationHistoryEvent.forwardRequested(),
              );
              return null;
            },
          ),
          SearchAllIntent: CallbackAction<SearchAllIntent>(
            onInvoke: (_) {
              context.go(const BrowseLocation.search().location);
              return null;
            },
          ),
          SettingsIntent: CallbackAction<SettingsIntent>(
            onInvoke: (_) {
              context.push(KansoRoutes.settings);
              return null;
            },
          ),
          ToggleLibraryIntent: CallbackAction<ToggleLibraryIntent>(
            onInvoke: (_) {
              if (layout == LayoutClass.triple) {
                context.read<LibraryPaneBloc>().add(
                  const LibraryPaneEvent.toggled(),
                );
              } else {
                KansoDrawerScope.maybeOf(context)?.toggle();
              }
              return null;
            },
          ),
          NextNoteIntent: CallbackAction<NextNoteIntent>(
            onInvoke: (_) => stepNote(context, forward: true),
          ),
          PreviousNoteIntent: CallbackAction<PreviousNoteIntent>(
            onInvoke: (_) => stepNote(context, forward: false),
          ),
          NewNoteIntent: CallbackAction<NewNoteIntent>(
            onInvoke: (_) {
              final here = location;
              final pinned =
                  here is PlaceLocation && here.place is ListPlace_Pinned;
              context.read<NoteCreationBloc>().add(
                NoteCreationEvent.requested(
                  here is PlaceLocation ? here.place : const ListPlace.all(),
                  pinned: pinned,
                ),
              );
              return null;
            },
          ),
          NewNotebookIntent: CallbackAction<NewNotebookIntent>(
            onInvoke: (_) {
              unawaited(NewNotebookSheet.open(context));
              return null;
            },
          ),
          PinNoteIntent: CallbackAction<PinNoteIntent>(
            onInvoke: (_) {
              final open = location?.noteId;
              if (open != null) {
                context.read<ListNavigationBloc>().add(
                  ListNavigationEvent.pinRequested(open),
                );
              }
              return null;
            },
          ),
        },
        child: Focus(autofocus: true, child: child),
      ),
    );
  }

  /// Opens the note one place on in the list, or back.
  Object? stepNote(BuildContext context, {required bool forward}) {
    final here = location ?? const BrowseLocation.place(ListPlace.all());
    final next = neighbourNote(
      context.read<ListNavigationBloc>().state.noteIds,
      here.noteId,
      forward: forward,
    );
    if (next != null) {
      context.go(here.withNote(next).location);
    }
    return null;
  }
}
