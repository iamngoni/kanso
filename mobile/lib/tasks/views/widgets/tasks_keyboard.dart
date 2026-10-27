// Kanso — tasks_keyboard.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart';

/// The keyboard of the Tasks list: up and down move between tasks, space
/// ticks the one it is on, return opens its note and the undo key takes the
/// last tick back, as the desktop's Tasks view has them.
///
/// [builder] draws the list with the key of the task the keyboard is on
/// (null before it has moved).
class TasksKeyboard extends StatefulWidget {
  const TasksKeyboard({
    required this.tasks,
    required this.onTick,
    required this.onOpen,
    required this.onUndo,
    required this.builder,
    super.key,
  });

  /// The tasks listed, in order.
  final List<TaskItem> tasks;
  final ValueChanged<TaskItem> onTick;
  final ValueChanged<TaskItem> onOpen;
  final VoidCallback onUndo;
  final Widget Function(BuildContext context, String? focusedKey) builder;

  @override
  State<TasksKeyboard> createState() => TasksKeyboardState();
}

/// The state of [TasksKeyboard]: the key of the task the keyboard is on.
class TasksKeyboardState extends State<TasksKeyboard> {
  final ValueNotifier<String?> focused = ValueNotifier<String?>(null);

  @override
  void dispose() {
    focused.dispose();
    super.dispose();
  }

  TaskItem? current() {
    for (final task in widget.tasks) {
      if (task.key == focused.value) {
        return task;
      }
    }
    return null;
  }

  void step(int delta) {
    final tasks = widget.tasks;
    if (tasks.isEmpty) {
      return;
    }
    final at = tasks.indexWhere((task) => task.key == focused.value);
    final next = at < 0
        ? (delta > 0 ? 0 : tasks.length - 1)
        : (at + delta).clamp(0, tasks.length - 1);
    focused.value = tasks[next].key;
  }

  @override
  Widget build(BuildContext context) {
    void onTask(ValueChanged<TaskItem> act) {
      final task = current();
      if (task != null) {
        act(task);
      }
    }

    return CallbackShortcuts(
      bindings: <ShortcutActivator, VoidCallback>{
        const SingleActivator(LogicalKeyboardKey.arrowDown): () => step(1),
        const SingleActivator(LogicalKeyboardKey.arrowUp): () => step(-1),
        const SingleActivator(LogicalKeyboardKey.space): () =>
            onTask(widget.onTick),
        const SingleActivator(LogicalKeyboardKey.enter): () =>
            onTask(widget.onOpen),
        SingleActivator(
          LogicalKeyboardKey.keyZ,
          meta:
              defaultTargetPlatform == TargetPlatform.macOS ||
              defaultTargetPlatform == TargetPlatform.iOS,
          control:
              defaultTargetPlatform != TargetPlatform.macOS &&
              defaultTargetPlatform != TargetPlatform.iOS,
        ): widget.onUndo,
      },
      child: Focus(
        autofocus: true,
        child: ValueListenableBuilder<String?>(
          valueListenable: focused,
          builder: (context, key, _) => widget.builder(context, key),
        ),
      ),
    );
  }
}
