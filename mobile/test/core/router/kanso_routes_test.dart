// Kanso — kanso_routes_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, StringAnys, any;
import 'package:kanso/core/router/kanso_routes.dart';

import '../../helpers/property.dart';

void main() {
  group('KansoRoutes, the account', () {
    test('the screens of the account are under Settings', () {
      for (final path in <String>[
        KansoRoutes.settingsAccount,
        KansoRoutes.settingsSignIn,
        KansoRoutes.settingsUnlock,
        KansoRoutes.settingsRecover,
        KansoRoutes.settingsRecoveryKey,
      ]) {
        expect(path, startsWith('/settings/account'));
      }
    });

    test('the recovery form is for the default server unless one is named', () {
      expect(KansoRoutes.recover(), KansoRoutes.settingsRecover);
      expect(KansoRoutes.recover(server: ''), KansoRoutes.settingsRecover);
      final named = Uri.parse(
        KansoRoutes.recover(server: 'https://sync.example.org'),
      );
      expect(named.path, KansoRoutes.settingsRecover);
      expect(
        named.queryParameters[KansoRoutes.serverParameter],
        'https://sync.example.org',
      );
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'a server named comes back as it was named',
      (name) {
        if (name.isEmpty) {
          return;
        }
        final address = 'https://$name.example.com:8787/a b?x=1&y=2';
        final uri = Uri.parse(KansoRoutes.recover(server: address));
        expect(uri.queryParameters[KansoRoutes.serverParameter], address);
      },
    );
  });
}
