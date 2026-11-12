// Kanso — whats_new_test.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/whats_new_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/whats_new_repository.dart';
import 'package:kanso/whats_new/state/whats_new_cubit.dart';
import 'package:kanso/whats_new/views/widgets/whats_new_sheet.dart';

import '../helpers/pump.dart';

PhoneWhatsNew _whatsNew({int skipped = 0, String videoPath = ''}) =>
    PhoneWhatsNew(
      version: '0.10.7',
      date: '2026-11-12',
      summary: '1 improvement and 1 fix.',
      videoPath: videoPath,
      sections: const <PhoneWhatsNewSection>[
        PhoneWhatsNewSection(title: 'Improved', items: <String>['Faster']),
        PhoneWhatsNewSection(title: 'Fixed', items: <String>['A crash']),
      ],
      earlier: <PhoneWhatsNewEarlier>[
        for (var i = 0; i < skipped; i++)
          PhoneWhatsNewEarlier(
            version: '0.10.${6 - i}',
            sections: <PhoneWhatsNewSection>[
              PhoneWhatsNewSection(
                title: 'Fixed',
                items: <String>['Fixed in 0.10.${6 - i}'],
              ),
            ],
          ),
      ],
      releaseUrl: 'https://github.com/iamngoni/kanso/releases/tag/v0.10.7',
    );

/// The sheet the library would hand over, in memory; every call recorded.
class _Repository implements WhatsNewRepository {
  _Repository(this.answer);

  PhoneWhatsNew? answer;
  final List<String> calls = <String>[];

  @override
  Future<Either<ApplicationError, PhoneWhatsNew?>> whatsNew() async {
    calls.add('whatsNew');
    return right(answer);
  }

  @override
  Future<Either<ApplicationError, Unit>> markSeen() async {
    calls.add('markSeen');
    answer = null;
    return right(unit);
  }
}

void main() {
  test('asks once, then puts the sheet away when seen', () async {
    final repository = _Repository(_whatsNew());
    final cubit = WhatsNewCubit(repository);
    expect(cubit.state, isNull);
    await cubit.load();
    expect(cubit.state?.version, '0.10.7');
    await cubit.load();
    expect(repository.calls, <String>['whatsNew']);
    await cubit.seen();
    expect(cubit.state, isNull);
    expect(repository.calls, <String>['whatsNew', 'markSeen']);
    await cubit.close();
  });

  test('nothing to show stays empty', () async {
    final cubit = WhatsNewCubit(_Repository(null));
    await cubit.load();
    expect(cubit.state, isNull);
    await cubit.close();
  });

  testWidgets('without a video the notes lead', (tester) async {
    await pumpKanso(tester, WhatsNewSheet(whatsNew: _whatsNew()));
    expect(find.byKey(const ValueKey<String>('whats-new-video')), findsNothing);
    expect(find.text('Kanso 0.10.7'), findsOneWidget);
    expect(find.text('v0.10.7'), findsOneWidget);
    expect(find.text('1 improvement and 1 fix.'), findsOneWidget);
    expect(find.text('Faster'), findsOneWidget);
    expect(find.text('A crash'), findsOneWidget);
    expect(
      find.byKey(const ValueKey<String>('whats-new-earlier')),
      findsNothing,
    );
  });

  testWidgets('a video leads the sheet', (tester) async {
    await pumpKanso(
      tester,
      WhatsNewSheet(
        whatsNew: _whatsNew(videoPath: '/nowhere/Kanso-0.10.7-whats-new.webp'),
      ),
    );
    expect(
      find.byKey(const ValueKey<String>('whats-new-video')),
      findsOneWidget,
    );
  });

  testWidgets('skipped releases are folded until opened', (tester) async {
    tester.view.physicalSize = const Size(800, 4000);
    addTearDown(tester.view.reset);
    for (var skipped = 1; skipped <= 4; skipped++) {
      await pumpKanso(
        tester,
        WhatsNewSheet(
          key: ValueKey<int>(skipped),
          whatsNew: _whatsNew(skipped: skipped),
        ),
      );
      final toggle = find.byKey(const ValueKey<String>('whats-new-earlier'));
      expect(toggle, findsOneWidget);
      expect(find.text('Fixed in 0.10.6'), findsNothing);
      await tester.tap(toggle);
      await tester.pump();
      for (var i = 0; i < skipped; i++) {
        expect(find.text('Fixed in 0.10.${6 - i}'), findsOneWidget);
      }
    }
  });
}
