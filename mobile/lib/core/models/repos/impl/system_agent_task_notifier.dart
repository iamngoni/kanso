// Kanso — system_agent_task_notifier.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/types.dart';
import '../abs/agent_task_notifier.dart';
import '../abs/reminder_notifier.dart';

/// The key prefix of an agent task's notification.
const String agentTaskKeyPrefix = 'agent-task:';

/// The notification text, which holds no note text.
const String agentTaskPausedTitle = 'Agent task paused';
const String agentTaskPausedBody = 'Open Kanso to review what it found.';

/// [AgentTaskNotifier] over the system's reminders channel.
@LazySingleton(as: AgentTaskNotifier)
class SystemAgentTaskNotifier implements AgentTaskNotifier {
  SystemAgentTaskNotifier(this._notifier);

  final ReminderNotifier _notifier;

  @override
  Future<void> paused({required int id}) async {
    final state = WidgetsBinding.instance.lifecycleState;
    if (state == null || state == AppLifecycleState.resumed) {
      return;
    }
    final now = DateTime.now();
    await _notifier.show(<ReminderNote>[
      ReminderNote(
        key: '$agentTaskKeyPrefix$id',
        noteId: '',
        line: 0,
        title: agentTaskPausedTitle,
        body: agentTaskPausedBody,
        fireAt: LocalTime(
          year: now.year,
          month: now.month,
          day: now.day,
          hour: now.hour,
          minute: now.minute,
          second: now.second,
        ),
        actions: false,
      ),
    ]);
  }
}
