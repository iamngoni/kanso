// Kanso — answer_parsing_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show AnyUtils, GeneratorUtils, Glados, ListAnys, any;
import 'package:kanso/ask/models/data/answer_block.dart';
import 'package:kanso/ask/models/data/answer_span.dart';
import 'package:kanso/ask/utils/note_citation.dart';
import 'package:kanso/ask/utils/parse_answer_blocks.dart';
import 'package:kanso/ask/utils/parse_answer_inline.dart';

import '../../helpers/property.dart';

const _idAlphabet = <String>[
  'a',
  'Z',
  '7',
  ':',
  '/',
  '(',
  ')',
  '%',
  ' ',
  '#',
  'é',
  '[',
  ']',
  '-',
  '_',
];

/// Strings of the characters a note id, or a hostile one, may have.
final _ids = any
    .listWithLengthInRange<String>(
      1,
      12,
      any.choose(_idAlphabet),
    )
    .map((chars) => chars.join());

const _hostile = <String>[
  '![',
  ']',
  '(',
  ')',
  '<',
  '>',
  'script',
  'javascript:',
  'data:',
  'http://x.y',
  'kanso://open/note/',
  '*',
  '`',
  '[',
  'a',
  ' ',
  '\n',
  '#',
  '-',
];

final _hostileText = any
    .listWithLengthInRange<String>(
      0,
      16,
      any.choose(_hostile),
    )
    .map((parts) => parts.join());

