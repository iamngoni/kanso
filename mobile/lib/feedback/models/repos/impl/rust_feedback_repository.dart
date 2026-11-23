// Kanso — rust_feedback_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/feedback.dart' as bridge;
import '../../../../crash/models/repos/abs/crash_context_repository.dart';
import '../../../../crash/models/repos/abs/crash_report_repository.dart';
import '../../data/feedback_credit.dart';
import '../../data/feedback_draft.dart';
import '../../data/feedback_entry.dart';
import '../../data/feedback_kind.dart';
import '../../data/feedback_result.dart';
import '../abs/feedback_repository.dart';

/// [FeedbackRepository] over the Rust side that owns the rules, the outbox
/// and the retry. The app info it sends is the version and the system the
/// crash context already reads, and the type names of waiting crash reports;
/// nothing comes from a note.
@LazySingleton(as: FeedbackRepository)
class RustFeedbackRepository implements FeedbackRepository {
  RustFeedbackRepository(this._feedback, this._context, this._crashes);

  final bridge.KansoFeedback _feedback;
  final CrashContextRepository _context;
  final CrashReportRepository _crashes;

  Future<bridge.FeedbackInput> _input(
    FeedbackDraft draft,
    String screen,
  ) async {
    final context = await _context.read();
    final pending = await _crashes.pending();
    final types = pending.fold<List<String>>(
      (error) => const <String>[],
      (reports) => <String>[for (final report in reports) report.report.type],
    );
    return bridge.FeedbackInput(
      kind: draft.kind.wire,
      text: draft.text,
      credit: _credit(draft),
      includeApp: draft.includeApp,
      screenshot: draft.screenshot?.bytes,
      context: bridge.FeedbackContext(
        version: context.app.version,
        build: context.app.build,
        os: context.os.version,
        device: context.device,
        // The platform's locale tag, `en_GB` or `en-GB`.
        locale: _locale(),
        screen: screen,
        crashTypes: types,
      ),
    );
  }

  /// The credit the draft asks for, or null when it is off. A GitHub credit
  /// with nobody picked carries an empty login, which the core refuses with
  /// [FeedbackProblem.badGithubUser].
  static bridge.FeedbackCredit? _credit(FeedbackDraft draft) {
    if (!draft.credit) {
      return null;
    }
    return switch (draft.creditKind) {
      FeedbackCreditKind.name => bridge.FeedbackCredit.name(
        name: draft.creditName.trim(),
      ),
      FeedbackCreditKind.github => bridge.FeedbackCredit.github(
        login: draft.githubUser?.login ?? '',
      ),
    };
  }

  static String _locale() {
    final name = Platform.localeName;
    return name.split('.').first;
  }

  @override
  Future<FeedbackPayload> preview(
    FeedbackDraft draft, {
    String screen = 'settings',
  }) async {
    try {
      final shown = await _feedback.preview(
        input: await _input(draft, screen),
      );
      return FeedbackPayload(
        json: shown.json,
        problem: _problem(shown.problem),
      );
    } on Object {
      return const FeedbackPayload(problem: FeedbackProblem.other);
    }
  }

  @override
  Future<List<FeedbackGithubUser>?> searchGithubUsers(String query) async {
    try {
      final found = await _feedback.searchUsers(query: query);
      return switch (found) {
        bridge.FeedbackUsers_Found(:final users) => <FeedbackGithubUser>[
          for (final user in users)
            FeedbackGithubUser(login: user.login, avatarUrl: user.avatarUrl),
        ],
        bridge.FeedbackUsers_Unavailable() => null,
      };
    } on Object {
      return null;
    }
  }

