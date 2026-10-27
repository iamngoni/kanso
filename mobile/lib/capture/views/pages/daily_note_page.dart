// Kanso — daily_note_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/daily_note/daily_note_bloc.dart';
import '../widgets/daily_note_failed_pane.dart';
import '../widgets/daily_note_loading_pane.dart';

/// Where `kanso://open/daily` lands: it asks for today's note, on the
/// phone's date, and goes to it as soon as it is found or made.
class DailyNotePage extends StatelessWidget {
  const DailyNotePage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<DailyNoteBloc>(
      create: (_) =>
          getIt<DailyNoteBloc>()..add(DailyNoteEvent.requested(clock.now())),
      child: BlocConsumer<DailyNoteBloc, DailyNoteState>(
        listenWhen: (before, after) =>
            before.status != after.status && after.status == LoadStatus.ready,
        listener: (context, state) {
          final id = state.noteId;
          if (id != null) {
            context.go(
              const BrowseLocation.place(ListPlace.all()).withNote(id).location,
            );
          }
        },
        builder: (context, state) {
          return Scaffold(
            backgroundColor: context.colors.editor,
            body: SafeArea(
              child: switch (state.status) {
                LoadStatus.failed => DailyNoteFailedPane(
                  onRetry: () => context.read<DailyNoteBloc>().add(
                    DailyNoteEvent.requested(clock.now()),
                  ),
                  onLeave: () => context.go(KansoRoutes.notes),
                ),
                LoadStatus.loading ||
                LoadStatus.ready => const DailyNoteLoadingPane(),
              },
            ),
          );
        },
      ),
    );
  }
}
