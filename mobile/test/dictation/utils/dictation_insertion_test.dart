// Kanso — dictation_insertion_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        Generator,
        GeneratorUtils,
        Glados3,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/dictation/models/data/dictated_text.dart';
import 'package:kanso/dictation/models/data/dictation_edit.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_text_rules.dart';
import 'package:kanso/dictation/models/repos/impl/plain_dictation_text_rules.dart';
import 'package:kanso/dictation/utils/dictation_insertion.dart';

import '../../helpers/property.dart';
import '../helpers/fake_dictation_text_rules.dart';

TextEditingValue _at(String text, int offset) => TextEditingValue(
  text: text,
  selection: TextSelection.collapsed(offset: offset),
);

TextEditingValue _selecting(String text, int start, int end) =>
    TextEditingValue(
      text: text,
      selection: TextSelection(baseOffset: start, extentOffset: end),
    );

/// Makes [edit] to [value] as the editor's `replace` does: the caret ends
/// after the new text.
TextEditingValue _made(TextEditingValue value, DictationEdit edit) => _at(
  DictationInsertion.apply(value.text, edit),
  edit.range.start + edit.text.length,
);

class _AfterRules implements DictationTextRules {
  String? after;

  @override
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  }) async {
    this.after = after;
    return DictatedText(text: transcript, caretUtf16: transcript.length);
  }
}

/// Says [phrases] one after the other into [start], as a dictation does, and
/// returns the note after each.
Future<List<TextEditingValue>> _dictate(
  TextEditingValue start,
  List<String> phrases,
  DictationTextRules rules,
) async {
  final insertion = DictationInsertion(rules: rules);
  final steps = <TextEditingValue>[];
  var value = start;
  for (var i = 0; i < phrases.length; i++) {
    final edit = await insertion.next(value, phrases.sublist(0, i + 1));
    if (edit != null) {
      value = _made(value, edit);
    }
    steps.add(value);
  }
  return steps;
}

extension _Inputs on Any {
  /// Text of letters, spaces, line breaks, brackets, a full stop and a
  /// Chinese character: the edges that spacing cares about.
  Generator<String> get spaced => listWithLengthInRange<String>(
    0,
    14,
    choose(<String>['a', 'b', ' ', '\n', '(', ')', '.', '漢', 'c']),
  ).map((chars) => chars.join());

  /// A phrase: what the recognizer says, with or without spaces around it.
  Generator<String> get phrase => listWithLengthInRange<String>(
    0,
    8,
    choose(<String>['hi', ' ', 'there', '.', '你好', ',', 'ok']),
  ).map((words) => words.join());
}