  @override
  Future<FeedbackSendResult> send(
    FeedbackDraft draft, {
    String screen = 'settings',
  }) async {
    try {
      final sent = await _feedback.send(input: await _input(draft, screen));
      return switch (sent) {
        bridge.FeedbackSent_Sent() => const FeedbackSendResult(
          outcome: FeedbackOutcome.sent,
        ),
        bridge.FeedbackSent_Queued() => const FeedbackSendResult(
          outcome: FeedbackOutcome.queued,
        ),
        bridge.FeedbackSent_Paused() => const FeedbackSendResult(
          outcome: FeedbackOutcome.paused,
        ),
        bridge.FeedbackSent_RateLimited() => const FeedbackSendResult(
          outcome: FeedbackOutcome.rateLimited,
        ),
        bridge.FeedbackSent_Rejected(:final field) => FeedbackSendResult(
          outcome: FeedbackOutcome.rejected,
          field: field,
        ),
        bridge.FeedbackSent_Invalid(:final problem) => FeedbackSendResult(
          outcome: FeedbackOutcome.invalid,
          problem: _problem(problem),
        ),
      };
    } on Object {
      // The library could not even save it.
      return const FeedbackSendResult(
        outcome: FeedbackOutcome.invalid,
        problem: FeedbackProblem.other,
      );
    }
  }

  @override
  Future<List<FeedbackEntry>?> entries({bool refresh = false}) async {
    try {
      final reports = await _feedback.reports(refresh: refresh);
      return <FeedbackEntry>[for (final report in reports) _entry(report)];
    } on Object {
      return null;
    }
  }

  @override
  bool get refreshDue {
    try {
      return _feedback.refreshDue();
    } on Object {
      return false;
    }
  }

  @override
  Future<bool> delete(String id) async {
    try {
      await _feedback.delete(id: id);
      return true;
    } on Object {
      return false;
    }
  }

  @override
  Future<void> flush() async {
    try {
      await _feedback.retryNow();
      await _feedback.flush();
    } on Object {
      // Best effort: the reports stay on the phone and go next time.
    }
  }

  @override
  Future<void> markSeen() async {
    try {
      await _feedback.markSeen();
    } on Object {
      // Best effort.
    }
  }

  static FeedbackProblem? _problem(bridge.FeedbackProblem? problem) =>
      switch (problem) {
        null => null,
        bridge.FeedbackProblem.tooShort => FeedbackProblem.tooShort,
        bridge.FeedbackProblem.tooLong => FeedbackProblem.tooLong,
        bridge.FeedbackProblem.badCharacter => FeedbackProblem.badCharacter,
        bridge.FeedbackProblem.badCredit => FeedbackProblem.badCredit,
        bridge.FeedbackProblem.badGithubUser => FeedbackProblem.badGithubUser,
        bridge.FeedbackProblem.badScreenshot => FeedbackProblem.badScreenshot,
        bridge.FeedbackProblem.other => FeedbackProblem.other,
      };

  static FeedbackEntry _entry(bridge.FeedbackReport report) => FeedbackEntry(
    id: report.id,
    createdMs: report.createdMs,
    kind: FeedbackKind.fromWire(report.kind),
    preview: report.preview,
    chip: switch (report.chip) {
      bridge.FeedbackChipKind.waitingToSend => FeedbackChip.waitingToSend,
      bridge.FeedbackChipKind.received => FeedbackChip.received,
      bridge.FeedbackChipKind.filedAs => FeedbackChip.filedAs,
      bridge.FeedbackChipKind.building => FeedbackChip.building,
      bridge.FeedbackChipKind.fixProposed => FeedbackChip.fixProposed,
      bridge.FeedbackChipKind.merged => FeedbackChip.merged,
      bridge.FeedbackChipKind.releasedIn => FeedbackChip.releasedIn,
      bridge.FeedbackChipKind.notFiled => FeedbackChip.notFiled,
      bridge.FeedbackChipKind.sameAs => FeedbackChip.sameAs,
      bridge.FeedbackChipKind.closed => FeedbackChip.closed,
      bridge.FeedbackChipKind.deleting => FeedbackChip.deleting,
    },
    number: report.number,
    release: report.release,
    pr: report.pr,
    issueUrl: report.issueUrl,
    deleting: report.deleting,
    fresh: report.fresh,
    onServer: report.onServer,
  );
}
