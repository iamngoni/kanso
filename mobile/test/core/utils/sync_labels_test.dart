// Kanso — sync_labels_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/sync_labels.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

void main() {
  final l10n = AppLocalizationsEn();

  test('no status yet says nothing', () {
    expect(l10n.syncStatusName(null), '');
  });

  test('every phase has its own words, Off while signed out', () {
    final names = <SyncPhase, String>{
      for (final phase in SyncPhase.values)
        phase: l10n.syncStatusName(
          SyncStatus(
            phase: phase,
            conflicts: 0,
            skippedCount: 0,
            account: phase == SyncPhase.signedOut
                ? const SyncAccount.signedOut()
                : const SyncAccount.signedIn(),
          ),
        ),
    };
    expect(names[SyncPhase.signedOut], 'Off');
    expect(names.values.toSet().length, SyncPhase.values.length);
    expect(names.values, everyElement(isNotEmpty));
  });
}
