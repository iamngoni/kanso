// Kanso — account_forms.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/form_problem.dart';

/// The checks an account form makes before it sends anything, the same rules
/// as the library's (docs/ACCOUNTS.md › Usernames, › Password and KDF) so the
/// person is told at once. The library checks again; it has the last word.
abstract final class AccountForms {
  /// The fewest characters a password has.
  static const int minPasswordLength = 12;

  /// The most characters a password has.
  static const int maxPasswordLength = 256;

  /// Names nobody can sign up with.
  static const List<String> reservedUsernames = <String>[
    'abuse',
    'admin',
    'administrator',
    'anonymous',
    'api',
    'everyone',
    'guest',
    'help',
    'hostmaster',
    'kanso',
    'me',
    'null',
    'owner',
    'postmaster',
    'root',
    'security',
    'server',
    'support',
    'sync',
    'system',
    'undefined',
    'webmaster',
    'www',
  ];

  static final RegExp _pattern = RegExp(r'^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$');

  /// [input] as a username: stripped of the five blanks at either end,
  /// lowercased, and null when it breaks the rules (a character outside
  /// U+0021 to U+007E, or a form other than 3 to 32 letters, digits, `-` and
  /// `_` that starts and ends with a letter or digit).
  static String? normalizeUsername(String input) {
    final name = _trim(input);
    for (final unit in name.codeUnits) {
      if (unit < 0x21 || unit > 0x7e) {
        return null;
      }
    }
    final lower = _asciiLowercase(name);
    return _pattern.hasMatch(lower) ? lower : null;
  }

  /// Whether the normalized [username] is one nobody may sign up with.
  static bool isReserved(String username) =>
      reservedUsernames.contains(username);

  /// What is wrong with [username] as a name for a new account.
  static FormProblem? newUsernameProblem(String username) {
    final name = normalizeUsername(username);
    if (name == null) {
      return FormProblem.usernameInvalid;
    }
    return isReserved(name) ? FormProblem.usernameReserved : null;
  }

  /// What is wrong with [password] as a password for the account of
  /// [username]: its length, and that it is not the username.
  static FormProblem? passwordProblem(String password, String username) {
    final length = password.runes.length;
    if (length < minPasswordLength) {
      return FormProblem.passwordTooShort;
    }
    if (length > maxPasswordLength) {
      return FormProblem.passwordTooLong;
    }
    final name = normalizeUsername(username) ?? username;
    return password.toLowerCase() == name.toLowerCase()
        ? FormProblem.passwordSameAsUsername
        : null;
  }

  /// What is wrong with a new password and its repeat.
  static FormProblem? newPasswordProblem(
    String password,
    String repeat,
    String username,
  ) {
    final problem = passwordProblem(password, username);
    if (problem != null) {
      return problem;
    }
    return password == repeat ? null : FormProblem.passwordsDiffer;
  }

  /// What is wrong with the Create account form: the username first, then
  /// the password and its repeat.
  static FormProblem? signUpProblem(
    String username,
    String password,
    String repeat,
  ) {
    final problem = newUsernameProblem(username);
    if (problem != null) {
      return problem;
    }
    return newPasswordProblem(password, repeat, username);
  }

  /// What is wrong with the Sign in form. A password no account can have is
  /// answered like a wrong one.
  static FormProblem? signInProblem(String username, String password) {
    if (normalizeUsername(username) == null) {
      return FormProblem.usernameInvalid;
    }
    return passwordProblem(password, username) == null
        ? null
        : FormProblem.credentialsDontMatch;
  }

  /// What is wrong with the password typed to unlock a locked account.
  static FormProblem? unlockProblem(String password, String username) {
    return passwordProblem(password, username) == null
        ? null
        : FormProblem.credentialsDontMatch;
  }

  /// What is wrong with the recovery form: the username, the key, and the
  /// new password twice.
  static FormProblem? recoveryProblem({
    required String username,
    required String recoveryKey,
    required String password,
    required String repeat,
  }) {
    if (normalizeUsername(username) == null) {
      return FormProblem.usernameInvalid;
    }
    if (recoveryKey.trim().isEmpty) {
      return FormProblem.recoveryKeyMissing;
    }
    return newPasswordProblem(password, repeat, username);
  }

  static const Set<int> _blanks = <int>{0x09, 0x0a, 0x0c, 0x0d, 0x20};

  static String _trim(String input) {
    final units = input.codeUnits;
    var start = 0;
    var end = units.length;
    while (start < end && _blanks.contains(units[start])) {
      start++;
    }
    while (end > start && _blanks.contains(units[end - 1])) {
      end--;
    }
    return input.substring(start, end);
  }

  static String _asciiLowercase(String text) {
    final out = StringBuffer();
    for (final unit in text.codeUnits) {
      out.writeCharCode(unit >= 0x41 && unit <= 0x5a ? unit + 0x20 : unit);
    }
    return out.toString();
  }
}
