// Kanso — browse_location.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart';
import 'search_scope_kind.dart';

part 'browse_location.freezed.dart';

/// Where in the notes the person is: a place or a search, and the note open
/// in it, if any. It is what the app's address (`/notes/note/n1`,
/// `/search?q=tea`) says, and [location] and [BrowseLocation.parse] turn it
/// into that address and back.
///
/// Ids are written with `Uri.encodeComponent`: note ids contain `:`.
@freezed
sealed class BrowseLocation with _$BrowseLocation {
  /// A list of notes: a place of the library.
  const factory BrowseLocation.place(ListPlace place, {String? noteId}) =
      PlaceLocation;

  /// A search for [query] in [scope]. [notebookId] is the notebook it was
  /// opened from, the one the notebook scope looks in.
  const factory BrowseLocation.search({
    @Default('') String query,
    @Default(SearchScopeKind.all) SearchScopeKind scope,
    String? notebookId,
    String? noteId,
  }) = SearchLocation;

  /// The tasks of all the notes: the Tasks place.
  const factory BrowseLocation.tasks({String? noteId}) = TasksLocation;

  const BrowseLocation._();

  /// The address of this location.
  String get location {
    final note = noteId == null ? '' : '/note/${Uri.encodeComponent(noteId!)}';
    return switch (this) {
      PlaceLocation(:final place) => '${_placePath(place)}$note',
      SearchLocation(:final query, :final scope, :final notebookId) =>
        '/search$note${_searchQuery(query, scope, notebookId)}',
      TasksLocation() => '/tasks$note',
    };
  }

  /// This location with the note [id] open.
  BrowseLocation withNote(String id) => switch (this) {
    final PlaceLocation here => here.copyWith(noteId: id),
    final SearchLocation here => here.copyWith(noteId: id),
    final TasksLocation here => here.copyWith(noteId: id),
  };

  /// This location with no note open: the list alone.
  BrowseLocation get withoutNote => switch (this) {
    final PlaceLocation here => here.copyWith(noteId: null),
    final SearchLocation here => here.copyWith(noteId: null),
    final TasksLocation here => here.copyWith(noteId: null),
  };

  /// The location [uri] is the address of, or null for an address that is
  /// not a place, a search, or a note in one.
  static BrowseLocation? parse(Uri uri) {
    final segments = <String>[
      for (final segment in uri.pathSegments)
        if (segment.isNotEmpty) segment,
    ];
    if (segments.isEmpty) {
      return null;
    }
    final head = segments.first;
    final rest = segments.sublist(1);
    final ListPlace? place;
    final List<String> tail;
    switch (head) {
      case 'notes':
        place = const ListPlace.all();
        tail = rest;
      case 'pinned':
        place = const ListPlace.pinned();
        tail = rest;
      case 'recent':
        place = const ListPlace.recent();
        tail = rest;
      case 'trash':
        place = const ListPlace.trash();
        tail = rest;
      case 'notebooks' when rest.isNotEmpty:
        place = ListPlace.notebook(id: rest.first);
        tail = rest.sublist(1);
      case 'tags' when rest.isNotEmpty:
        place = ListPlace.tag(id: rest.first);
        tail = rest.sublist(1);
      case 'tasks':
        final noteId = _noteIdOf(rest);
        if (rest.isNotEmpty && noteId == null) {
          return null;
        }
        return BrowseLocation.tasks(noteId: noteId);
      case 'search':
        final noteId = _noteIdOf(rest);
        if (rest.isNotEmpty && noteId == null) {
          return null;
        }
        final query = uri.queryParameters;
        final scope = SearchScopeKind.values.firstWhere(
          (kind) => kind.name == query['scope'],
          orElse: () => SearchScopeKind.all,
        );
        return BrowseLocation.search(
          query: query['q'] ?? '',
          scope: scope,
          notebookId: _nonEmpty(query['in']),
          noteId: noteId,
        );
      default:
        return null;
    }
    final noteId = _noteIdOf(tail);
    if (tail.isNotEmpty && noteId == null) {
      return null;
    }
    return BrowseLocation.place(place, noteId: noteId);
  }
}

/// The id in `['note', id]`, or null when the segments are anything else.
String? _noteIdOf(List<String> segments) {
  if (segments.length == 2 && segments.first == 'note') {
    return segments.last;
  }
  return null;
}

String? _nonEmpty(String? value) =>
    value == null || value.isEmpty ? null : value;

String _placePath(ListPlace place) => switch (place) {
  ListPlace_All() => '/notes',
  ListPlace_Pinned() => '/pinned',
  ListPlace_Recent() => '/recent',
  ListPlace_Trash() => '/trash',
  ListPlace_Notebook(:final id) => '/notebooks/${Uri.encodeComponent(id)}',
  ListPlace_Tag(:final id) => '/tags/${Uri.encodeComponent(id)}',
};

String _searchQuery(String query, SearchScopeKind scope, String? notebookId) {
  final parameters = <String, String>{
    if (query.isNotEmpty) 'q': query,
    if (scope != SearchScopeKind.all) 'scope': scope.name,
    if (notebookId != null) 'in': notebookId,
  };
  if (parameters.isEmpty) {
    return '';
  }
  return '?${Uri(queryParameters: parameters).query}';
}
