// Kanso — fixture_notices_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/notices_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/notices_repository.dart';

/// The notices that apply to the phone, in memory: the newest first, as the
/// library orders them. What was dismissed or marked fixed is remembered the
/// way the library remembers it, and every call is recorded.
class FixtureNoticesRepository implements NoticesRepository {
  FixtureNoticesRepository({List<PhoneNotice>? applying})
    : applying = applying ?? <PhoneNotice>[exampleNotice];

  /// A made-up notice for tests and pictures. No real notice ships yet.
  static const PhoneNotice exampleNotice = PhoneNotice(
    id: 'fixture-example-0.9',
    title: 'Something you set up needs a new step',
    whatChanged:
        'This update changed how an old setup is read, so anything that used '
        'it stops working until it is set up again. Your notes are untouched.',
    steps: <String>[
      'Open Settings.',
      'Set up each program that used the old setup again.',
      'Restart those programs.',
    ],
    actionKind: PhoneNoticeActionKind.openSettings,
    actionLabel: 'Open Settings',
    actionTarget: 'account',
  );

  /// Every notice that applies, newest first.
  final List<PhoneNotice> applying;
  final Set<String> dismissed = <String>{};
  final Set<String> fixed = <String>{};

  /// What was asked, in order: `notices:in`, `notices:out`, `dismiss:<id>`,
  /// `fixed:<id>`.
  final List<String> calls = <String>[];

  /// When set, every call fails with it.
  ApplicationError? failure;

  /// The notices not put away.
  List<PhoneNotice> get waiting => <PhoneNotice>[
    for (final notice in applying)
      if (!dismissed.contains(notice.id) && !fixed.contains(notice.id)) notice,
  ];

  @override
  Future<Either<ApplicationError, PhoneNotices>> notices({
    required bool signedIn,
  }) async {
    calls.add(signedIn ? 'notices:in' : 'notices:out');
    final error = failure;
    if (error != null) {
      return left(error);
    }
    final shown = waiting;
    return right(
      PhoneNotices(
        card: shown.isEmpty ? null : shown.first,
        total: shown.length,
        countLabel: shown.length > 1 ? '1 of ${shown.length}' : '',
      ),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> dismiss(String id) async {
    calls.add('dismiss:$id');
    final error = failure;
    if (error != null) {
      return left(error);
    }
    dismissed.add(id);
    return right(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> markFixed(String id) async {
    calls.add('fixed:$id');
    final error = failure;
    if (error != null) {
      return left(error);
    }
    fixed.add(id);
    return right(unit);
  }
}
