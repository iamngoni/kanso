// Kanso — account_forms_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: prefer_interpolation_to_compose_strings

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        Generator,
        GeneratorUtils,
        Glados,
        Glados2,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/utils/account_forms.dart';

import '../../helpers/property.dart';

/// The characters a username is typed with, the ones it may not have, and
/// the ones a fixture flags: letters of both cases, digits, hyphen and
/// underscore, the blanks that are trimmed and those that are not, and what
/// is not ASCII.
final List<String> _typed = <String>[
  'a',
  'b',
  'Z',
  '7',
  '-',
  '_',
  ' ',
  '\t',
  '\n',
  String.fromCharCode(0x0b),
  '.',
  '@',
  String.fromCharCode(0xe9),
  String.fromCharCode(0x212a),
  String.fromCharCode(0xa0),
];

extension _FormsAny on Any {
  Generator<String> get typedName => listWithLengthInRange<String>(
    0,
    36,
    choose(_typed),
  ).map((chars) => chars.join());

  Generator<String> get passwordLike => combine2(
    intInRange(0, 300),
    choose(<String>['x', 'é', 'ß', String.fromCharCode(0x1F600)]),
    (int length, String unit) => unit * length,
  );
}

void main() {
  group('AccountForms, usernames, against the cases the Worker runs', () {
    final file = File('../cloud/test/fixtures/usernames.json');
    final fixture = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
    final cases = (fixture['cases'] as List<dynamic>)
        .cast<Map<String, dynamic>>();

    test('there are cases to run', () {
      expect(cases.length, greaterThanOrEqualTo(60));
    });

    test('every case normalizes as the Worker and the library do', () {
      for (final entry in cases) {
        final input = entry['input'] as String;
        expect(
          AccountForms.normalizeUsername(input),
          entry['normalized'],
          reason: 'normalizing ' + jsonEncode(input),
        );
      }
    });

    test('every case is accepted, invalid or reserved as they say', () {
      for (final entry in cases) {
        final input = entry['input'] as String;
        final error = entry['error'];
        final expected = switch (error) {
          null => null,
          'invalid_username' => FormProblem.usernameInvalid,
          'reserved_username' => FormProblem.usernameReserved,
          _ => fail('unknown error ' + error.toString()),
        };
        expect(
          AccountForms.newUsernameProblem(input),
          expected,
          reason: 'signing up as ' + jsonEncode(input),
        );
      }
    });

    test('the reserved names are the Worker’s', () {
      expect(
        AccountForms.reservedUsernames,
        (fixture['reserved'] as List<dynamic>).cast<String>(),
      );
    });
  });

  group('AccountForms, usernames', () {
    Glados<String>(any.typedName, kansoExploreConfig()).test(
      'normalizing twice is normalizing once',
      (input) {
        final name = AccountForms.normalizeUsername(input);
        if (name != null) {
          expect(AccountForms.normalizeUsername(name), name);
        }
      },
    );

    Glados<String>(any.typedName, kansoExploreConfig()).test(
      'a name that is accepted is 3 to 32 of a-z, 0-9, - and _, starting and '
      'ending with a letter or digit',
      (input) {
        final name = AccountForms.normalizeUsername(input);
        if (name != null) {
          expect(name, matches(RegExp(r'^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$')));
        }
      },
    );

    Glados<String>(any.typedName, kansoExploreConfig()).test(
      'a new name is refused as invalid, or as reserved, and only then',
      (input) {
        final name = AccountForms.normalizeUsername(input);
        final problem = AccountForms.newUsernameProblem(input);
        if (name == null) {
          expect(problem, FormProblem.usernameInvalid);
        } else if (AccountForms.reservedUsernames.contains(name)) {
          expect(problem, FormProblem.usernameReserved);
        } else {
          expect(problem, isNull);
        }
      },
    );

    test('only the five blanks are trimmed; any other is refused', () {
      final blanks = <int>[0x20, 0x09, 0x0a, 0x0c, 0x0d];
      final padded =
          String.fromCharCodes(blanks) +
          'ana' +
          String.fromCharCodes(blanks.reversed);
      expect(AccountForms.normalizeUsername(padded), 'ana');
      expect(
        AccountForms.normalizeUsername(String.fromCharCode(0x0b) + 'ana'),
        isNull,
      );
      expect(
        AccountForms.normalizeUsername('a' + String.fromCharCode(0xa0) + 'b'),
        isNull,
      );
      expect(
        AccountForms.normalizeUsername(String.fromCharCode(0x212a) + 'anso'),
        isNull,
      );
    });
  });

  group('AccountForms, passwords', () {
    Glados<String>(any.passwordLike, kansoExploreConfig()).test(
      'a password is too short under 12 characters, too long over 256',
      (password) {
        final length = password.runes.length;
        final problem = AccountForms.passwordProblem(password, 'ana');
        if (length < 12) {
          expect(problem, FormProblem.passwordTooShort);
        } else if (length > 256) {
          expect(problem, FormProblem.passwordTooLong);
        } else {
          expect(problem, isNull);
        }
      },
    );

    test('a username of a dozen characters is not a password for itself', () {
      expect(
        AccountForms.passwordProblem('Long-Name-123', 'long-name-123'),
        FormProblem.passwordSameAsUsername,
      );
      expect(
        AccountForms.passwordProblem('Long-Name-124', 'long-name-123'),
        isNull,
      );
    });

    Glados2<String, String>(
      any.passwordLike,
      any.passwordLike,
      kansoExploreConfig(),
    ).test('a new password and its repeat must be the same', (a, b) {
      final problem = AccountForms.newPasswordProblem(a, b, 'ana');
      final alone = AccountForms.passwordProblem(a, 'ana');
      if (alone != null) {
        expect(problem, alone);
      } else if (a != b) {
        expect(problem, FormProblem.passwordsDiffer);
      } else {
        expect(problem, isNull);
      }
    });
  });

  group('AccountForms, whole forms', () {
    const good = 'correct horse battery';

    test('a sign-up tells the username first, then the password', () {
      expect(
        AccountForms.signUpProblem('a', 'x', 'y'),
        FormProblem.usernameInvalid,
      );
      expect(
        AccountForms.signUpProblem('admin', good, good),
        FormProblem.usernameReserved,
      );
      expect(
        AccountForms.signUpProblem('ana', 'short', 'short'),
        FormProblem.passwordTooShort,
      );
      expect(
        AccountForms.signUpProblem('ana', good, good + '!'),
        FormProblem.passwordsDiffer,
      );
      expect(AccountForms.signUpProblem('ana', good, good), isNull);
    });

    Glados2<String, String>(
      any.typedName,
      any.passwordLike,
      kansoExploreConfig(),
    ).test('a sign-up is accepted exactly when both of its parts are', (
      username,
      password,
    ) {
      final problem = AccountForms.signUpProblem(username, password, password);
      final parts =
          AccountForms.newUsernameProblem(username) ??
          AccountForms.passwordProblem(password, username);
      expect(problem, parts);
    });

    test(
      'a sign-in answers a password no account can have like a wrong one',
      () {
        expect(
          AccountForms.signInProblem('ana', 'short'),
          FormProblem.credentialsDontMatch,
        );
        expect(
          AccountForms.signInProblem('a b', good),
          FormProblem.usernameInvalid,
        );
        expect(AccountForms.signInProblem('Ana', good), isNull);
        expect(
          AccountForms.unlockProblem('short', 'ana'),
          FormProblem.credentialsDontMatch,
        );
        expect(AccountForms.unlockProblem(good, 'ana'), isNull);
      },
    );

    test('a recovery needs a name, a key and a new password twice', () {
      FormProblem? problem({
        String username = 'ana',
        String key = 'K7QF-2MXD',
        String password = good,
        String? repeat,
      }) => AccountForms.recoveryProblem(
        username: username,
        recoveryKey: key,
        password: password,
        repeat: repeat ?? password,
      );

      expect(problem(username: '!'), FormProblem.usernameInvalid);
      expect(problem(key: ' '), FormProblem.recoveryKeyMissing);
      expect(problem(password: 'short'), FormProblem.passwordTooShort);
      expect(problem(repeat: 'other password'), FormProblem.passwordsDiffer);
      expect(problem(), isNull);
    });
  });
}
