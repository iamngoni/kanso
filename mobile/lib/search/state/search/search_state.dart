// Kanso — search_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'search_bloc.dart';

@freezed
abstract class SearchState with _$SearchState {
  const factory SearchState({
    /// The words being looked for.
    @Default('') String query,

    /// Where to look.
    @Default(SearchScopeKind.all) SearchScopeKind scope,

    /// The notebook the search was opened from, if any: what the notebook
    /// scope looks in.
    String? notebookId,
    @Default(SearchStatus.idle) SearchStatus status,

    /// What was found for [query] in [scope], best match first.
    @Default(<Hit>[]) List<Hit> hits,

    /// Notes found with the local semantic model, when it is available.
    @Default(<RelatedRow>[]) List<RelatedRow> meaningHits,

    /// Why the last search failed, while [status] is failed.
    ApplicationError? error,
  }) = _SearchState;

  const SearchState._();

  /// Whether the search has words to look for.
  bool get hasQuery => query.trim().isNotEmpty;

  /// [scope] as the library takes it: the notebook scope with no notebook to
  /// look in is a search of everything.
  SearchScope get libraryScope {
    final book = notebookId;
    return switch (scope) {
      SearchScopeKind.all => const SearchScope.all(),
      SearchScopeKind.titles => const SearchScope.titles(),
      SearchScopeKind.tags => const SearchScope.tags(),
      SearchScopeKind.notebook =>
        book == null ? const SearchScope.all() : SearchScope.notebook(id: book),
    };
  }
}
