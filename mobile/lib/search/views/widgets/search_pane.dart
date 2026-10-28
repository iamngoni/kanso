// Kanso — search_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_chip.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_input.dart';
import '../../models/data/search_status.dart';
import '../../state/search/search_bloc.dart';
import 'meaning_search_results.dart';
import 'search_results.dart';

/// Search: the focused field, the chips for where to look and the results
/// with what matched highlighted. On a tablet it takes the place of the
/// note list. It reads the `SearchBloc` above it.
///
/// Cancel leaves the search: back to what it was opened from, or All Notes.
/// [initialQuery] is what the field starts with; [openNoteId] is the note
/// open beside the results.
class SearchPane extends StatelessWidget {
  const SearchPane({this.initialQuery = '', this.openNoteId, super.key});

  final String initialQuery;
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bloc = context.read<SearchBloc>();
    return ColoredBox(
      color: colors.list,
      child: BlocBuilder<SearchBloc, SearchState>(
        builder: (context, state) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              SafeArea(
                bottom: false,
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    0,
                  ),
                  child: Row(
                    children: <Widget>[
                      Expanded(
                        child: KansoSearchInput(
                          // A different search address is a different field.
                          key: ValueKey<String>(initialQuery),
                          hint: l10n.searchHint,
                          initialText: initialQuery,
                          autofocus: initialQuery.isEmpty,
                          onChanged: (text) =>
                              bloc.add(SearchEvent.queryChanged(text)),
                          onCleared: () =>
                              bloc.add(const SearchEvent.cleared()),
                          clearLabel: l10n.searchClear,
                        ),
                      ),
                      KansoPressable(
                        onTap: () => leave(context),
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: KansoSpacing.sm,
                            vertical: KansoSpacing.sm,
                          ),
                          child: Text(
                            l10n.searchCancel,
                            style: context.typography.body.copyWith(
                              color: colors.accent,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  KansoSpacing.xs,
                  KansoSpacing.md,
                  KansoSpacing.sm,
                ),
                child: Wrap(
                  spacing: KansoSpacing.xs,
                  runSpacing: KansoSpacing.xs,
                  children: <Widget>[
                    KansoChip(
                      label: l10n.searchFilterAll,
                      selected: state.scope == SearchScopeKind.all,
                      onTap: () => bloc.add(
                        const SearchEvent.scopeChanged(SearchScopeKind.all),
                      ),
                    ),
                    KansoChip(
                      label: l10n.searchFilterTitles,
                      selected: state.scope == SearchScopeKind.titles,
                      onTap: () => bloc.add(
                        const SearchEvent.scopeChanged(SearchScopeKind.titles),
                      ),
                    ),
                    KansoChip(
                      label: l10n.searchFilterTags,
                      selected: state.scope == SearchScopeKind.tags,
                      onTap: () => bloc.add(
                        const SearchEvent.scopeChanged(SearchScopeKind.tags),
                      ),
                    ),
                    // Only a search opened from a notebook can look in it.
                    if (state.notebookId != null)
                      KansoChip(
                        label: l10n.searchFilterNotebook,
                        selected: state.scope == SearchScopeKind.notebook,
                        onTap: () => bloc.add(
                          const SearchEvent.scopeChanged(
                            SearchScopeKind.notebook,
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              const KansoDivider(),
              if (state.status == SearchStatus.ready)
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.sm,
                    KansoSpacing.md,
                    KansoSpacing.xxs,
                  ),
                  child: Text(
                    l10n.searchResultCount(state.hits.length),
                    style: context.typography.sectionLabel,
                  ),
                ),
              Expanded(
                child: Column(
                  children: <Widget>[
                    if (state.meaningHits.isNotEmpty)
                      MeaningSearchResults(rows: state.meaningHits),
                    Expanded(
                      child: SearchResults(
                        state: state,
                        openNoteId: openNoteId,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  /// Leaves the search for the page it was opened from.
  void leave(BuildContext context) {
    if (context.canPop()) {
      context.pop();
    } else {
      context.go(KansoRoutes.notes);
    }
  }
}
