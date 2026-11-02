// Kanso — agent_tasks_page.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/agent_tasks.dart' as bridge;
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';

/// Agent tasks: a goal, a progress card for each task, and the review of what
/// the AI proposed. Nothing is written until the person applies it, and Undo
/// stays for 30 seconds.
class AgentTasksPage extends StatefulWidget {
  const AgentTasksPage({required this.tasks, super.key});

  final bridge.KansoAgentTasks tasks;

  @override
  State<AgentTasksPage> createState() => _AgentTasksPageState();
}

class _AgentTasksPageState extends State<AgentTasksPage> {
  final TextEditingController _goal = TextEditingController();
  Timer? _poll;
  List<bridge.TaskView> _rows = const <bridge.TaskView>[];
  String? _error;

  @override
  void initState() {
    super.initState();
    unawaited(_refresh());
    _poll = Timer.periodic(const Duration(seconds: 1), (_) => _refresh());
  }

  @override
  void dispose() {
    _poll?.cancel();
    _goal.dispose();
    super.dispose();
  }

  Future<void> _refresh() async {
    final rows = await widget.tasks.tasks();
    if (mounted) {
      setState(() => _rows = rows);
    }
  }

  Future<void> _start() async {
    final goal = _goal.text.trim();
    if (goal.isEmpty) {
      return;
    }
    try {
      await widget.tasks.start(
        goal: goal,
        noteIds: const <String>[],
        newNoteBook: '',
      );
      _goal.clear();
      setState(() => _error = null);
      await _refresh();
    } on Object catch (error) {
      setState(() => _error = '$error');
    }
  }

  Future<void> _review(bridge.TaskView row) async {
    final applied = await Navigator.of(context).push<String>(
      MaterialPageRoute<String>(
        builder: (_) => AgentReviewPage(tasks: widget.tasks, task: row),
      ),
    );
    if (applied == null || !mounted) {
      return;
    }
    await _refresh();
    if (!mounted) {
      return;
    }
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(applied),
        duration: const Duration(seconds: 30),
        action: SnackBarAction(
          label: 'Undo',
          onPressed: () async {
            await widget.tasks.undo(
              id: row.id,
              nowMs: DateTime.now().millisecondsSinceEpoch,
            );
            await _refresh();
          },
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Scaffold(
      backgroundColor: colors.app,
      appBar: AppBar(title: const Text('Agent tasks')),
      body: KansoPage(
        backgroundColor: colors.app,
        body: ListView(
          padding: const EdgeInsets.all(KansoSpacing.md),
          children: <Widget>[
            TextField(
              controller: _goal,
              minLines: 2,
              maxLines: 5,
              decoration: const InputDecoration(
                hintText: 'Merge my Lisbon trip notes into one plan',
                labelText: 'What should the AI do?',
              ),
            ),
            const SizedBox(height: KansoSpacing.xs),
            Align(
              alignment: Alignment.centerRight,
              child: FilledButton(
                onPressed: _start,
                child: const Text('Start'),
              ),
            ),
            if (_error != null) Text(_error!),
            for (final row in _rows)
              _TaskCard(
                row: row,
                onStop: () => widget.tasks.stop(id: row.id),
                onDiscard: () async {
                  await widget.tasks.discard(id: row.id);
                  await _refresh();
                },
                onReview: () => _review(row),
                onReply: (text) => widget.tasks.reply(id: row.id, text: text),
              ),
          ],
        ),
      ),
    );
  }
}

class _TaskCard extends StatelessWidget {
  const _TaskCard({
    required this.row,
    required this.onStop,
    required this.onDiscard,
    required this.onReview,
    required this.onReply,
  });

  final bridge.TaskView row;
  final VoidCallback onStop;
  final VoidCallback onDiscard;
  final VoidCallback onReview;
  final ValueChanged<String> onReply;

  String get _status => switch (row.phase) {
    'waiting' => 'Waiting',
    'running' => row.line.isEmpty ? 'Working…' : row.line,
    'asking' => row.detail,
    'ready' => '${row.changes} changes to review',
    'nothing' => 'Nothing to change',
    'failed' => row.detail,
    'applied' => 'Applied ${row.detail} changes',
    'undone' => 'Undone',
    _ => row.phase,
  };

