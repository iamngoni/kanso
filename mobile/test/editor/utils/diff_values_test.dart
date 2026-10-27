// Kanso — diff_values_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        Generator,
        GeneratorUtils,
        Glados2,
        Glados3,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/editor/models/data/text_diff.dart';
import 'package:kanso/editor/utils/diff_values.dart';

import '../../helpers/property.dart';

extension _Texts on Any {
  /// Short texts of a few letters, a line break and a pair of surrogates
  /// that share a half with another pair, so changes land on repeats and
  /// next to pairs.
  Generator<String> get edited => listWithLengthInRange<String>(
    0,
    10,
    choose(<String>['a', 'b', '\n', '\u{1F600}', '\u{1F601}']),
  ).map((chars) => chars.join());

  /// Texts with no surrogates.
  Generator<String> get plain => listWithLengthInRange<String>(
    0,
    10,
    choose(<String>['a', 'b', 'c', ' ']),
  ).map((chars) => chars.join());
}

TextEditingValue _value(String text, [int? caret]) => TextEditingValue(
  text: text,
  selection: caret == null
      ? const TextSelection.collapsed(offset: -1)
      : TextSelection.collapsed(offset: caret),
);

String _apply(String text, TextDiff diff) =>
    text.replaceRange(diff.start, diff.end, diff.text);

void main() {
  group('diffValues', () {
    test('says nothing changed for the same text', () {
      expect(diffValues(_value('same'), _value('same')).isEmpty, isTrue);
      expect(diffValues(_value(''), _value('')).isEmpty, isTrue);
    });

    test('finds a character typed, deleted and replaced', () {
      expect(
        diffValues(_value('ab'), _value('aXb')),
        const TextDiff(start: 1, end: 1, text: 'X'),
      );
      expect(
        diffValues(_value('aXb'), _value('ab')),
        const TextDiff(start: 1, end: 2, text: ''),
      );
      expect(
        diffValues(_value('aXb'), _value('aYb')),
        const TextDiff(start: 1, end: 2, text: 'Y'),
      );
      expect(
        diffValues(_value(''), _value('hi')),
        const TextDiff(start: 0, end: 0, text: 'hi'),
      );
    });

    test('puts an inserted repeat where the caret says', () {
      // An "a" typed between two: without the caret it could be anywhere
      // in the run; the caret after it says where.
      expect(
        diffValues(_value('aaa'), _value('aaaa', 2)),
        const TextDiff(start: 1, end: 1, text: 'a'),
      );
      expect(
        diffValues(_value('aaa'), _value('aaaa', 4)),
        const TextDiff(start: 3, end: 3, text: 'a'),
      );
    });

    test('keeps a surrogate pair whole', () {
      // The pairs share their first half.
      expect(
        diffValues(_value('\u{1F600}'), _value('\u{1F601}')),
        const TextDiff(start: 0, end: 2, text: '\u{1F601}'),
      );
      expect(
        diffValues(_value('a'), _value('a\u{1F600}')),
        const TextDiff(start: 1, end: 1, text: '\u{1F600}'),
      );
    });

    Glados2<String, String>(any.edited, any.edited, kansoExploreConfig()).test(
      'the change turns the old text into the new',
      (before, after) {
        final diff = diffValues(_value(before), _value(after));
        expect(_apply(before, diff), after);
        expect(diff.start, lessThanOrEqualTo(diff.end));
        expect(diff.end, lessThanOrEqualTo(before.length));
        if (before == after) {
          expect(diff.isEmpty, isTrue);
        }
      },
    );

    Glados3<String, String, int>(
      any.edited,
      any.edited,
      any.intInRange(0, 12),
    ).test(
      'is as good with a caret, and never cuts a surrogate pair',
      (before, after, at) {
        final caret = at.clamp(0, after.length);
        final diff = diffValues(_value(before), _value(after, caret));
        expect(_apply(before, diff), after);
        expect(diff.start, lessThanOrEqualTo(diff.end));
        expect(splitsPair(before, diff.start), isFalse);
        expect(splitsPair(before, diff.end), isFalse);
        expect(splitsPair(after, diff.start), isFalse);
        expect(
          splitsPair(after, diff.start + diff.text.length),
          isFalse,
        );
      },
    );

    Glados2<String, String>(any.plain, any.plain, kansoExploreConfig()).test(
      'is the smallest change when there is no caret',
      (before, after) {
        final diff = diffValues(_value(before), _value(after));
        var prefix = 0;
        while (prefix < before.length &&
            prefix < after.length &&
            before[prefix] == after[prefix]) {
          prefix++;
        }
        expect(diff.start, prefix);
        // Nothing of the end is left that could be shared.
        final tailOld = before.substring(diff.end);
        final tailNew = after.substring(diff.start + diff.text.length);
        expect(tailOld, tailNew);
        if (diff.end > diff.start && diff.text.isNotEmpty) {
          expect(
            before[diff.end - 1],
            isNot(after[diff.start + diff.text.length - 1]),
          );
        }
      },
    );

    Glados3<String, String, int>(
      any.plain,
      any.plain,
      any.intInRange(0, 10),
    ).test(
      'a character typed ends at the caret',
      (before, typed, at) {
        if (typed.isEmpty) {
          return;
        }
        final position = at.clamp(0, before.length);
        final after = before.replaceRange(position, position, typed);
        final diff = diffValues(
          _value(before),
          _value(after, position + typed.length),
        );
        expect(diff.start + diff.text.length, position + typed.length);
        expect(_apply(before, diff), after);
      },
    );
  });
}
