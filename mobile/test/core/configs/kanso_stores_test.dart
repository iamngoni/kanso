// Kanso — kanso_stores_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_stores.dart';

void main() {
  group('KansoStores', () {
    test('iOS rates the app on its App Store write-review page', () {
      expect(
        KansoStores.writeReviewFor(TargetPlatform.iOS).toString(),
        'itms-apps://apps.apple.com/app/id6828301844?action=write-review',
      );
    });

    test('Android has no page until the Play listing is live', () {
      expect(
        KansoStores.writeReviewFor(TargetPlatform.android),
        KansoStores.playListingLive ? KansoStores.playListing : null,
      );
      expect(
        KansoStores.playListing.toString(),
        'market://details?id=app.kanso.kanso',
      );
    });

    test('the desktops have none', () {
      for (final platform in <TargetPlatform>[
        TargetPlatform.macOS,
        TargetPlatform.windows,
        TargetPlatform.linux,
      ]) {
        expect(KansoStores.writeReviewFor(platform), isNull);
      }
    });

    test("matches the website's store values", () {
      final site = File('../site/src/lib/stores.ts').readAsStringSync();
      expect(
        site,
        contains('APP_STORE_ID = "${KansoStores.appStoreId}"'),
      );
      // The website sets ANDROID_URL when the listing is live; so does this.
      expect(
        site.contains('ANDROID_URL: string | null = null'),
        !KansoStores.playListingLive,
      );
    });
  });
}
