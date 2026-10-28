// Kanso — device_approval_host_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/views/widgets/device_approval_host.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_approvals_repository.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/seeded_account_blocs.dart';

Future<FixtureApprovalsRepository> _pump(
  WidgetTester tester,
  List<ApprovalPrompt> prompts,
) async {
  final approvals = FixtureApprovalsRepository(prompts);
  final bloc = DeviceApprovalBloc(approvals)
    ..add(
      const DeviceApprovalEvent.conditionsChanged(
        signedIn: true,
        foreground: true,
      ),
    );
  final account = SeededAccountBloc(
    const AccountBlocState(
      account: AccountState.signedIn(username: 'ana', server: 's'),
    ),
  );
  closeBlocLater(bloc);
  closeBlocLater(account);
  await pumpKanso(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<DeviceApprovalBloc>.value(value: bloc),
        BlocProvider<AccountBloc>.value(value: account),
      ],
      child: const SizedBox(
        width: 393,
        height: 852,
        child: DeviceApprovalHost(child: Center(child: Text('the app'))),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return approvals;
}

void main() {
  group('DeviceApprovalHost', () {
    testWidgets('shows nothing while no request waits', (tester) async {
      await _pump(tester, <ApprovalPrompt>[]);
      expect(find.text('the app'), findsOneWidget);
      expect(find.text('Approve a device'), findsNothing);
    });

    testWidgets('a ready request asks to compare the code, in the desktop’s '
        'words', (tester) async {
      await _pump(tester, <ApprovalPrompt>[readyPrompt()]);
      expect(find.text('Approve a device'), findsOneWidget);
      expect(
        find.text('A device wants to join ana: iPad, asked at 14:02.'),
        findsOneWidget,
      );
      expect(
        find.text('Does it show 123 456? Only approve if it does.'),
        findsOneWidget,
      );
      expect(find.text('Approve'), findsOneWidget);
      expect(find.text('Don’t approve'), findsOneWidget);
    });

    testWidgets('Approve answers the request, and the sheet goes', (
      tester,
    ) async {
      final approvals = await _pump(tester, <ApprovalPrompt>[readyPrompt()]);
      await tester.tap(find.text('Approve'));
      await tester.pumpAndSettle();
      expect(approvals.answers, <String>['approve a1']);
      expect(find.text('Approve a device'), findsNothing);
    });

    testWidgets('Don’t approve turns it away', (tester) async {
      final approvals = await _pump(tester, <ApprovalPrompt>[readyPrompt()]);
      await tester.tap(find.text('Don’t approve'));
      await tester.pumpAndSettle();
      expect(approvals.answers, <String>['decline a1']);
    });

    testWidgets('a request that changed can only be declined', (tester) async {
      final approvals = await _pump(tester, <ApprovalPrompt>[
        readyPrompt(state: PromptState.changed),
      ]);
      expect(
        find.text('This request changed after it was made. Don’t approve.'),
        findsOneWidget,
      );
      expect(find.text('Approve'), findsNothing);
      expect(
        find.text('Does it show 123 456? Only approve if it does.'),
        findsNothing,
      );
      await tester.tap(find.text('Don’t approve'));
      await tester.pumpAndSettle();
      expect(approvals.answers, <String>['decline a1']);
    });

    testWidgets('an unnamed device is "A device", and the others waiting '
        'are counted', (tester) async {
      await _pump(tester, <ApprovalPrompt>[
        readyPrompt(deviceName: null),
        readyPrompt(id: 'a2'),
        readyPrompt(id: 'a3'),
      ]);
      expect(
        find.text('A device wants to join ana: A device, asked at 14:02.'),
        findsOneWidget,
      );
      expect(find.text('2 more requests are waiting.'), findsOneWidget);
    });
  });
}
