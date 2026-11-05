// Kanso — notices_cubit_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/notices_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/notices/state/notices_cubit.dart';

import '../../helpers/fixture_notices_repository.dart';

PhoneNotice _notice(String id) => PhoneNotice(
  id: id,
  title: 'Notice $id',
  whatChanged: 'Something changed.',
  steps: const <String>['Do this.'],
  actionKind: PhoneNoticeActionKind.none,
  actionLabel: '',
  actionTarget: '',
);

final List<PhoneNotice> _three = <PhoneNotice>[
  _notice('a'),
  _notice('b'),
  _notice('c'),
];

void main() {
  test(
    'starts with no card, then shows the newest notice that applies',
    () async {
      final repository = FixtureNoticesRepository(applying: _three);
      final cubit = NoticesCubit(repository);
      expect(cubit.state, isNull);

      await cubit.refresh(signedIn: true);
      expect(cubit.state?.card?.id, 'a');
      expect(cubit.state?.total, 3);
      expect(cubit.state?.countLabel, '1 of 3');
      expect(repository.calls, <String>['notices:in']);
      await cubit.close();
    },
  );

  test('shows nothing when nothing applies', () async {
    final cubit = NoticesCubit(FixtureNoticesRepository(applying: const []));
    await cubit.refresh(signedIn: false);
    expect(cubit.state, isNull);
    await cubit.close();
  });

  test('dismissing hides the card and moves on to the next', () async {
    final repository = FixtureNoticesRepository(applying: _three);
    final cubit = NoticesCubit(repository);
    await cubit.refresh(signedIn: false);
    await cubit.dismiss('a', signedIn: false);
    expect(cubit.state?.card?.id, 'b');
    expect(cubit.state?.total, 2);
    await cubit.markFixed('b', signedIn: false);
    await cubit.dismiss('c', signedIn: false);
    expect(cubit.state, isNull);
    expect(repository.dismissed, <String>{'a', 'c'});
    expect(repository.fixed, <String>{'b'});
    await cubit.close();
  });

  test('a failure shows no card instead of an error', () async {
    final repository = FixtureNoticesRepository(applying: _three)
      ..failure = const ApplicationError.unexpected(message: 'broken');
    final cubit = NoticesCubit(repository);
    await cubit.refresh(signedIn: false);
    expect(cubit.state, isNull);
    await cubit.close();
  });

  test('a cubit closed while the library answers does not emit', () async {
    final cubit = NoticesCubit(FixtureNoticesRepository(applying: _three));
    final pending = cubit.refresh(signedIn: false);
    await cubit.close();
    await pending;
    expect(cubit.state, isNull);
  });

  // Whatever is dismissed or marked fixed, in any order and any number of
  // times, the card never shows a notice that was put away, and doing the
  // same thing again changes nothing.
  Glados<List<int>>(any.list(any.intInRange(0, 6))).test(
    'a notice put away never comes back and putting it away twice is the '
    'same as once',
    (ops) async {
      final repository = FixtureNoticesRepository(applying: _three);
      final cubit = NoticesCubit(repository);
      await cubit.refresh(signedIn: false);
      for (final op in ops) {
        final id = _three[op % 3].id;
        if (op < 3) {
          await cubit.dismiss(id, signedIn: false);
        } else {
          await cubit.markFixed(id, signedIn: false);
        }
        final shown = cubit.state?.card?.id;
        expect(repository.dismissed.contains(shown), isFalse);
        expect(repository.fixed.contains(shown), isFalse);
      }
      final before = cubit.state;
      for (final id in repository.dismissed) {
        await cubit.dismiss(id, signedIn: false);
      }
      expect(cubit.state, before);
      expect(
        cubit.state?.total ?? 0,
        _three.length - repository.dismissed.union(repository.fixed).length,
      );
      await cubit.close();
    },
  );
}
