// Kanso — feedback_credit.dart
// Created 2026-11-23.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'feedback_credit.freezed.dart';

/// How a report asks to be credited once it is fixed.
enum FeedbackCreditKind {
  /// A plain name, written as itself in the public issue. Never a link.
  name,

  /// A GitHub username picked from the search; the issue mentions
  /// `@username`, so GitHub notifies that person.
  github,
}

/// How long typing must pause before GitHub is searched.
const Duration githubSearchDebounce = Duration(milliseconds: 300);

/// The most users the search returns.
const int githubSearchLimit = 8;

/// A GitHub user the picker offers: the login and where the avatar is.
@freezed
abstract class FeedbackGithubUser with _$FeedbackGithubUser {
  const factory FeedbackGithubUser({
    required String login,

    /// An https avatar URL on GitHub's avatar host.
    required String avatarUrl,
  }) = _FeedbackGithubUser;
}

/// What the GitHub picker shows under its search box.
@freezed
sealed class FeedbackGithubSearch with _$FeedbackGithubSearch {
  /// Nothing typed yet, or too little to search.
  const factory FeedbackGithubSearch.idle() = FeedbackGithubSearchIdle;

  /// Waiting for the pause after typing, or for GitHub.
  const factory FeedbackGithubSearch.searching() =
      FeedbackGithubSearchSearching;

  /// The users found (possibly none).
  const factory FeedbackGithubSearch.found(List<FeedbackGithubUser> users) =
      FeedbackGithubSearchFound;

  /// The search could not be made; the person can credit a name instead.
  const factory FeedbackGithubSearch.unavailable() =
      FeedbackGithubSearchUnavailable;
}

final RegExp _loginChars = RegExp(r'^[A-Za-z0-9-]+$');

/// Whether [login] is a GitHub login: 1 to 39 ASCII letters, digits and
/// single hyphens, with no hyphen first, last or doubled. The Worker checks
/// the same rule before it writes `@login` into an issue.
bool isGithubLogin(String login) =>
    login.isNotEmpty &&
    login.length <= 39 &&
    _loginChars.hasMatch(login) &&
    !login.startsWith('-') &&
    !login.endsWith('-') &&
    !login.contains('--');

/// The text to search for, or null when there is nothing to ask: a leading
/// `@` dropped, then 2 to 39 login characters (a hyphen anywhere is fine for
/// a prefix). Nothing else is ever put in the request.
String? githubSearchQuery(String raw) {
  var text = raw.trim();
  if (text.startsWith('@')) {
    text = text.substring(1);
  }
  return text.length >= 2 && text.length <= 39 && _loginChars.hasMatch(text)
      ? text
      : null;
}
