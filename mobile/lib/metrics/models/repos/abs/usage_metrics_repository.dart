// Kanso — usage_metrics_repository.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A feature the app's product code may count. The names are the closed
/// list `kanso-metrics` accepts; Rust keeps the counts and reports each as a
/// range with the next hourly batch.
enum UsageFeature {
  noteCreated('note_created'),
  voiceInput('voice_input'),
  publish('publish'),
  lock('lock'),
  ai('ai'),
  quickCapture('quick_capture'),
  sharedNote('shared_note'),
  export('export'),
  search('search'),
  graphOpened('graph_opened'),
  themeChanged('theme_changed'),
  wikilinkFollowed('wikilink_followed'),
  focusMode('focus_mode'),
  reviewPrompt('review_prompt');

  const UsageFeature(this.wireName);

  /// The name sent across the bridge.
  final String wireName;
}

/// The anonymous usage count as the Flutter shell sees it: the notice state,
/// feature counts, and the daily batch. Implementations must never make a
/// normal note action fail because metrics could not be written or sent.
abstract interface class UsageMetricsRepository {
  /// Whether the one-time notice was acknowledged.
  Future<bool> noticeSeen();

  /// Keeps the acknowledgement. Returns whether it was saved.
  Future<bool> markNoticeSeen();

  /// Counts one use of [feature], if the notice was seen.
  Future<void> record(UsageFeature feature);

  /// Notes that the app was launched (once per process).
  Future<void> launched();

  /// Notes that the app was in the foreground for [seconds] and has gone to
  /// the background.
  Future<void> endSession(int seconds);

  /// Sends what is due. Returns whether the Worker accepted a batch.
  /// [locale] is the interface language's code; [themeMode] is `light`,
  /// `dark` or `system` and [theme] the theme's ID, either empty to leave it
  /// out. The core checks all three against its closed lists.
  Future<bool> sendDue({
    required bool tablet,
    required String osVersion,
    required bool ai,
    String locale = '',
    String themeMode = '',
    String theme = '',
  });
}
