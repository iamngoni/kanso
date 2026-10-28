// Kanso — kanso_auth_session.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'src/messages.g.dart';

/// The system's sign-in session for the Apple, Google and GitHub pages:
/// `ASWebAuthenticationSession` on iOS, a Custom Tab on Android.
///
/// Only `http` and `https` pages are opened; the native side refuses
/// anything else as well.
abstract final class KansoAuthSession {
  static AuthSessionHostApi _api = AuthSessionHostApi();

  /// Replaces the channel. For tests.
  static set debugApi(AuthSessionHostApi api) => _api = api;

  /// Opens [url] and completes with the URL the session ended at when it
  /// ends at [callbackScheme] (Apple: `kanso`), or with null when the person
  /// dismissed it or [close] ended it. Without a [callbackScheme] (Google's
  /// loopback, GitHub's device flow) the session ends only by [close] or by
  /// the person.
  ///
  /// Throws an [ArgumentError] for a [url] that is not `http` or `https`.
  static Future<String?> open(String url, {String? callbackScheme}) {
    if (!isWebUrl(url)) {
      throw ArgumentError.value(url, 'url', 'must be an http or https URL');
    }
    return _api.open(url, callbackScheme);
  }

  /// Ends the open session, if there is one, and brings the app back in
  /// front.
  static Future<void> close() => _api.close();

  /// Whether [url] is an `http` or `https` URL with a host.
  static bool isWebUrl(String url) {
    final uri = Uri.tryParse(url);
    if (uri == null || !uri.hasAuthority || uri.host.isEmpty) return false;
    final scheme = uri.scheme.toLowerCase();
    return scheme == 'http' || scheme == 'https';
  }
}
