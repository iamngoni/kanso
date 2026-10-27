// Kanso — gallery_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The theme gallery, through the bridge: every call of `KansoGallery`. The
/// signed index is fetched from the gallery's own address (the one the
/// desktop reads, with no account and nothing of the person's), so a machine
/// with no network gets a network error instead of a list.
void gallerySuite(OpenLibrary open) {
  group('KansoGallery', () {
    test('nothing is installed in a new library', () async {
      final gallery = (await open()).gallery();
      expect(await gallery.installedThemes(), isEmpty);
    });

    test('a theme that is not installed has no tokens', () async {
      final gallery = (await open()).gallery();
      await expectLater(
        gallery.installedThemeTokens(id: 'no-such-theme'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('the index lists the gallery, or says the network is out', () async {
      final gallery = (await open()).gallery();
      try {
        final items = await gallery.index(refresh: false);
        expect(items.where((item) => item.installed), isEmpty);
        final again = await gallery.index(refresh: true);
        expect(again.length, items.length);
      } on KansoError_Network {
        // No network: the error is the answer.
      }
    });

    test('a theme that is not in the gallery is not found', () async {
      final gallery = (await open()).gallery();
      for (final call in <Future<Object?> Function()>[
        () => gallery.install(id: 'no-such-theme'),
        () => gallery.uninstall(id: 'no-such-theme'),
      ]) {
        await expectLater(
          call(),
          throwsA(
            anyOf(isA<KansoError_NotFound>(), isA<KansoError_Network>()),
          ),
        );
      }
    });
  });
}
