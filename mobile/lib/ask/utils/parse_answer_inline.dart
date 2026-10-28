// Kanso — parse_answer_inline.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/utils/parse_inline_markdown.dart';
import '../models/data/answer_span.dart';
import 'note_citation.dart';

/// `![alt](address)`: an image, which an answer never draws.
final RegExp _image = RegExp(r'!\[([^\]]*)\]\([^)]*\)');

/// [source] cut into the spans of one block: text with its marks, citations
/// and links.
///
/// An image is its description in plain text, and a link to anything but a
/// note or an `http` or `https` page is plain text: nothing here loads a
/// picture, runs a script or opens another app. An unfinished mark is plain
/// text, since an answer is read while it is written.
List<AnswerSpan> parseAnswerInline(
  String source, {
  Set<String> usedNoteIds = const <String>{},
}) {
  final runs = parseInlineMarkdown(
    source.replaceAllMapped(_image, (match) => match.group(1)!),
  );
  final spans = <AnswerSpan>[];

  void add(
    String text, {
    bool bold = false,
    bool italic = false,
    bool code = false,
  }) {
    if (text.isEmpty) {
      return;
    }
    final last = spans.isEmpty ? null : spans.last;
    if (last is AnswerText &&
        last.bold == bold &&
        last.italic == italic &&
        last.code == code) {
      spans[spans.length - 1] = last.copyWith(text: last.text + text);
    } else {
      spans.add(
        AnswerSpan.text(text, bold: bold, italic: italic, code: code),
      );
    }
  }

  for (var i = 0; i < runs.length; i++) {
    final run = runs[i];
    switch (run.kind) {
      case InlineKind.plain:
        add(run.text);
      case InlineKind.bold:
        add(run.text, bold: true);
      case InlineKind.italic:
        add(run.text, italic: true);
      case InlineKind.code:
        add(run.text, code: true);
      case InlineKind.wiki:
        add('[[${run.text}]]');
      case InlineKind.link:
        // The link's address is two runs on: `[`, text, `](`, address, `)`.
        final address = i + 2 < runs.length ? runs[i + 2].text : '';
        final noteId = noteIdOfCitation(address);
        if (noteId != null && usedNoteIds.contains(noteId)) {
          spans.add(AnswerSpan.note(text: run.text, noteId: noteId));
        } else if (_isWebAddress(address)) {
          spans.add(AnswerSpan.web(text: run.text, url: address));
        } else {
          add(run.text);
        }
      case InlineKind.marker || InlineKind.url:
        break;
    }
  }
  return spans;
}

bool _isWebAddress(String address) {
  final uri = Uri.tryParse(address);
  return uri != null &&
      (uri.scheme == 'http' || uri.scheme == 'https') &&
      uri.host.isNotEmpty;
}
