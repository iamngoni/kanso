// Kanso — shared_note_live.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../injection.dart';
import '../../state/shared_note_live/shared_note_live_bloc.dart';

/// Keeps the live room of the shared note [noteId] open for as long as this
/// is in the tree: wrap the note's page with it. A note that is not shared
/// has no room, and nothing happens.
///
/// The room is closed when the app goes to the background (the library does
/// that); coming back to the front joins it again.
class SharedNoteLive extends StatefulWidget {
  const SharedNoteLive({required this.noteId, required this.child, super.key});

  final String noteId;
  final Widget child;

  @override
  State<SharedNoteLive> createState() => _SharedNoteLiveState();
}

class _SharedNoteLiveState extends State<SharedNoteLive> {
  late final SharedNoteLiveBloc bloc;
  late final AppLifecycleListener lifecycle;

  @override
  void initState() {
    super.initState();
    bloc = getIt<SharedNoteLiveBloc>()
      ..add(SharedNoteLiveEvent.opened(widget.noteId));
    lifecycle = AppLifecycleListener(
      onResume: () => bloc.add(const SharedNoteLiveEvent.resumed()),
    );
  }

  @override
  void didUpdateWidget(SharedNoteLive oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.noteId != widget.noteId) {
      bloc.add(SharedNoteLiveEvent.opened(widget.noteId));
    }
  }

  @override
  void dispose() {
    lifecycle.dispose();
    unawaited(bloc.close());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SharedNoteLiveBloc>.value(
      value: bloc,
      child: widget.child,
    );
  }
}
