// Kanso — fixture_approvals_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/approvals_repository.dart';

/// A request ready to be answered, asked at 14:02 local time on 2026-10-26.
ApprovalPrompt readyPrompt({
  String id = 'a1',
  String? deviceName = 'iPad',
  String? code = '123 456',
  PromptState state = PromptState.ready,
}) {
  final asked = DateTime(2026, 10, 26, 14, 2);
  return ApprovalPrompt(
    id: id,
    deviceName: deviceName,
    askedAtMs: asked.millisecondsSinceEpoch,
    expiresAtMs: asked.add(const Duration(minutes: 10)).millisecondsSinceEpoch,
    code: code,
    state: state,
  );
}

/// An approvals service for tests: the requests it is told are waiting, and
/// the answers it was asked to give. An answer removes its request.
class FixtureApprovalsRepository implements ApprovalsRepository {
  FixtureApprovalsRepository([List<ApprovalPrompt>? waiting])
    : prompts = waiting ?? <ApprovalPrompt>[];

  /// The requests waiting.
  List<ApprovalPrompt> prompts;

  /// What an answer fails with, if it does.
  ApplicationError? failure;

  /// How often the requests were watched.
  int watches = 0;

  /// The answers given, as "approve id" and "decline id".
  final List<String> answers = <String>[];

  final StreamController<List<ApprovalPrompt>> _changes =
      StreamController<List<ApprovalPrompt>>.broadcast();

  /// Tells the watchers the requests are now [next].
  void change(List<ApprovalPrompt> next) {
    prompts = next;
    _changes.add(next);
  }

  @override
  Stream<List<ApprovalPrompt>> watch() {
    watches++;
    // Listening to the changes now, not when the first requests are taken, so
    // a change made right after cannot be missed.
    final watched = StreamController<List<ApprovalPrompt>>()..add(prompts);
    final changes = _changes.stream.listen(watched.add);
    watched.onCancel = changes.cancel;
    return watched.stream;
  }

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> refresh() async =>
      Right<ApplicationError, List<ApprovalPrompt>>(prompts);

  Either<ApplicationError, List<ApprovalPrompt>> _answer(String id) {
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, List<ApprovalPrompt>>(error);
    }
    prompts = <ApprovalPrompt>[
      for (final prompt in prompts)
        if (prompt.id != id) prompt,
    ];
    return Right<ApplicationError, List<ApprovalPrompt>>(prompts);
  }

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> approve(
    String id,
  ) async {
    answers.add('approve $id');
    return _answer(id);
  }

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> decline(
    String id,
  ) async {
    answers.add('decline $id');
    return _answer(id);
  }
}
