// Kanso — parse_answer_blocks.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/answer_block.dart';

final RegExp _heading = RegExp(r'^(#{1,6})\s+(.*?)\s*#*\s*$');
final RegExp _rule = RegExp(r'^\s{0,3}([-*_])(\s*\1){2,}\s*$');
final RegExp _bullet = RegExp(r'^(\s*)[-*+]\s+(.*)$');
final RegExp _numbered = RegExp(r'^(\s*)(\d{1,9})[.)]\s+(.*)$');

/// An answer's Markdown cut into the blocks that are drawn: headings,
/// paragraphs, lists, code, quotations and rules.
///
/// It is forgiving, for text that is still being written: a code fence that
/// is not closed runs to the end, and a line that is none of the above is
/// part of a paragraph. Nothing is dropped except blank lines.
List<AnswerBlock> parseAnswerBlocks(String markdown) {
  final blocks = <AnswerBlock>[];
  final lines = markdown.replaceAll('\r\n', '\n').split('\n');
  final paragraph = <String>[];
  final quote = <String>[];
  var items = <AnswerListItem>[];

  void endParagraph() {
    if (paragraph.isNotEmpty) {
      blocks.add(AnswerBlock.paragraph(paragraph.join(' ')));
      paragraph.clear();
    }
  }

  void endQuote() {
    if (quote.isNotEmpty) {
      blocks.add(AnswerBlock.quote(quote.join(' ')));
      quote.clear();
    }
  }

  void endList() {
    if (items.isNotEmpty) {
      blocks.add(AnswerBlock.list(items));
      items = <AnswerListItem>[];
    }
  }

  void endAll() {
    endParagraph();
    endQuote();
    endList();
  }

  var i = 0;
  while (i < lines.length) {
    final line = lines[i];
    final trimmed = line.trim();
    if (trimmed.startsWith('```') || trimmed.startsWith('~~~')) {
      endAll();
      final fence = trimmed.substring(0, 3);
      final code = <String>[];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence)) {
        code.add(lines[i]);
        i++;
      }
      blocks.add(AnswerBlock.code(code.join('\n')));
      i++;
      continue;
    }
    if (trimmed.isEmpty) {
      endAll();
      i++;
      continue;
    }
    final heading = _heading.firstMatch(trimmed);
    if (heading != null) {
      endAll();
      blocks.add(
        AnswerBlock.heading(
          level: heading.group(1)!.length,
          text: heading.group(2)!,
        ),
      );
      i++;
      continue;
    }
    if (_rule.hasMatch(line)) {
      endAll();
      blocks.add(const AnswerBlock.rule());
      i++;
      continue;
    }
    if (trimmed.startsWith('>')) {
      endParagraph();
      endList();
      quote.add(trimmed.substring(1).trim());
      i++;
      continue;
    }
    final bullet = _bullet.firstMatch(line);
    final numbered = _numbered.firstMatch(line);
    if (bullet != null || numbered != null) {
      endParagraph();
      endQuote();
      final indent = (bullet?.group(1) ?? numbered!.group(1)!).length;
      items.add(
        AnswerListItem(
          marker: bullet != null ? '•' : '${numbered!.group(2)}.',
          text: bullet?.group(2) ?? numbered!.group(3)!,
          depth: indent ~/ 2,
        ),
      );
      i++;
      continue;
    }
    if (items.isNotEmpty && line.startsWith(' ')) {
      // The wrapped second line of an item.
      final last = items.removeLast();
      items.add(last.copyWith(text: '${last.text} $trimmed'));
      i++;
      continue;
    }
    endList();
    endQuote();
    paragraph.add(trimmed);
    i++;
  }
  endAll();
  return blocks;
}
