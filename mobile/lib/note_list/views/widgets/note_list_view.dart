// Kanso — note_list_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/day_grouping.dart';
import '../../../core/utils/list_labels.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_refresh_control.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_swipe_host.dart';
import '../../models/data/note_list_entry.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'note_list_row.dart';

/// The notes of a list under their headers, scrolling.
///
/// A tap opens a note in the place [location] is in, and while several
/// are being chosen it chooses or lets go of it. [openNoteId] is the note
/// open beside the list.
class NoteListView extends StatelessWidget {
  const NoteListView({
    required this.state,
    required this.index,
    required this.location,
    this.openNoteId,
    this.bottomClearance = 0,
    super.key,
  });

  final NoteListState state;
  final LibraryIndexState index;
  final BrowseLocation location;
  final String? openNoteId;

  /// How much room the list leaves below its last row for something afloat
  /// over it, besides the usual gutter.
  final double bottomClearance;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final now = clock.now();
    final today = localDayOf(now.millisecondsSinceEpoch);
    final byCreated = state.shape?.date == DateField.created;
    final entries = state.entries;
    final bloc = context.read<NoteListBloc>();
    // Only an account has anything to pull: a phone that is signed out
    // just bounces.
    final signedIn = context.select<SyncStatusBloc, bool>(
      (sync) => sync.state.signedIn,
    );
    // Rows keep what they hold (a swipe in progress) by what they are, not
    // by where they are, so a note that moves takes it along.
    Object keyOf(NoteListEntry entry) => switch (entry) {
      HeaderEntry(:final bucket) => bucket,
      NoteEntry(:final row) => row.id,
    };
    final positions = <Object, int>{
      for (var at = 0; at < entries.length; at++) keyOf(entries[at]): at,
    };
    return KansoSwipeHost(
      child: CustomScrollView(
        physics: signedIn ? const AlwaysScrollableScrollPhysics() : null,
        slivers: <Widget>[
          if (signedIn) KansoRefreshControl(onRefresh: () => syncNow(context)),
          SliverPadding(
            padding: EdgeInsets.only(
              top: KansoSpacing.xxs,
              bottom:
                  MediaQuery.paddingOf(context).bottom +
                  KansoSpacing.md +
                  bottomClearance,
            ),
            sliver: SliverList.builder(
              itemCount: entries.length,
              findChildIndexCallback: (key) =>
                  key is ValueKey<Object> ? positions[key.value] : null,
              itemBuilder: (context, at) {
                final entry = entries[at];
                return switch (entry) {
                  HeaderEntry(:final bucket) => KansoSectionHeader(
                    key: ValueKey<Object>(bucket),
                    label: bucketLabel(l10n, bucket, today: today),
                  ),
                  NoteEntry(:final row) => Column(
                    key: ValueKey<Object>(row.id),
                    children: <Widget>[
                      if (at > 0 && entries[at - 1] is NoteEntry)
                        const KansoDivider(indent: KansoSpacing.md),
                      NoteListRow(
                        row: row,
                        index: index,
                        time: rowTimeLabel(
                          l10n,
                          byCreated ? row.createdMs : row.updatedMs,
                          now: now,
                        ),
                        open: row.id == openNoteId,
                        checked: state.selecting
                            ? state.selected.contains(row.id)
                            : null,
                        onTap: () => state.selecting
                            ? bloc.add(NoteListEvent.selectionToggled(row.id))
                            : openNote(context, row.id),
                        onLongPress: () => state.selecting
                            ? bloc.add(NoteListEvent.selectionToggled(row.id))
                            : bloc.add(NoteListEvent.selectionStarted(row.id)),
                      ),
                    ],
                  ),
                };
              },
            ),
          ),
        ],
      ),
    );
  }

  /// Asks for a sync, and waits for it to end.
  Future<void> syncNow(BuildContext context) {
    final sync = context.read<SyncStatusBloc>()
      ..add(const SyncStatusEvent.syncRequested());
    return sync.stream.firstWhere((state) => !state.syncing);
  }

  /// Opens the note [id] in the place being listed, and tells the history.
  void openNote(BuildContext context, String id) {
    context.read<NavigationHistoryBloc>().add(
      NavigationHistoryEvent.visited(id),
    );
    context.go(location.withNote(id).location);
  }
}
