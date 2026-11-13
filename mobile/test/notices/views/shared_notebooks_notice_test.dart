// Kanso — shared_notebooks_notice_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notices_types.dart';
import 'package:kanso/notices/state/notices_cubit.dart';
import 'package:kanso/notices/views/widgets/notice_card.dart';
import 'package:kanso/notices/views/widgets/notice_card_host.dart';
import 'package:kanso/notices/views/widgets/notice_sheet.dart';

import '../../helpers/fixture_notices_repository.dart';
import '../../helpers/pump.dart';

/// What the library hands the phone when the sync server says this Kanso is
/// too old for shared notebooks (`kanso-core`'s `SHARED_NOTEBOOKS_UPDATE`).
const String _restart =
    'Restart Kanso, then open the Shared section to see your shared '
    'notebooks and any invitations.';
const String _others =
    'Update Kanso on your other computers and phones too, so they keep up '
    'with shared notebooks.';

const PhoneNotice _update = PhoneNotice(
  id: 'shared-notebooks-update-0.10',
  title: 'Update Kanso to use shared notebooks',
  whatChanged:
      'The sync server no longer takes shared-notebook requests from this '
      'version of Kanso. Your notes keep syncing; only shared notebooks, and '
      'invitations to them, wait until Kanso is updated.',
  steps: <String>[
    'Open Settings › Updates and install the latest version of Kanso.',
    _restart,
    _others,
  ],
  actionKind: PhoneNoticeActionKind.openSettings,
  actionLabel: 'Open Settings',
  actionTarget: 'updates',
);

Widget _host(NoticesCubit cubit) => BlocProvider<NoticesCubit>.value(
  value: cubit,
  child: const Scaffold(body: NoticeCardHost()),
);

void main() {
  testWidgets('nothing shows until the server has refused the phone', (
    tester,
  ) async {
    final cubit = NoticesCubit(FixtureNoticesRepository(applying: const []));
    addTearDown(cubit.close);
    await pumpKanso(tester, _host(cubit));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeCard), findsNothing);
  });

  testWidgets('the card says to update, and Dismiss hides it for good', (
    tester,
  ) async {
    final repository = FixtureNoticesRepository(
      applying: const <PhoneNotice>[_update],
    );
    final cubit = NoticesCubit(repository);
    addTearDown(cubit.close);
    await pumpKanso(tester, _host(cubit));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeCard), findsOneWidget);
    expect(find.text('Update Kanso to use shared notebooks'), findsOneWidget);
    expect(find.text('Dismiss'), findsOneWidget);

    await tester.tap(find.text('Dismiss'));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeCard), findsNothing);
    expect(repository.dismissed, <String>{_update.id});
  });

  testWidgets('How to fix lists the steps and leads to the update page', (
    tester,
  ) async {
    final cubit = NoticesCubit(
      FixtureNoticesRepository(applying: const <PhoneNotice>[_update]),
    );
    addTearDown(cubit.close);
    await pumpKanso(tester, _host(cubit));
    await tester.pumpAndSettle();
    await tester.tap(find.text('How to fix'));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeSheet), findsOneWidget);
    // The card keeps its summary behind the sheet; the sheet has its own.
    Finder inSheet(Finder f) =>
        find.descendant(of: find.byType(NoticeSheet), matching: f);
    expect(inSheet(find.text(_update.whatChanged)), findsOneWidget);
    for (final step in _update.steps) {
      expect(inSheet(find.text(step)), findsOneWidget);
    }
    expect(inSheet(find.text('Open Settings')), findsOneWidget);
    // The button opens the phone's update page.
    expect(
      NoticeCardHost.settingsLocation(_update.actionTarget),
      '/settings/about',
    );
  });
}
