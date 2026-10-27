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
import '../../state/library_pane/library_pane_bloc.dart';
import 'kanso_shortcuts.dart';
import 'library_drawer_host.dart';
import 'library_navigation.dart';
import 'list_note_layout.dart';
import 'list_slot.dart';
import 'navigation_history_listener.dart';
import 'note_creation_listener.dart';
import 'tablet_layout.dart';

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

  @override
  Widget build(BuildContext context) {
    final location = widget.location;
    final layout = widget.layout;
    final designGallery = widget.designGallery;
    return NavigationHistoryListener(
      location: location,
      child: NoteCreationListener(
        // The panes beside the pages are not in a page's Scaffold.
        child: KansoPlainText(
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

              return switch (layoutClass) {
                LayoutClass.single => drawerAround(
                  LayoutClass.single,
                  child!,
                  width * KansoSizes.drawerFraction,
                ),
                LayoutClass.double => drawerAround(
                  LayoutClass.double,
                  ListNoteLayout(
                    list: ListSlot(key: listKey, location: location),
                    note: child!,
                  ),
                  KansoSizes.libraryPane,
                ),
                // The library pane can be hidden: then it is the drawer of the
                // two panes left.
                LayoutClass.triple =>
                  BlocBuilder<LibraryPaneBloc, LibraryPaneState>(
                    builder: (context, pane) {
                      if (!pane.visible) {
                        return drawerAround(
                          LayoutClass.triple,
                          ListNoteLayout(
                            list: ListSlot(key: listKey, location: location),
                            note: child!,
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
                          note: child!,
                        ),
                      );
                    },
                  ),
              };
            },
          ),
        ),
      ),
    );
  }
}
