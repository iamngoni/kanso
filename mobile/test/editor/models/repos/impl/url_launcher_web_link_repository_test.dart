// Kanso — url_launcher_web_link_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/models/repos/impl/url_launcher_web_link_repository.dart';

void main() {
  group('UrlLauncherWebLinkRepository', () {
    test('opens only web and mail addresses', () async {
      final repository = UrlLauncherWebLinkRepository();
      for (final url in <String>[
        'javascript:alert(1)',
        'file:///etc/passwd',
        'tel:+10000000',
        'sms:+10000000',
        'kanso://open/note/x',
        'intent://scan/#Intent;scheme=zxing;end',
        'not an address',
        '',
      ]) {
        final result = await repository.open(url);
        expect(
          result.fold((error) => error, (_) => null),
          isA<InvalidError>(),
          reason: url,
        );
      }
    });

    test('the schemes it opens are http, https and mailto', () {
      expect(UrlLauncherWebLinkRepository.schemes, <String>{
        'http',
        'https',
        'mailto',
      });
    });
  });
}
