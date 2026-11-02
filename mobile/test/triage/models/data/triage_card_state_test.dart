// Kanso — triage_card_state_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:glados/glados.dart';
import 'package:kanso/core/bridge/api/triage_types.dart';
import 'package:kanso/triage/models/data/triage_card_state.dart';

TriageChip chip(TriageField field, String value) => TriageChip(
  field: field,
  value: value,
  label: value,
  score: 1,
  because: const <TriageNote>[],
  ai: false,
  isNew: false,
);

TriageCardState card(List<TriageChip> chips) => TriageCardState(
  noteId: 'n',
  rev: 'r',
  title: 't',
  snippet: '',
  updatedMs: 0,
  chips: <TriageChipState>[for (final c in chips) TriageChipState(chip: c)],
);

void main() {
  group('new chips on a card', () {
    test('accepted and edited chips stay, untouched ones are replaced', () {
      var c = card(<TriageChip>[
        chip(TriageField.tag, 't1'),
        chip(TriageField.tag, 't2'),
        chip(TriageField.tag, 't3'),
      ]).toggleAccept(0).edit(1, chip(TriageField.tag, 'typed'));
      c = c.toggleReject(2);
      final next = c.withChips(<TriageChip>[
        chip(TriageField.tag, 't3'),
        chip(TriageField.tag, 'ai1'),
        chip(TriageField.tag, 't1'),
      ]);
      expect(
        next.chips.map((x) => (x.chip.value, x.mark, x.edited)).toList(),
        <(String, ChipMark, bool)>[
          ('t1', ChipMark.accepted, false),
          ('typed', ChipMark.accepted, true),
          ('t3', ChipMark.rejected, false),
          ('ai1', ChipMark.untouched, false),
        ],
      );
    });

    test('chips never move, so the one in hand stays in hand', () {
      final c = card(<TriageChip>[
        chip(TriageField.tag, 'a'),
        chip(TriageField.tag, 'b'),
      ]).toggleAccept(1);
      final next = c.withChips(<TriageChip>[chip(TriageField.tag, 'z')]);
      expect(next.chips.first.chip.value, 'b');
      expect(next.chips.first.mark, ChipMark.accepted);
    });
  });

  group('the marks of a card', () {
    test('accepting a notebook lifts the one accepted before', () {
      final c = card(<TriageChip>[
        chip(TriageField.notebook, 'A'),
        chip(TriageField.notebook, 'B'),
      ]).toggleAccept(0).toggleAccept(1);
      expect(c.accepted.map((x) => x.value), <String>['B']);
    });

    test('tags can be accepted together', () {
      final c = card(<TriageChip>[
        chip(TriageField.tag, 'a'),
        chip(TriageField.tag, 'b'),
      ]).toggleAccept(0).toggleAccept(1);
      expect(c.accepted, hasLength(2));
    });

    test('a rejected chip is never filed, even after Accept all', () {
      final c = card(<TriageChip>[
        chip(TriageField.tag, 'a'),
        chip(TriageField.tag, 'b'),
      ]).toggleReject(0).acceptAll();
      expect(c.accepted.map((x) => x.value), <String>['b']);
    });

    test('a card whose note changed has nothing to file', () {
      final c = card(<TriageChip>[
        chip(TriageField.tag, 'a'),
      ]).toggleAccept(0).copyWith(changed: true);
      expect(c.filing, isNull);
    });

    test('an index out of range changes nothing', () {
      final c = card(<TriageChip>[chip(TriageField.tag, 'a')]);
      expect(c.toggleAccept(5), c);
      expect(c.toggleReject(-1), c);
    });

    Glados<List<int>>(any.list(any.intInRange(0, 6))).test(
      'whatever is tapped, a notebook is accepted at most once and a '
      'rejected chip is never accepted',
      (taps) {
        var c = card(<TriageChip>[
          chip(TriageField.notebook, 'A'),
          chip(TriageField.notebook, 'B'),
          chip(TriageField.tag, 'a'),
          chip(TriageField.tag, 'b'),
          chip(TriageField.title, 'T'),
        ]);
        for (final t in taps) {
          c = t.isEven ? c.toggleAccept(t ~/ 2) : c.toggleReject(t ~/ 2);
        }
        final books = c.accepted.where((x) => x.field == TriageField.notebook);
        expect(books.length, lessThanOrEqualTo(1));
        for (final s in c.chips) {
          if (s.mark == ChipMark.rejected) {
            expect(c.accepted, isNot(contains(s.chip)));
          }
        }
        expect(
          c.acceptAll().accepted.where((x) => x.field == TriageField.title),
          hasLength(c.chips[4].mark == ChipMark.rejected ? 0 : 1),
        );
      },
    );
  });
}