  @override
  Widget build(BuildContext context) {
    final active = row.phase == 'running' || row.phase == 'waiting';
    return Card(
      margin: const EdgeInsets.only(top: KansoSpacing.sm),
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(row.goal, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: KansoSpacing.xxs),
            Text(_status),
            Text('${row.notesRead} notes read'),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: <Widget>[
                if (active)
                  TextButton(onPressed: onStop, child: const Text('Stop')),
                if (row.phase == 'ready' || row.phase == 'failed')
                  FilledButton(
                    onPressed: onReview,
                    child: const Text('Review'),
                  ),
                if (row.phase == 'asking')
                  TextButton(
                    onPressed: () => _ask(context),
                    child: const Text('Answer'),
                  ),
                if (!active)
                  TextButton(
                    onPressed: onDiscard,
                    child: const Text('Dismiss'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _ask(BuildContext context) async {
    final controller = TextEditingController();
    final text = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(row.detail),
        content: TextField(controller: controller, autofocus: true),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('Send'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (text != null && text.trim().isNotEmpty) {
      onReply(text);
    }
  }
}

/// The cards of one task, each with a tick, and Apply for the ticked ones.
class AgentReviewPage extends StatefulWidget {
  const AgentReviewPage({required this.tasks, required this.task, super.key});

  final bridge.KansoAgentTasks tasks;
  final bridge.TaskView task;

  @override
  State<AgentReviewPage> createState() => _AgentReviewPageState();
}

class _AgentReviewPageState extends State<AgentReviewPage> {
  bridge.TaskReview? _review;
  final Set<int> _ticked = <int>{};
  String? _error;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final review = await widget.tasks.review(id: widget.task.id);
      setState(() {
        _review = review;
        _ticked
          ..clear()
          ..addAll(review.cards.where((c) => c.ticked).map((c) => c.index));
      });
    } on Object catch (error) {
      setState(() => _error = '$error');
    }
  }

  Future<void> _apply() async {
    try {
      final text = await widget.tasks.apply(
        id: widget.task.id,
        ticked: _ticked.toList(),
        nowMs: DateTime.now().millisecondsSinceEpoch,
      );
      if (mounted) {
        Navigator.of(context).pop(text);
      }
    } on Object catch (error) {
      setState(() => _error = '$error');
    }
  }

  Color _lineColor(String line, BuildContext context) {
    if (line.startsWith('+ ')) {
      return Colors.green;
    }
    if (line.startsWith('- ')) {
      return Colors.red;
    }
    return context.colors.textSecondary;
  }

  @override
  Widget build(BuildContext context) {
    final review = _review;
    return Scaffold(
      appBar: AppBar(title: Text(review?.header ?? 'Review')),
      body: review == null
          ? Center(child: Text(_error ?? 'Loading…'))
          : ListView(
              padding: const EdgeInsets.all(KansoSpacing.md),
              children: <Widget>[
                if (review.fromAgent.isNotEmpty) Text(review.fromAgent),
                for (final warning in review.warnings) Text(warning),
                if (_error != null) Text(_error!),
                for (final card in review.cards)
                  Card(
                    child: CheckboxListTile(
                      value: _ticked.contains(card.index),
                      onChanged: (on) => setState(() {
                        if (on ?? false) {
                          _ticked.add(card.index);
                        } else {
                          _ticked.remove(card.index);
                        }
                      }),
                      title: Text('${card.kind}: ${card.title}'),
                      subtitle: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: <Widget>[
                          if (card.book.isNotEmpty) Text(card.book),
                          if (card.outside)
                            const Text('Not one of the notes you chose'),
                          if (card.changedSince)
                            const Text('This note changed since it was read'),
                          for (final line in card.lines.take(40))
                            Text(
                              line,
                              style: TextStyle(
                                color: _lineColor(line, context),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
      bottomNavigationBar: review == null
          ? null
          : SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: FilledButton(
                  onPressed: _ticked.isEmpty ? null : _apply,
                  child: Text('Apply ${_ticked.length}'),
                ),
              ),
            ),
    );
  }
}
