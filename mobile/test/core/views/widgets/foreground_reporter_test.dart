// Kanso — foreground_reporter_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/core/views/widgets/foreground_reporter.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_approvals_repository.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/seeded_account_blocs.dart';

class _Rig {
  final FixtureSyncRepository sync = FixtureSyncRepository();
  final FixtureApprovalsRepository approvals = FixtureApprovalsRepository();
  final SeededAccountBloc account = SeededAccountBloc(
    const AccountBlocState(
      account: AccountState.signedIn(username: 'ana', server: 's'),
    ),
  );
}

/// Lets the blocs carry out what they were told.
Future<void> _settle(WidgetTester tester) async {
  await tester.pump();
  await tester.runAsync(pumpEventQueue);
  await tester.pump();
}

Future<_Rig> _pump(WidgetTester tester) async {
  final rig = _Rig();
  final status = SyncStatusBloc(rig.sync);
  final approval = DeviceApprovalBloc(rig.approvals);
  closeBlocLater(status);
  closeBlocLater(approval);
  closeBlocLater(rig.account);
  await tester.pumpWidget(
    Directionality(
      textDirection: TextDirection.ltr,
      child: MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<SyncStatusBloc>.value(value: status),
          BlocProvider<DeviceApprovalBloc>.value(value: approval),
          BlocProvider<AccountBloc>.value(value: rig.account),
        ],
        child: const ForegroundReporter(child: SizedBox()),
      ),
    ),
  );
  await _settle(tester);
  return rig;
}

void main() {
  group('ForegroundReporter', () {
    testWidgets('tells the app is in front as it starts, and watches for '
        'requests', (tester) async {
      final rig = await _pump(tester);
      expect(rig.sync.foreground, isTrue);
      expect(rig.approvals.watches, 1);
    });

    testWidgets('going behind stops both, coming back starts them', (
      tester,
    ) async {
      final rig = await _pump(tester);
      for (final state in <AppLifecycleState>[
        AppLifecycleState.inactive,
        AppLifecycleState.hidden,
        AppLifecycleState.paused,
      ]) {
        tester.binding.handleAppLifecycleStateChanged(state);
      }
      await _settle(tester);
      expect(rig.sync.foreground, isFalse);

      for (final state in <AppLifecycleState>[
        AppLifecycleState.hidden,
        AppLifecycleState.inactive,
        AppLifecycleState.resumed,
      ]) {
        tester.binding.handleAppLifecycleStateChanged(state);
      }
      await _settle(tester);
      expect(rig.sync.foreground, isTrue);
      expect(rig.approvals.watches, 2);
    });

    testWidgets('inactive only (a pulled-down shade) is still in front', (
      tester,
    ) async {
      final rig = await _pump(tester);
      tester.binding.handleAppLifecycleStateChanged(
        AppLifecycleState.inactive,
      );
      await _settle(tester);
      expect(rig.sync.foreground, isTrue);
      expect(rig.approvals.watches, 1);
    });

    testWidgets('signing out stops watching for requests', (tester) async {
      final rig = await _pump(tester);
      rig.account.seed(
        const AccountBlocState(account: AccountState.signedOut()),
      );
      await _settle(tester);
      // Watching again would have counted a second watch.
      expect(rig.approvals.watches, 1);
      rig.account.seed(
        const AccountBlocState(
          account: AccountState.signedIn(username: 'ana', server: 's'),
        ),
      );
      await _settle(tester);
      expect(rig.approvals.watches, 2);
    });
  });
}
