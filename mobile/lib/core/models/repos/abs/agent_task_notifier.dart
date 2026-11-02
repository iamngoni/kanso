// Kanso — agent_task_notifier.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Tells the person, outside the app, that an agent task needs them.
// ignore: one_member_abstracts
abstract interface class AgentTaskNotifier {
  /// "Agent task paused. Open Kanso to review what it found." — shown only
  /// while the app is not in front. No note text is in it.
  Future<void> paused({required int id});
}
