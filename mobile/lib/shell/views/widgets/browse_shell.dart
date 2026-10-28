// Kanso — browse_shell.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/layout_class.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../tabs/views/widgets/tab_actions.dart';
import '../../../tabs/views/widgets/tabbed_note.dart';
import '../../../tabs/views/widgets/tabs_sync.dart';
import '../../state/focus_mode/focus_mode_bloc.dart';
import '../../state/library_pane/library_pane_bloc.dart';
import 'focus_mode_exit.dart';
import 'kanso_shortcuts.dart';
import 'library_drawer_host.dart';
import 'library_navigation.dart';
import 'list_note_layout.dart';
import 'list_slot.dart';
import 'navigation_history_listener.dart';
import 'note_creation_listener.dart';
import 'tablet_layout.dart';
import 'undo_notice_host.dart';

/// The frame around every place, search and note, in as many panes as the
/// window has room for.
///
/// * One pane ([LayoutClass.single]): [child], the navigator showing the
///   pages of the address, fills the window, and the library is a drawer
///   over it.
/// * Two ([LayoutClass.double]): the list of [location] beside [child],
///   which is the note; the library is a drawer.
/// * Three ([LayoutClass.triple]): the library, the list and the note.
///
/// [child] keeps its state when the window changes from one to another, so
/// the open note is still open; so does the list, so a turn of the device
/// leaves it where it was scrolled.
class BrowseShell extends StatefulWidget {
  const BrowseShell({
    required this.layout,
    required this.location,
    required this.child,
    this.designGallery = false,
    super.key,
  });

  /// How many panes there is room for now.
  final ValueListenable<LayoutClass> layout;

  /// Where the person is; null for an address that is neither a place nor
  /// a search.
  final BrowseLocation? location;

  /// Whether the design gallery can be opened from here.
  final bool designGallery;

  /// The navigator showing the pages of the current address.
  final Widget child;

  @override
  State<BrowseShell> createState() => BrowseShellState();
}

/// The state of a [BrowseShell]: it holds the key that carries the list from
/// one layout of the panes to the next.
class BrowseShellState extends State<BrowseShell> {
  final GlobalKey listKey = GlobalKey(debugLabel: 'list');

  /// Whether an exit from focus mode is already on its way.
  bool _leavingFocus = false;

  @override
  Widget build(BuildContext context) {
    final focus = _maybeReadBloc<FocusModeBloc>(context);
    if (focus == null) {
      return _build(context, focusActive: false, onExit: () {});
    }
    return BlocBuilder<FocusModeBloc, FocusModeState>(
      buildWhen: (before, after) => before.active != after.active,
      builder: (context, state) => _build(
        context,
        focusActive: state.active,
        onExit: () => focus.add(const FocusModeEvent.exited()),
      ),
    );
  }

  Widget _build(
    BuildContext context, {
    required bool focusActive,
    required VoidCallback onExit,
  }) {
    final location = widget.location;
    final layout = widget.layout;
    final designGallery = widget.designGallery;
    return NavigationHistoryListener(
      location: location,
      child: NoteCreationListener(
        // The panes beside the pages are not in a page's Scaffold.
        child: KansoPlainText(
          child: UndoNoticeHost(
            child: ValueListenableBuilder<LayoutClass>(
              valueListenable: layout,
              child: widget.child,
              builder: (context, layoutClass, child) {
                final width = MediaQuery.sizeOf(context).width;
                final listOnly = location?.noteId == null;
                Widget drawerAround(
                  LayoutClass shown,
                  Widget content,
                  double drawer,
                ) {
                  return LibraryDrawerHost(
                    location: location,
                    designGallery: designGallery,
                    width: drawer,
                    edgeDragEnabled: shown != LayoutClass.single || listOnly,
                    child: KansoShortcuts(
                      layout: layoutClass,
                      location: location,
                      child: content,
                    ),
                  );
                }

                if (focusActive && listOnly) {
                  // No note is open: there is nothing to show by itself, so
                  // the mode leaves rather than hide a page it cannot fill.
                  _leaveFocusWithoutNote(onExit);
                }

                if (focusActive && !listOnly) {
                  // Focus mode leaves the note alone: no library, no list,
                  // no tabs and no note bars, with one way back.
                  return TabsSync(
                    enabled: false,
                    location: location,
                    child: TabActions(
                      child: KansoShortcuts(
                        layout: layoutClass,
                        location: location,
                        child: Stack(
                          fit: StackFit.expand,
                          children: <Widget>[
                            child!,
                            FocusModeExit(onExit: onExit),
                          ],
                        ),
                      ),
                    ),
                  );
                }

                final panes = switch (layoutClass) {
                  LayoutClass.single => drawerAround(
                    LayoutClass.single,
                    child!,
                    width * KansoSizes.drawerFraction,
                  ),
                  LayoutClass.double => drawerAround(
                    LayoutClass.double,
                    ListNoteLayout(
                      list: ListSlot(key: listKey, location: location),
                      note: TabbedNote(child: child!),
                    ),
                    KansoSizes.libraryPane,
                  ),
                  // The library pane can be hidden: then it is the drawer
                  // of the two panes left.
                  LayoutClass.triple =>
                    BlocBuilder<LibraryPaneBloc, LibraryPaneState>(
                      builder: (context, pane) {
                        if (!pane.visible) {
                          return drawerAround(
                            LayoutClass.triple,
                            ListNoteLayout(
                              list: ListSlot(key: listKey, location: location),
                              note: TabbedNote(child: child!),
                            ),
                            KansoSizes.libraryPane,
                          );
                        }
                        return KansoShortcuts(
                          layout: layoutClass,
                          location: location,
                          child: TabletLayout(
                            library: LibraryNavigation(
                              location: location,
                              designGallery: designGallery,
                            ),
                            list: ListSlot(key: listKey, location: location),
                            note: TabbedNote(child: child!),
                          ),
                        );
                      },
                    ),
                };
                // Tabs are for the layouts with a pane for the note.
                return TabsSync(
                  enabled: layoutClass != LayoutClass.single,
                  location: location,
                  child: TabActions(child: panes),
                );
              },
            ),
          ),
        ),
      ),
    );
  }

  /// Leaves focus mode when no note is open, as the desktop does when the
  /// note of a window in focus mode closes: the mode has nothing left to
  /// show by itself.
  void _leaveFocusWithoutNote(VoidCallback leave) {
    if (_leavingFocus) {
      return;
    }
    _leavingFocus = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _leavingFocus = false;
      if (mounted) {
        leave();
      }
    });
  }
}

T? _maybeReadBloc<T>(BuildContext context) {
  try {
    return context.read<T>();
  } on ProviderNotFoundException {
    return null;
  }
}
