// Kanso — fixture_feedback_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/feedback/models/data/feedback_draft.dart';
import 'package:kanso/feedback/models/data/feedback_entry.dart';
import 'package:kanso/feedback/models/data/feedback_kind.dart';
import 'package:kanso/feedback/models/data/feedback_result.dart';
import 'package:kanso/feedback/models/repos/abs/feedback_repository.dart';
import 'package:kanso/feedback/models/repos/abs/feedback_screenshot_repository.dart';

/// A feedback outbox in memory: sends are answered with [result], and every
/// call is recorded. It never reaches the Rust core or the network.
class FixtureFeedbackRepository implements FeedbackRepository {
  FixtureFeedbackRepository({
    List<FeedbackEntry> entries = const <FeedbackEntry>[],
    this.result = const FeedbackSendResult(outcome: FeedbackOutcome.sent),
    this.due = false,
  }) : _entries = List<FeedbackEntry>.of(entries);

  List<FeedbackEntry> _entries;

  /// What a send answers.
  FeedbackSendResult result;

  /// What [refreshDue] answers.
  bool due;

  final List<FeedbackDraft> sent = <FeedbackDraft>[];
  final List<String> deleted = <String>[];
  final List<bool> refreshes = <bool>[];
  int flushes = 0;
  int seen = 0;

  @override
  Future<FeedbackPayload> preview(
    FeedbackDraft draft, {
    String screen = 'settings',
  }) async => draft.length < feedbackMinText
      ? const FeedbackPayload(problem: FeedbackProblem.tooShort)
      : FeedbackPayload(
          json: '{"v":1,"kind":"${draft.kind.wire}","text":"${draft.text}"}',
        );

  @override
  Future<FeedbackSendResult> send(
    FeedbackDraft draft, {
    String screen = 'settings',
  }) async {
    sent.add(draft);
    return result;
  }

  @override
  Future<List<FeedbackEntry>?> entries({bool refresh = false}) async {
    refreshes.add(refresh);
    return _entries;
  }

  @override
  bool get refreshDue => due;

  @override
  Future<bool> delete(String id) async {
    deleted.add(id);
    _entries = <FeedbackEntry>[
      for (final entry in _entries)
        if (entry.id != id) entry,
    ];
    return true;
  }

  @override
  Future<void> flush() async => flushes += 1;

  @override
  Future<void> markSeen() async => seen += 1;
}

/// A photo picker that answers with [picked].
class FixtureFeedbackScreenshots implements FeedbackScreenshotRepository {
  FixtureFeedbackScreenshots([this.picked]);

  FeedbackScreenshot? picked;
  int picks = 0;

  @override
  Future<FeedbackScreenshot?> pick() async {
    picks += 1;
    return picked;
  }
}

/// A report for tests.
FeedbackEntry sampleFeedbackEntry({
  String id = 'fb_a',
  String preview = 'Export to PDF cuts off tables',
  FeedbackChip chip = FeedbackChip.received,
  int createdMs = 1730650000000,
  int? number,
  String? release,
  bool fresh = false,
  bool deleting = false,
  String? issueUrl,
}) => FeedbackEntry(
  id: id,
  createdMs: createdMs,
  kind: FeedbackKind.bug,
  preview: preview,
  chip: chip,
  number: number,
  release: release,
  fresh: fresh,
  deleting: deleting,
  issueUrl: issueUrl,
);
