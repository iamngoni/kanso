// Kanso — account_error_text_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, StringAnys, any;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/account_error_text.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

void main() {
  final l10n = AppLocalizationsEn();

  group('AccountErrorText', () {
    test('the library’s own words are shown as they are', () {
      for (final error in <ApplicationError>[
        const ApplicationError.invalid(message: 'That username is taken.'),
        const ApplicationError.auth(
          message: 'That username and password don’t match.',
        ),
        const ApplicationError.network(
          message:
              'Can’t reach the server. Your notes are safe on this '
              'device.',
        ),
        const ApplicationError.storage(message: 'The library is not open.'),
      ]) {
        expect(l10n.accountError(error), error.message);
      }
    });

    test('a failure with no words, and a bug, get the app’s own line', () {
      expect(
        l10n.accountError(const ApplicationError.invalid(message: '  ')),
        l10n.accountSomethingWrong,
      );
      expect(
        l10n.accountError(const ApplicationError.internal(message: 'panic')),
        l10n.accountSomethingWrong,
      );
      expect(
        l10n.accountError(const ApplicationError.unexpected(message: 'x')),
        l10n.accountSomethingWrong,
      );
      expect(
        l10n.accountError(const ApplicationError.notFound(message: 'x')),
        l10n.accountSomethingWrong,
      );
    });

    test('a session the server ended says so', () {
      expect(
        l10n.accountError(const ApplicationError.signedOut(message: 'x')),
        l10n.accountSignedOutError,
      );
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'what is shown is never empty',
      (message) {
        for (final error in <ApplicationError>[
          ApplicationError.invalid(message: message),
          ApplicationError.auth(message: message),
          ApplicationError.network(message: message),
          ApplicationError.storage(message: message),
          ApplicationError.internal(message: message),
        ]) {
          expect(l10n.accountError(error), isNotEmpty);
        }
      },
    );
  });
}
