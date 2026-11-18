// Kanso — session_tracker.dart
// Created 2026-11-18.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The default longest stay in the background that continues a session.
///
/// A minute covers the system round trips that pause the app on a phone
/// (picking a photo or file, taking a picture, a biometric prompt, the share
/// sheet, a glance at the notification shade) with room to spare, while
/// someone who puts the phone down and picks it up later starts a new
/// sitting.
const Duration defaultSessionGrace = Duration(seconds: 60);

/// Turns the app's trips between front and background into sessions
/// (docs/METRICS.md › `session`): a sitting, not each stretch in front.
///
/// A stay in the background no longer than [grace] continues the session;
/// a longer one ends it when the app comes back. A session's length is only
/// the time spent in front. Pure: the caller passes the time in.
class SessionTracker {
  SessionTracker({required DateTime now, this.grace = defaultSessionGrace})
    : _inFrontSince = now;

  final Duration grace;

  /// When the app last came to the front; null while it is away.
  DateTime? _inFrontSince;

  /// When the app went away; null while it is in front or no session is
  /// open.
  DateTime? _leftAt;

  /// Whole seconds in front from the stretches before the latest trip away.
  int _seconds = 0;

  /// The app is in front again. Returns the seconds of the session that
  /// ended because the stay away was longer than [grace], or null when the
  /// open session goes on (or the app was already in front).
  int? resume(DateTime now) {
    if (_inFrontSince != null) {
      return null;
    }
    final leftAt = _leftAt;
    final ended = leftAt != null && now.difference(leftAt) > grace
        ? end()
        : null;
    _leftAt = null;
    _inFrontSince = now;
    return ended;
  }

  /// The app went to the background. Returns whether it was in front, so
  /// the caller sends only on a real change.
  bool leave(DateTime now) {
    final since = _inFrontSince;
    if (since == null) {
      return false;
    }
    final seconds = now.difference(since).inSeconds;
    // A clock that went back counts nothing rather than a negative length.
    _seconds += seconds < 0 ? 0 : seconds;
    _inFrontSince = null;
    _leftAt = now;
    return true;
  }

  /// Ends the session now (the app is detached). Returns its seconds, or
  /// null when none is open: the app must have left the front first.
  int? end() {
    final open = _leftAt != null;
    final seconds = _seconds;
    _seconds = 0;
    _leftAt = null;
    return open ? seconds : null;
  }
}
