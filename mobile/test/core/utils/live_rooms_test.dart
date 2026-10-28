// Kanso — live_rooms_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, Generator, GeneratorUtils, Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/utils/live_rooms.dart';

import '../../helpers/property.dart';

/// A room with no count of its own, like the library's: it records what was
/// asked of it and refuses a second enter or a leave of a room not entered.
class _Rooms {
  final Set<String> open = <String>{};
  final List<String> calls = <String>[];

  /// Notes whose next enter fails.
  final Set<String> failing = <String>{};

  /// How long each enter and leave takes, in turns of the event loop.
  int turns = 0;

  Future<void> _wait() async {
    for (var turn = 0; turn < turns; turn++) {
      await Future<void>.value();
    }
  }

  Future<void> enter(String noteId) async {
    await _wait();
    if (failing.remove(noteId)) {
      throw StateError('no room for $noteId');
    }
    if (!open.add(noteId)) {
      throw StateError('$noteId entered twice');
    }
    calls.add('open $noteId');
  }

  Future<void> leave(String noteId) async {
    await _wait();
    if (!open.remove(noteId)) {
      throw StateError('$noteId left while closed');
    }
    calls.add('close $noteId');
  }

  LiveRooms rooms() => LiveRooms(enter, leave);
}

/// A take or a let-go of one of three notes by one of three holders.
typedef _Step = ({bool take, int note, int holder});

extension on Any {
  Generator<List<_Step>> get steps =>
      listWithLengthInRange<int>(
        0,
        24,
        intInRange(0, 18),
      ).map(
        (codes) => <_Step>[
          for (final code in codes)
            (take: code.isEven, note: code ~/ 2 % 3, holder: code ~/ 6),
        ],
      );
}

void main() {
  group('LiveRooms', () {
    test('enters a room for its first holder and leaves it after its '
        'last', () async {
      final fake = _Rooms();
      final rooms = fake.rooms();
      final a = Object();
      final b = Object();
      await rooms.take('n', a);
      await rooms.take('n', b);
      await rooms.letGo('n', a);
      expect(fake.open, <String>{'n'});
      await rooms.letGo('n', b);
      expect(fake.open, isEmpty);
      expect(fake.calls, <String>['open n', 'close n']);
    });

    test('a holder taking a room twice holds it once', () async {
      final fake = _Rooms();
      final rooms = fake.rooms();
      final a = Object();
      await rooms.take('n', a);
      await rooms.take('n', a);
      await rooms.letGo('n', a);
      expect(fake.open, isEmpty);
    });

    test('letting go of a room not held does nothing', () async {
      final fake = _Rooms();
      final rooms = fake.rooms();
      await rooms.letGo('n', Object());
      expect(fake.calls, isEmpty);
    });

    test('a take that fails holds nothing, and the next one enters', () async {
      final fake = _Rooms()..failing.add('n');
      final rooms = fake.rooms();
      final a = Object();
      await expectLater(rooms.take('n', a), throwsStateError);
      expect(rooms.isOpen('n'), isFalse);
      await rooms.take('n', a);
      expect(fake.open, <String>{'n'});
    });

    test('a take that comes while the last holder is leaving finds the '
        'room open', () async {
      // The bug this guards: B's enter ran while A's leave was on its way,
      // so the room closed under B, who believed it was in.
      final fake = _Rooms()..turns = 3;
      final rooms = fake.rooms();
      final a = Object();
      final b = Object();
      await rooms.take('n', a);
      final leaving = rooms.letGo('n', a);
      final taking = rooms.take('n', b);
      await Future.wait(<Future<void>>[leaving, taking]);
      expect(fake.open, <String>{'n'});
      expect(rooms.isOpen('n'), isTrue);
    });

    Glados<List<_Step>>(any.steps, kansoExploreConfig()).test(
      'whatever order takes and let-gos come in, at once, a room is open '
      'exactly while somebody holds it',
      (steps) async {
        final fake = _Rooms()..turns = 2;
        final rooms = fake.rooms();
        final holders = <Object>[Object(), Object(), Object()];
        final model = <String, Set<int>>{};
        final all = <Future<void>>[];
        for (final step in steps) {
          final note = 'n${step.note}';
          if (step.take) {
            model.putIfAbsent(note, () => <int>{}).add(step.holder);
            all.add(rooms.take(note, holders[step.holder]));
          } else {
            model[note]?.remove(step.holder);
            all.add(rooms.letGo(note, holders[step.holder]));
          }
        }
        // None of them is awaited before the next is asked for.
        await Future.wait(all);
        final held = <String>{
          for (final entry in model.entries)
            if (entry.value.isNotEmpty) entry.key,
        };
        expect(fake.open, held);
        for (final note in <String>['n0', 'n1', 'n2']) {
          expect(rooms.isOpen(note), held.contains(note), reason: note);
        }
      },
    );
  });
}
