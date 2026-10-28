// Kanso — search_results.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../note_list/views/widgets/loading_list_pane.dart';
import '../../models/data/search_status.dart';
import '../../state/search/search_bloc.dart';
import 'search_result_tile.dart';

/// What is under a search's field and chips: what to type, the shape of a
/// search while it runs, why it could not, that nothing matched, or the
/// results.
class SearchResults extends StatelessWidget {
  const SearchResults({required this.state, this.openNoteId, super.key});

  final SearchState state;

  /// The note open beside the results.
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return switch (state.status) {
      SearchStatus.idle => KansoEmptyState(
        title: l10n.searchIdleTitle,
        message: l10n.searchIdleMessage,
      ),
      SearchStatus.searching => LoadingListPane(
        semanticLabel: l10n.searchLoading,
      ),
      SearchStatus.failed => Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: KansoMessageCard(
            icon: KansoIcons.search,
            title: l10n.searchFailedTitle,
            message: l10n.searchFailedMessage,
            actions: <Widget>[
              KansoButton(
                label: l10n.retry,
                onPressed: () =>
                    context.read<SearchBloc>().add(const SearchEvent.retried()),
              ),
            ],
          ),
        ),
      ),
      SearchStatus.ready when state.hits.isEmpty => KansoEmptyState(
        title: l10n.searchNoResultsTitle,
        message: l10n.searchNoResultsMessage(state.query.trim()),
      ),
      SearchStatus.ready => BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
        builder: (context, index) {
          final here = BrowseLocation.search(
            query: state.query,
            scope: state.scope,
            notebookId: state.notebookId,
          );
          return ListView.separated(
            padding: EdgeInsets.only(
              bottom: MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
            ),
            itemCount: state.hits.length,
            separatorBuilder: (context, at) =>
                const KansoDivider(indent: KansoSpacing.md),
            itemBuilder: (context, at) {
              final hit = state.hits[at];
              return SearchResultTile(
                hit: hit,
                notebook: index.notebookPath(hit.row.notebookId) ?? '',
                open: hit.row.id == openNoteId,
                onTap: () {
                  context.read<NavigationHistoryBloc>().add(
                    NavigationHistoryEvent.visited(hit.row.id),
                  );
                  context.go(here.withNote(hit.row.id).location);
                },
              );
            },
          );
        },
      ),
    };
  }
}
