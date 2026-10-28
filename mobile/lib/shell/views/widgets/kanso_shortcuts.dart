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
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/neighbour_note.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../library/views/widgets/new_notebook_sheet.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../note/state/note_discovery/note_discovery_bloc.dart';
import '../../../note/views/widgets/note_info_sheet.dart';
import '../../../palette/views/palette_launcher.dart';
import '../../../tabs/models/data/tab_intent.dart';
import '../../../tabs/state/tabs/tabs_bloc.dart';
import '../../models/data/kanso_intent.dart';
import '../../state/library_pane/library_pane_bloc.dart';
import '../../state/note_creation/note_creation_bloc.dart';
import '../../utils/kanso_shortcut_bindings.dart'
    show ShortcutContext, kansoShortcutBindings, shortcutBindingsFor;

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
    this.contexts = const <ShortcutContext>[
      ShortcutContext.global,
      ShortcutContext.noteList,
    ],
    super.key,
  });

  final LayoutClass layout;
  final BrowseLocation? location;
  final Widget child;

  /// The platform whose keys apply; the device's when null.
  final TargetPlatform? platform;

  /// Catalog contexts active on this surface.
  final List<ShortcutContext> contexts;

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: shortcutBindingsFor(
        contexts,
        platform ?? defaultTargetPlatform,
      ),
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
          OpenPaletteIntent: CallbackAction<OpenPaletteIntent>(
            onInvoke: (_) {
              unawaited(PaletteLauncher.open(context));
              return null;
            },
          ),
          OpenTodayIntent: CallbackAction<OpenTodayIntent>(
            onInvoke: (_) {
              context.go(KansoRoutes.daily);
              return null;
            },
          ),
          ReplaceInNotesIntent: CallbackAction<ReplaceInNotesIntent>(
            onInvoke: (_) {
              context.go(KansoRoutes.replace);
              return null;
            },
          ),
          ToggleInspectorIntent: CallbackAction<ToggleInspectorIntent>(
            onInvoke: (_) {
              final focused =
                  FocusManager.instance.primaryFocus?.context ?? context;
              final note = _maybeReadBloc<NoteBloc>(focused)?.state.note;
              final index = _maybeReadBloc<LibraryIndexBloc>(focused);
              final discovery = _maybeReadBloc<NoteDiscoveryBloc>(focused);
              if (note != null && index != null && discovery != null) {
                unawaited(
                  showKansoBottomSheet<void>(
                    focused,
                    builder: (_) => BlocProvider<NoteDiscoveryBloc>.value(
                      value: discovery,
                      child: NoteInfoSheet(note: note, index: index.state),
                    ),
                  ),
                );
              }
              return null;
            },
          ),
          CloseTabIntent: CallbackAction<CloseTabIntent>(
            onInvoke: (_) {
              final tabs = _maybeReadBloc<TabsBloc>(context);
              if (tabs != null &&
                  tabs.state.enabled &&
                  tabs.state.tabs.length > 1) {
                tabs.add(TabsEvent.closed(tabs.state.tabs.active));
              }
              return null;
            },
          ),
          NextTabIntent: CallbackAction<NextTabIntent>(
            onInvoke: (_) {
              final tabs = _maybeReadBloc<TabsBloc>(context);
              tabs?.add(const TabsEvent.nextRequested());
              return null;
            },
          ),
          PreviousTabIntent: CallbackAction<PreviousTabIntent>(
            onInvoke: (_) {
              final tabs = _maybeReadBloc<TabsBloc>(context);
              tabs?.add(const TabsEvent.previousRequested());
              return null;
            },
          ),
          ReopenClosedTabIntent: CallbackAction<ReopenClosedTabIntent>(
            onInvoke: (_) {
              final tabs = _maybeReadBloc<TabsBloc>(context);
              tabs?.add(const TabsEvent.reopenRequested());
              return null;
            },
          ),
          GoToTabIntent: CallbackAction<GoToTabIntent>(
            onInvoke: (intent) {
              final tabs = _maybeReadBloc<TabsBloc>(context);
              tabs?.add(TabsEvent.numbered(intent.number));
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

T? _maybeReadBloc<T>(BuildContext context) {
  try {
    return context.read<T>();
  } on ProviderNotFoundException {
    return null;
  }
}
