// Kanso — linked_text.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

/// The in-app link scheme of a note: `inkdrop://note/<id>`.
const String noteLinkPrefix = 'inkdrop://note/';

final RegExp _link = RegExp(r'\[([^\]]+)\]\(([^)\s]+)\)');

/// The note a link points at (`note:abc`), or null when it is not one.
String? linkedNote(String url) {
  if (url.startsWith(noteLinkPrefix)) {
    final rest = url.substring(noteLinkPrefix.length).split('#').first;
    if (rest.isEmpty) {
      return null;
    }
    return rest.startsWith('note:') ? rest : 'note:$rest';
  }
  if (url.startsWith('inkdrop://note:')) {
    final rest = url.substring('inkdrop://'.length).split('#').first;
    return rest.length > 'note:'.length ? rest : null;
  }
  return null;
}

/// A piece of the model's paragraph: plain words, or words that open a note.
class LinkPiece {
  const LinkPiece(this.text, [this.noteId]);

  final String text;

  /// The note the words open, when they are a citation.
  final String? noteId;
}

/// Splits Markdown text with `[words](inkdrop://note/id)` links into pieces.
/// A link that is not to a note is kept as its words.
List<LinkPiece> linkPieces(String markdown) {
  final out = <LinkPiece>[];
  var at = 0;
  for (final match in _link.allMatches(markdown)) {
    if (match.start > at) {
      out.add(LinkPiece(markdown.substring(at, match.start)));
    }
    final label = match.group(1)!;
    final note = linkedNote(match.group(2)!);
    out.add(LinkPiece(label, note));
    at = match.end;
  }
  if (at < markdown.length) {
    out.add(LinkPiece(markdown.substring(at)));
  }
  return out;
}

/// The paragraph as spans, citations underlined in [linkColor] and opening
/// their note on a tap. The recognizers are returned to be disposed.
(List<InlineSpan>, List<TapGestureRecognizer>) linkedSpans(
  String markdown, {
  required Color linkColor,
  required void Function(String noteId) onOpen,
}) {
  final recognizers = <TapGestureRecognizer>[];
  final spans = <InlineSpan>[];
  for (final piece in linkPieces(markdown)) {
    final note = piece.noteId;
    if (note == null) {
      spans.add(TextSpan(text: piece.text));
      continue;
    }
    final tap = TapGestureRecognizer()..onTap = () => onOpen(note);
    recognizers.add(tap);
    spans.add(
      TextSpan(
        text: piece.text,
        recognizer: tap,
        style: TextStyle(
          color: linkColor,
          decoration: TextDecoration.underline,
          decorationColor: linkColor.withValues(alpha: 0.5),
        ),
      ),
    );
  }
  return (spans, recognizers);
}
