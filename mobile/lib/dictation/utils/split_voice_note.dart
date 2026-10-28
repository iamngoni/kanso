// Kanso — split_voice_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/voice_note_text.dart';

/// How long a title may be before it is cut short.
const int voiceNoteTitleLength = 80;

/// A voice note's title and body, as the desktop's quick capture makes them
/// (`split_capture`): a heading or a short first line is the title and the
/// lines after it the body. A long first line, or one that starts a list or
/// a quote, is cut short for the title and the body keeps all of the text,
/// so nothing said is lost. Null for text that is only spaces.
VoiceNoteText? splitVoiceNote(String text) {
  final trimmed = text.trim();
  if (trimmed.isEmpty) {
    return null;
  }
  final newline = trimmed.indexOf('\n');
  final first = (newline < 0 ? trimmed : trimmed.substring(0, newline)).trim();
  final rest = withoutBlankLead(
    (newline < 0 ? '' : trimmed.substring(newline + 1)).trimRight(),
  );
  final heading = headingOf(first);
  if (heading != null) {
    return VoiceNoteText(title: cutTitle(heading), body: rest);
  }
  final marker = const <String>[
    '-',
    '*',
    '>',
    '+',
    '|',
    '`',
    '#',
  ].any(first.startsWith);
  if (!marker && first.runes.length <= voiceNoteTitleLength) {
    return VoiceNoteText(title: first, body: rest);
  }
  final plain = first.replaceFirst(RegExp('^[-*>+|`# ]+'), '');
  return VoiceNoteText(
    title: cutTitle(plain.isEmpty ? first : plain),
    body: trimmed,
  );
}

/// The text of a Markdown heading line, or null if [line] is not one.
String? headingOf(String line) {
  if (!line.startsWith('#')) {
    return null;
  }
  final marks = line.replaceFirst(RegExp('^#+'), '');
  if (!marks.startsWith(' ')) {
    return null;
  }
  final heading = marks.trim();
  return heading.isEmpty ? null : heading;
}

/// [text] from its first line that has more than spaces in it.
String withoutBlankLead(String text) {
  var rest = text;
  while (true) {
    final newline = rest.indexOf('\n');
    if (newline < 0 || rest.substring(0, newline).trim().isNotEmpty) {
      break;
    }
    rest = rest.substring(newline + 1);
  }
  return rest.trim().isEmpty ? '' : rest;
}

/// At most [voiceNoteTitleLength] characters, cut at a space, with an
/// ellipsis when cut.
String cutTitle(String line) {
  final characters = line.runes.toList();
  if (characters.length <= voiceNoteTitleLength) {
    return line;
  }
  final cut = String.fromCharCodes(characters.take(voiceNoteTitleLength));
  final space = cut.lastIndexOf(' ');
  final kept = space > voiceNoteTitleLength ~/ 2
      ? cut.substring(0, space)
      : cut;
  return '${kept.trimRight()}…';
}
