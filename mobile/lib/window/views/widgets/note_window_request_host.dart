// Kanso — note_window_request_host.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../models/data/note_window_request.dart';
import '../../models/repos/abs/note_window_repository.dart';

/// Does in the main window what a note's own window hands to it (see
/// [NoteWindowRequest]): a new note, a search, a link opened. The native side
/// has brought the main window forward already.
///
/// It must be below the app's BLoCs and is given the [router] the main
/// window navigates with.
class NoteWindowRequestHost extends StatefulWidget {
  const NoteWindowRequestHost({
    required this.windows,
    required this.router,
    required this.child,
    super.key,
  });

  final NoteWindowRepository windows;
  final GoRouter router;
  final Widget child;

  @override
  State<NoteWindowRequestHost> createState() => _NoteWindowRequestHostState();
}

class _NoteWindowRequestHostState extends State<NoteWindowRequestHost> {
  StreamSubscription<NoteWindowRequest>? _requests;

  @override
  void initState() {
    super.initState();
    _requests = widget.windows.requests().listen(_handle);
  }

  @override
  void didUpdateWidget(NoteWindowRequestHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.windows != widget.windows) {
      unawaited(_requests?.cancel());
      _requests = widget.windows.requests().listen(_handle);
    }
  }

  @override
  void dispose() {
    unawaited(_requests?.cancel());
    super.dispose();
  }

  void _handle(NoteWindowRequest request) {
    if (!mounted) {
      return;
    }
    switch (request) {
      case NewNoteRequest():
        context.read<NoteCreationBloc>().add(
          const NoteCreationEvent.requested(ListPlace.all()),
        );
      case NewNoteTitledRequest(:final title):
        context.read<NoteCreationBloc>().add(NoteCreationEvent.titled(title));
      case SearchRequest():
        widget.router.go(const BrowseLocation.search().location);
      case OpenNoteRequest(:final noteId):
        widget.router.go(
          BrowseLocation.place(const ListPlace.all(), noteId: noteId).location,
        );
      case PaletteRequest():
        // The command palette is package C's; it acts on this once it is in
        // the main window.
        break;
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
