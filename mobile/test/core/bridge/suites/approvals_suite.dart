// Kanso — approvals_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The requests of other devices, through the bridge: every call of
/// `KansoApprovals`, signed out.
void approvalsSuite(OpenLibrary open) {
  group('KansoApprovals', () {
    test('signed out, there is nothing to answer', () async {
      final approvals = (await open()).approvals();
      await expectLater(
        approvals.refresh(),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        approvals.approve(id: 'request'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        approvals.decline(id: 'request'),
        throwsA(isA<KansoError_SignedOut>()),
      );
    });

    test('the watch starts with the requests now: none', () async {
      final approvals = (await open()).approvals();
      final first = await approvals.watch().first.timeout(
        const Duration(seconds: 10),
      );
      expect(first, isEmpty);
    });
  });
}
