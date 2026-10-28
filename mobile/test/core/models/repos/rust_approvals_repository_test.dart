// Kanso — rust_approvals_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/approvals.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_approvals_repository.dart';
import 'package:mocktail/mocktail.dart';

class _MockApprovals extends Mock implements KansoApprovals {}

const ApprovalPrompt _prompt = ApprovalPrompt(
  id: 'a1',
  deviceName: 'iPad',
  askedAtMs: 1,
  expiresAtMs: 2,
  code: '123 456',
  state: PromptState.ready,
);

void main() {
  late _MockApprovals approvals;
  late RustApprovalsRepository repository;

  setUp(() {
    approvals = _MockApprovals();
    repository = RustApprovalsRepository(approvals);
  });

  group('RustApprovalsRepository', () {
    test('the requests are read, and watched as they change', () async {
      when(
        approvals.refresh,
      ).thenAnswer((_) async => <ApprovalPrompt>[_prompt]);
      when(approvals.watch).thenAnswer(
        (_) => Stream<List<ApprovalPrompt>>.fromIterable(
          <List<ApprovalPrompt>>[
            <ApprovalPrompt>[],
            <ApprovalPrompt>[_prompt],
          ],
        ),
      );
      final read = await repository.refresh();
      expect(read.isRight(), isTrue);
      expect(read.getOrElse(() => <ApprovalPrompt>[]), <ApprovalPrompt>[
        _prompt,
      ]);
      expect(await repository.watch().toList(), <List<ApprovalPrompt>>[
        <ApprovalPrompt>[],
        <ApprovalPrompt>[_prompt],
      ]);
    });

    test('an answer gives the requests that are left', () async {
      when(
        () => approvals.approve(id: 'a1'),
      ).thenAnswer((_) async => <ApprovalPrompt>[]);
      when(
        () => approvals.decline(id: 'a2'),
      ).thenAnswer((_) async => <ApprovalPrompt>[_prompt]);
      final approved = await repository.approve('a1');
      final declined = await repository.decline('a2');
      expect(approved.getOrElse(() => <ApprovalPrompt>[_prompt]), isEmpty);
      expect(declined.getOrElse(() => <ApprovalPrompt>[]), <ApprovalPrompt>[
        _prompt,
      ]);
    });

    test('a request that changed is the library’s error', () async {
      when(() => approvals.approve(id: 'a1')).thenAnswer(
        (_) => Future<List<ApprovalPrompt>>.error(
          const KansoError.invalid(
            message: 'This request changed after it was made. Don’t approve.',
          ),
        ),
      );
      final result = await repository.approve('a1');
      expect(result.swap().toOption().toNullable(), isA<InvalidError>());
    });
  });
}
