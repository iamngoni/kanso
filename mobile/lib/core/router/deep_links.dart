// Kanso — deep_links.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart' show ListPlace;
import '../models/data/browse_location.dart';
import '../models/data/new_link_kind.dart';
import 'kanso_routes.dart';

/// The scheme of the app's own links.
const String kansoScheme = 'kanso';

/// The host of the links that open something: `kanso://open/note/<id>`.
const String kansoOpenHost = 'open';

/// Where the address [uri] should go instead, or null when it is already
/// where it should be.
///
/// It reads two things. The app's links:
///
/// * `kanso://open/note/<id>` opens the note in All Notes;
/// * `kanso://open/notebook/<id>` lists the notebook;
/// * `kanso://open/search?q=…` searches (with `scope` and `in` as in the
///   search address);
/// * `kanso://open/daily` is today's daily note (`/daily`, which finds or
///   makes it and goes to it);
/// * `kanso://open/new` is a new note in Edit (`/new`),
///   `kanso://open/new?voice=1` one that starts in its text, for dictation,
///   and `kanso://open/new?task=1` the capture sheet in Task mode (see
///   [newLinkKind]).
///
/// And the short forms those become, which the router accepts too: `/` goes
/// to All Notes, `/note/<id>` to the note in All Notes, `/notebook/<id>` to
/// the notebook. A link from another host, or one that names nothing, goes
/// to All Notes. The router matches paths and ignores the scheme and host,
/// so this is the one place that knows which is which.
String? redirectLocation(Uri uri) {
  final foreign = uri.hasScheme && uri.scheme != kansoScheme;
  if (foreign) {
    return KansoRoutes.notes;
  }
  final isLink = uri.scheme == kansoScheme;
  if (isLink && uri.host != kansoOpenHost) {
    return KansoRoutes.notes;
  }
  final segments = <String>[
    for (final segment in uri.pathSegments)
      if (segment.isNotEmpty) segment,
  ];
  if (segments.isEmpty) {
    return KansoRoutes.notes;
  }
  final head = segments.first;
  if (segments.length == 2 && head == 'note') {
    return const BrowseLocation.place(
      ListPlace.all(),
    ).withNote(segments.last).location;
  }
  if (segments.length == 2 && head == 'notebook') {
    return '${KansoRoutes.notebooks}/${Uri.encodeComponent(segments.last)}';
  }
  if (isLink) {
    // Everything else a link can say is already an address of the app; the
    // path and query are kept, the scheme and host dropped.
    final path = '/${segments.map(Uri.encodeComponent).join('/')}';
    return uri.hasQuery ? '$path?${uri.query}' : path;
  }
  return null;
}

/// What the `/new` address with [query] starts: the capture sheet in Task
/// mode when `task` is `1`, a note to dictate into when `voice` is `1`, and
/// otherwise a note open to write its title. Task wins over voice, as a
/// task has no voice form. Any other value of either is left alone, so an
/// address made by hand that says `task=0` is a plain note.
NewLinkKind newLinkKind(Map<String, String> query) {
  if (query[KansoRoutes.taskParameter] == '1') {
    return NewLinkKind.task;
  }
  if (query[KansoRoutes.voiceParameter] == '1') {
    return NewLinkKind.voice;
  }
  return NewLinkKind.note;
}

/// Whether [uri], an address of the app (no scheme), asks for the capture
/// sheet in Task mode rather than for a page: `/new?task=1`.
bool opensTaskCapture(Uri uri) {
  final path = uri.path.endsWith('/') && uri.path.length > 1
      ? uri.path.substring(0, uri.path.length - 1)
      : uri.path;
  return path == KansoRoutes.newNote &&
      newLinkKind(uri.queryParameters) == NewLinkKind.task;
}

/// Where the app stays while a sheet a link asked for is open over it:
/// where it was ([current]), so the sheet opens over what the person was
/// doing. With nothing there yet (a cold start), or when that was an
/// address that makes something each time it is visited (`/new`, `/daily`),
/// it is All Notes.
String locationUnderSheet(Uri? current) {
  if (current == null || current.path.isEmpty || current.path == '/') {
    return KansoRoutes.notes;
  }
  final path = current.path;
  if (path == KansoRoutes.newNote ||
      path.startsWith('${KansoRoutes.newNote}/') ||
      path == KansoRoutes.daily ||
      path.startsWith('${KansoRoutes.daily}/')) {
    return KansoRoutes.notes;
  }
  return current.toString();
}
