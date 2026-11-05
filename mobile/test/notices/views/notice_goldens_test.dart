// Kanso — notice_goldens_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notices_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_spacing.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/notices/state/notices_cubit.dart';
import 'package:kanso/notices/views/widgets/notice_card.dart';
import 'package:kanso/notices/views/widgets/notice_card_host.dart';
import 'package:kanso/notices/views/widgets/notice_sheet.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fixture_notices_repository.dart';
import '../../helpers/golden.dart';
import '../../helpers/pump.dart';

const PhoneNotice _notice = FixtureNoticesRepository.exampleNotice;

/// The card where it sits: under a list's bar, over the list's colour.
Widget _card({String countLabel = ''}) => Scaffold(
  resizeToAvoidBottomInset: false,
  body: Builder(
    builder: (context) => ColoredBox(
      color: context.colors.list,
      child: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            const SizedBox(height: KansoSpacing.xxl),
            NoticeCard(
              notice: _notice,
              countLabel: countLabel,
              onHowToFix: () {},
              onDismiss: () {},
            ),
          ],
        ),
      ),
    ),
  ),
);

/// The sheet as the panel of a bottom sheet holds it, over the list.
Widget _sheet({bool action = true}) => Scaffold(
  resizeToAvoidBottomInset: false,
  body: Builder(
    builder: (context) => ColoredBox(
      color: context.colors.list,
      child: Align(
        alignment: Alignment.bottomCenter,
        child: KansoBottomSheet(
          child: NoticeSheet(
            notice: _notice,
            onAction: action ? () {} : null,
            onMarkFixed: () {},
            onDismiss: () {},
          ),
        ),
      ),
    ),
  ),
);

void main() {
  for (final brightness in <Brightness>[Brightness.light, Brightness.dark]) {
    final name = brightness.name;
    testWidgets('the notice card, $name', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _card(),
        name: 'phone_notice_card_$name',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('the notice card with a count, $name', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _card(countLabel: '1 of 2'),
        name: 'phone_notice_card_count_$name',
        size: phoneSize,
        brightness: brightness,
      );
    });

    // The card where it lives: at the top of the notes list.
    testWidgets('the card above the notes list, $name', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      final cubit = NoticesCubit(FixtureNoticesRepository());
      addTearDown(cubit.close);
      await atMockNow(
        () => expectGoldenOnBothPlatforms(
          tester,
          BlocProvider<NoticesCubit>.value(
            value: cubit,
            child: blocs.provide(
              const Scaffold(
                resizeToAvoidBottomInset: false,
                body: NoteListPane(),
              ),
              list: list,
            ),
          ),
          name: 'phone_note_list_notice_$name',
          size: phoneSize,
          brightness: brightness,
        ),
      );
    });

    testWidgets('the How to fix sheet, $name', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _sheet(),
        name: 'phone_notice_sheet_$name',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }

  testWidgets('the sheet says what changed and lists the steps', (
    tester,
  ) async {
    await pumpKanso(
      tester,
      Scaffold(
        body: NoticeSheet(
          notice: _notice,
          onAction: null,
          onMarkFixed: () {},
          onDismiss: () {},
        ),
      ),
    );
    for (final step in _notice.steps) {
      expect(find.text(step), findsOneWidget);
    }
    expect(find.text(_notice.whatChanged), findsOneWidget);
  });

  testWidgets('the host is empty without a cubit above it', (tester) async {
    await pumpKanso(tester, const Scaffold(body: NoticeCardHost()));
    expect(find.byType(NoticeCard), findsNothing);
  });

  testWidgets('the host shows the card and Dismiss hides it for good', (
    tester,
  ) async {
    final repository = FixtureNoticesRepository();
    final cubit = NoticesCubit(repository);
    addTearDown(cubit.close);
    await pumpKanso(
      tester,
      BlocProvider<NoticesCubit>.value(
        value: cubit,
        child: const Scaffold(body: NoticeCardHost()),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byType(NoticeCard), findsOneWidget);
    expect(find.text(_notice.title), findsOneWidget);

    await tester.tap(find.text('Dismiss'));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeCard), findsNothing);
    expect(repository.dismissed, <String>{_notice.id});
  });

  testWidgets('How to fix opens the steps and Mark as fixed puts it away', (
    tester,
  ) async {
    final repository = FixtureNoticesRepository();
    final cubit = NoticesCubit(repository);
    addTearDown(cubit.close);
    await pumpKanso(
      tester,
      BlocProvider<NoticesCubit>.value(
        value: cubit,
        child: const Scaffold(body: NoticeCardHost()),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('How to fix'));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeSheet), findsOneWidget);

    await tester.tap(find.text('Mark as fixed'));
    await tester.pumpAndSettle();
    expect(find.byType(NoticeSheet), findsNothing);
    expect(find.byType(NoticeCard), findsNothing);
    expect(repository.fixed, <String>{_notice.id});
  });

  test('settings pages open on the phone where there is one', () {
    expect(NoticeCardHost.settingsLocation('account'), '/settings/account');
    expect(NoticeCardHost.settingsLocation('general'), '/settings/general');
    expect(NoticeCardHost.settingsLocation('updates'), '/settings/about');
    // The phones have no Local API page: Settings itself opens.
    expect(NoticeCardHost.settingsLocation('local-api'), '/settings');
    expect(NoticeCardHost.settingsLocation('anything'), '/settings');
  });
}
