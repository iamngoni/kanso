// Kanso — note_citation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// The decoder reports a bad escape as an ArgumentError, the only way to ask.
// ignore_for_file: avoid_catching_errors

import '../../core/router/deep_links.dart';

/// What a citation in an answer links to: the app's own link to a note,
/// `kanso://open/note/<id>` (see `redirectLocation`).
const String noteCitationPrefix = '$kansoScheme://$kansoOpenHost/note/';

/// How the Rust core writes the link of a citation.
const String coreCitationPrefix = '$kansoScheme://note/';

/// The link that cites the note [id].
///
/// Parentheses are written out, since a Markdown link ends at the first one.
String noteCitationHref(String id) {
  final encoded = Uri.encodeComponent(
    id,
  ).replaceAll('(', '%28').replaceAll(')', '%29');
  return '$noteCitationPrefix$encoded';
}

/// The id of the note [href] cites, or null when it cites none.
///
/// The core writes its citations as `kanso://note/<id>`; both forms are read.
String? noteIdOfCitation(String href) {
  final prefix = href.startsWith(noteCitationPrefix)
      ? noteCitationPrefix
      : href.startsWith(coreCitationPrefix)
      ? coreCitationPrefix
      : null;
  if (prefix == null) {
    return null;
  }
  final encoded = href.substring(prefix.length);
  if (encoded.isEmpty) {
    return null;
  }
  try {
    final id = Uri.decodeComponent(encoded);
    return id.isEmpty ? null : id;
  } on ArgumentError {
    return null;
  }
}