void main() {
  test(
    'passes one whole Unicode scalar after a UTF-16 insertion offset',
    () async {
      final rules = _AfterRules();
      await DictationInsertion(rules: rules).next(
        _at('a😀b', 1),
        <String>['hello'],
      );
      expect(rules.after, '😀');
    },
  );

  final plain = PlainDictationTextRules();

  group('with the plain rules', () {
    test(
      'the first phrase goes at the caret with a space after a word',
      () async {
        final edit = await DictationInsertion(
          rules: plain,
        ).next(_at('Buy', 3), <String>['milk']);
        expect(edit!.range, const TextRange.collapsed(3));
        expect(edit.text, ' milk');
      },
    );

    test('takes no space at the start of the text or of a line', () async {
      Future<String?> first(String text) async => (await DictationInsertion(
        rules: plain,
      ).next(_at(text, text.length), <String>['milk']))?.text;
      expect(await first(''), 'milk');
      expect(await first('a\n'), 'milk');
      expect(await first('a '), 'milk');
      expect(await first('('), 'milk');
    });

    test('takes no space between Chinese characters', () async {
      final edit = await DictationInsertion(
        rules: plain,
      ).next(_at('我', 1), <String>['你好']);
      expect(edit!.text, '你好');
    });

    test('replaces the selection', () async {
      final value = _selecting('Buy bread now', 4, 9);
      final edit = await DictationInsertion(
        rules: plain,
      ).next(value, <String>['milk']);
      expect(edit!.range, const TextRange(start: 4, end: 9));
      expect(DictationInsertion.apply(value.text, edit), 'Buy milk now');
    });

    test('keeps a word that follows it apart', () async {
      final value = _at('Buy now', 3);
      final edit = await DictationInsertion(
        rules: plain,
      ).next(value, <String>['milk']);
      expect(DictationInsertion.apply(value.text, edit!), 'Buy milk now');
    });

    test('goes to the end when the editor has no selection', () async {
      const value = TextEditingValue(text: 'Buy');
      final edit = await DictationInsertion(
        rules: plain,
      ).next(value, <String>['milk']);
      expect(edit!.range, const TextRange.collapsed(3));
    });

    test('changes nothing while nothing is said', () async {
      final value = _selecting('keep this', 0, 4);
      final insertion = DictationInsertion(rules: plain);
      expect(await insertion.next(value, <String>[]), isNull);
      expect(await insertion.next(value, <String>[' \n ']), isNull);
    });

    test('the phrases follow one another', () async {
      final steps = await _dictate(_at('Notes:', 6), <String>[
        'Buy milk.',
        'Call Ana.',
      ], plain);
      expect(steps.last.text, 'Notes: Buy milk. Call Ana.');
      expect(steps.last.selection.baseOffset, steps.last.text.length);
    });

    test('preview says what would land, and changes nothing', () async {
      final insertion = DictationInsertion(rules: plain);
      final value = _at('Buy', 3);
      expect(
        await insertion.preview(value, <String>[], 'milk'),
        ' milk',
      );
      expect(insertion.range, isNull);
      expect(await insertion.preview(value, <String>[], ''), isEmpty);
    });
  });

  group('with rules that act on commands', () {
    final commands = CommandRules();

    test('a command rewrites only the words of this dictation', () async {
      final steps = await _dictate(_at('Earlier words.', 14), <String>[
        'buy milk',
        'comma eggs',
        'delete that',
      ], commands);
      expect(steps[0].text, 'Earlier words. buy milk');
      expect(steps[1].text, 'Earlier words. buy milk, eggs');
      // "delete that" takes back "eggs", and goes no further than the
      // dictation: the note's own words stay.
      expect(steps[2].text, 'Earlier words. buy milk,');
    });

    test('a delete that takes back everything said, never the note', () async {
      final steps = await _dictate(_at('Keep me.', 8), <String>[
        'one',
        'delete that',
        'delete that',
      ], commands);
      expect(steps[1].text, 'Keep me.');
      expect(steps[2].text, 'Keep me.');
    });

    test('words after a lost range start a new range at the caret', () async {
      final insertion = DictationInsertion(rules: commands);
      var value = _at('Notes:', 6);
      var edit = await insertion.next(value, <String>['one']);
      value = _made(value, edit!);
      // The person typed above the dictated words.
      value = _at('New\n${value.text}', 4 + 10);
      edit = await insertion.next(value, <String>['one', 'two']);
      expect(edit!.text, ' two');
      final after = DictationInsertion.apply(value.text, edit);
      expect(after, startsWith('New\nNotes: one'));
      expect(after, endsWith('two'));
    });
  });

  group('properties', () {
    Glados3<String, int, String>(
      any.spaced,
      any.intInRange(0, 15),
      any.phrase,
      kansoExploreConfig(),
    ).test('text outside the replaced range is never changed', (
      text,
      offset,
      phrase,
    ) async {
      final value = _at(text, offset.clamp(0, text.length));
      final edit = await DictationInsertion(
        rules: plain,
      ).next(value, <String>[phrase]);
      if (edit == null) {
        return;
      }
      final after = DictationInsertion.apply(text, edit);
      expect(
        after.substring(0, edit.range.start),
        text.substring(0, edit.range.start),
      );
      expect(
        after.substring(edit.range.start + edit.text.length),
        text.substring(edit.range.end),
      );
    });

    Glados3<String, int, String>(
      any.spaced,
      any.intInRange(0, 15),
      any.phrase,
      kansoExploreConfig(),
    ).test('the phrase lands trimmed, with spacing only at its edges', (
      text,
      offset,
      phrase,
    ) async {
      final value = _at(text, offset.clamp(0, text.length));
      final edit = await DictationInsertion(
        rules: plain,
      ).next(value, <String>[phrase]);
      if (edit == null) {
        expect(phrase.trim(), isEmpty);
        return;
      }
      expect(edit.text.trim(), phrase.trim());
      expect(
        edit.text.length - edit.text.trimLeft().length,
        lessThanOrEqualTo(1),
      );
      expect(
        edit.text.length - edit.text.trimRight().length,
        lessThanOrEqualTo(1),
      );
    });

    Glados3<String, List<String>, int>(
      any.spaced,
      any.listWithLengthInRange<String>(0, 5, any.phrase),
      any.intInRange(0, 15),
      kansoExploreConfig(),
    ).test('a whole dictation leaves the text around the selection alone', (
      text,
      phrases,
      offset,
    ) async {
      final start = offset.clamp(0, text.length);
      final end = (start + offset % 4).clamp(0, text.length);
      final steps = await _dictate(
        _selecting(text, start, end),
        phrases,
        plain,
      );
      final last = steps.isEmpty ? _selecting(text, start, end) : steps.last;
      expect(last.text.startsWith(text.substring(0, start)), isTrue);
      final saidAnything = phrases.any((p) => p.trim().isNotEmpty);
      final tail = text.substring(saidAnything ? end : start);
      expect(last.text.endsWith(tail), isTrue);
      if (!saidAnything) {
        expect(last.text, text);
      }
    });

    Glados3<String, int, List<String>>(
      any.spaced,
      any.intInRange(0, 15),
      any.listWithLengthInRange<String>(1, 4, any.phrase),
      kansoExploreConfig(),
    ).test('every update replaces only what the dictation put in', (
      text,
      offset,
      phrases,
    ) async {
      var value = _at(text, offset.clamp(0, text.length));
      final insertion = DictationInsertion(rules: CommandRules());
      for (var i = 0; i < phrases.length; i++) {
        final before = value.text;
        final range = insertion.range;
        final edit = await insertion.next(value, phrases.sublist(0, i + 1));
        if (edit == null) {
          continue;
        }
        expect(edit.range.start, inInclusiveRange(0, before.length));
        expect(
          edit.range.end,
          inInclusiveRange(edit.range.start, before.length),
        );
        if (range != null) {
          // After the first, the edit is exactly the last range.
          expect(edit.range, range);
        }
        value = _made(value, edit);
      }
    });
  });
}
