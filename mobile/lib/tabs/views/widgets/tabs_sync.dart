// Kanso — tabs_sync.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../state/tabs/tabs_bloc.dart';

/// Keeps the tabs and the router in step.
///
/// It tells the tabs' BLoC when the window has panes for tabs ([enabled]) and
/// which note the address shows, and opens, where the person is
/// ([location]), the note the tabs ask for (a tab chosen, closed or
/// reopened, going back) and says it has. The router stays the source of
/// truth: a note it opens goes to the tabs through the navigation history.
class TabsSync extends StatefulWidget {
  const TabsSync({
    required this.enabled,
    required this.location,
    required this.child,
    super.key,
  });

  /// Whether the window has two or three panes, and so tabs.
  final bool enabled;

  /// Where the person is; All Notes when it is neither a place nor a search.
  final BrowseLocation? location;
  final Widget child;

  @override
  State<TabsSync> createState() => _TabsSyncState();
}

class _TabsSyncState extends State<TabsSync> {
  @override
  void initState() {
    super.initState();
    _tell();
  }

  @override
  void didUpdateWidget(TabsSync oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.enabled != widget.enabled) {
      _tell();
    }
  }

  void _tell() {
    context.read<TabsBloc>().add(
      TabsEvent.enabledChanged(
        enabled: widget.enabled,
        shownNoteId: widget.location?.noteId,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<TabsBloc, TabsState>(
      listenWhen: (before, after) =>
          after.pendingNoteId != null &&
          after.pendingNoteId != before.pendingNoteId,
      listener: (context, state) {
        final here =
            widget.location ?? const BrowseLocation.place(ListPlace.all());
        context.go(here.withNote(state.pendingNoteId!).location);
        context.read<TabsBloc>().add(const TabsEvent.consumed());
      },
      child: widget.child,
    );
  }
}