void main() {
  group('note citations', () {
    Glados<String>(_ids, kansoExploreConfig()).test(
      'the link of an id gives the id back',
      (id) {
        expect(noteIdOfCitation(noteCitationHref(id)), id);
      },
    );

    Glados<String>(_ids, kansoExploreConfig()).test(
      'a citation survives being written in a Markdown link',
      (id) {
        final spans = parseAnswerInline(
          '[T](${noteCitationHref(id)})',
          usedNoteIds: <String>{id},
        );
        expect(spans.single, AnswerSpan.note(text: 'T', noteId: id));
      },
    );

    test('the core’s own form of a citation is read too', () {
      expect(noteIdOfCitation('kanso://note/n:1'), 'n:1');
    });

    test('a model link opens only a note Rust reported as used', () {
      expect(
        parseAnswerInline(
          '[Secret](kanso://note/secret)',
          usedNoteIds: <String>{'reported'},
        ),
        <AnswerSpan>[const AnswerSpan.text('Secret')],
      );
    });

    test('only the app’s own note links are citations', () {
      expect(noteIdOfCitation('https://example.com/note/n1'), isNull);
      expect(noteIdOfCitation('kanso://open/notebook/n1'), isNull);
      expect(noteIdOfCitation(noteCitationPrefix), isNull);
      expect(noteIdOfCitation('$noteCitationPrefix%E0%A4%A'), isNull);
    });
  });

  group('parseAnswerInline', () {
    test('reads marks, citations and web links', () {
      expect(
        parseAnswerInline(
          // Keep the closing bracket adjacent to the opening parenthesis.
          // ignore: missing_whitespace_between_adjacent_strings
          'It is **40**, *per month*, `x`; see [Pricing]'
          '(kanso://open/note/n1) or [docs](https://example.com/a).',
          usedNoteIds: <String>{'n1'},
        ),
        <AnswerSpan>[
          const AnswerSpan.text('It is '),
          const AnswerSpan.text('40', bold: true),
          const AnswerSpan.text(', '),
          const AnswerSpan.text('per month', italic: true),
          const AnswerSpan.text(', '),
          const AnswerSpan.text('x', code: true),
          const AnswerSpan.text('; see '),
          const AnswerSpan.note(text: 'Pricing', noteId: 'n1'),
          const AnswerSpan.text(' or '),
          const AnswerSpan.web(text: 'docs', url: 'https://example.com/a'),
          const AnswerSpan.text('.'),
        ],
      );
    });

    test('an image is its description and never its address', () {
      final spans = parseAnswerInline('A ![chart](https://x.test/c.png) B');
      expect(spans, <AnswerSpan>[const AnswerSpan.text('A chart B')]);
    });

    test('HTML is the characters it is made of', () {
      expect(
        parseAnswerInline('<img src=x onerror=alert(1)>'),
        <AnswerSpan>[const AnswerSpan.text('<img src=x onerror=alert(1)>')],
      );
    });

    test('a link to anything else is plain text', () {
      for (final address in <String>[
        'javascript:alert(1)',
        'file:///etc/passwd',
        'data:text/html,x',
        'tel:123',
        'https://',
      ]) {
        expect(
          parseAnswerInline('[go]($address)'),
          everyElement(isA<AnswerText>()),
          reason: address,
        );
      }
    });

    test('a half-written mark is plain text', () {
      expect(parseAnswerInline('**40 and [Pri'), <AnswerSpan>[
        const AnswerSpan.text('**40 and [Pri'),
      ]);
    });

    Glados<String>(_hostileText, kansoExploreConfig()).test(
      'for any text: no image, and links only to notes or web pages',
      (source) {
        final spans = parseAnswerInline(source);
        for (final span in spans) {
          switch (span) {
            case AnswerText(:final text):
              expect(text, isNotEmpty);
              // An image's address is gone with its mark.
              expect(RegExp(r'!\[[^\]]*\]\([^)]*\)').hasMatch(text), isFalse);
            case AnswerNoteLink(:final noteId):
              expect(noteId, isNotEmpty);
            case AnswerWebLink(:final url):
              final uri = Uri.parse(url);
              expect(<String>['http', 'https'], contains(uri.scheme));
              expect(uri.host, isNotEmpty);
          }
        }
      },
    );
  });

  group('parseAnswerBlocks', () {
    test('cuts headings, paragraphs, lists, code, quotes and rules', () {
      final blocks = parseAnswerBlocks(
        '# Plan\n\nIt costs\n40 a month.\n\n- one\n- two\n  wrapped\n'
        '1. first\n2. second\n\n> quoted\n\n---\n\n```\ncode(1)\n  x\n```\n',
      );
      expect(blocks, <AnswerBlock>[
        const AnswerBlock.heading(level: 1, text: 'Plan'),
        const AnswerBlock.paragraph('It costs 40 a month.'),
        const AnswerBlock.list(<AnswerListItem>[
          AnswerListItem(marker: '•', text: 'one'),
          AnswerListItem(marker: '•', text: 'two wrapped'),
          AnswerListItem(marker: '1.', text: 'first'),
          AnswerListItem(marker: '2.', text: 'second'),
        ]),
        const AnswerBlock.quote('quoted'),
        const AnswerBlock.rule(),
        const AnswerBlock.code('code(1)\n  x'),
      ]);
    });

    test('nests a list by its indent', () {
      final list =
          parseAnswerBlocks('- a\n  - b\n    - c').single as AnswerList;
      expect(list.items.map((i) => i.depth), <int>[0, 1, 2]);
    });

    test('code that is not closed runs to the end', () {
      expect(parseAnswerBlocks('```\nabc\ndef'), <AnswerBlock>[
        const AnswerBlock.code('abc\ndef'),
      ]);
    });

    Glados<String>(_hostileText, kansoExploreConfig()).test(
      'for any text: no text is invented',
      (source) {
        String squash(String text) => text.replaceAll(RegExp(r'\s'), '');
        final blocks = parseAnswerBlocks(source);
        final kept = StringBuffer();
        for (final block in blocks) {
          switch (block) {
            case AnswerHeading(:final text):
              kept.write(text);
            case AnswerParagraph(:final text) || AnswerQuote(:final text):
              kept.write(text);
            case AnswerList(:final items):
              for (final item in items) {
                kept.write(item.text);
              }
            case AnswerCode(:final text):
              kept.write(text);
            case AnswerRule():
              break;
          }
        }
        // Fences, rules, quote marks and list marks are the only losses.
        final lost = squash(source).length - squash(kept.toString()).length;
        expect(lost, greaterThanOrEqualTo(0));
      },
    );
  });
}
