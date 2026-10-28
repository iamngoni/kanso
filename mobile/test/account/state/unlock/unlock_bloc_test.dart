// Kanso — unlock_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/account_outcome.dart';
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/state/unlock/unlock_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const String _password = 'correct horse battery';
const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: 'https://sync.example.invalid',
);

ApprovalWait _wait(ApprovalPhase phase, {String? code, bool recovery = false}) {
  return ApprovalWait(
    phase: phase,
    code: code,
    otherDevices: 1,
    offerRecovery: recovery,
  );
}

void main() {
  late MockAccountRepository account;

  setUp(() {
    account = MockAccountRepository();
    when(account.cancelApproval).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
  });

  group('UnlockBloc, the password', () {
    test(
      'a password no account can have is answered like a wrong one',
      () async {
        final bloc = UnlockBloc(account)
          ..add(
            const UnlockEvent.passwordSubmitted(
              username: 'ana',
              password: 'short',
            ),
          );
        await pumpEventQueue();
        expect(bloc.state.problem, FormProblem.credentialsDontMatch);
        verifyNever(() => account.unlockWithPassword(any()));
        await bloc.close();
      },
    );

    test('the right password opens the notes', () async {
      final gate = Completer<void>();
      when(() => account.unlockWithPassword(_password)).thenAnswer((_) async {
        await gate.future;
        return const Right<ApplicationError, AccountState>(_signedIn);
      });
      final bloc = UnlockBloc(account)
        ..add(
          const UnlockEvent.passwordSubmitted(
            username: 'ana',
            password: _password,
          ),
        );
      await bloc.stream.firstWhere((state) => state.busy);
      gate.complete();
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });

    test('a wrong password is the error, and the screen stays', () async {
      when(() => account.unlockWithPassword(_password)).thenAnswer(
        (_) async => const Left<ApplicationError, AccountState>(
          ApplicationError.auth(
            message: 'That username and password don’t match.',
          ),
        ),
      );
      final bloc = UnlockBloc(account)
        ..add(
          const UnlockEvent.passwordSubmitted(
            username: 'ana',
            password: _password,
          ),
        );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.busy, isFalse);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });

    test(
      'notes of another account ask the question, and an answer goes on',
      () async {
        when(() => account.unlockWithPassword(_password)).thenAnswer(
          (_) async => const Right<ApplicationError, AccountState>(
            AccountState.question(
              BindingQuestion(
                title: 'This device already has notes',
                body: '3 notes on this device came from another account.',
                answers: <BindingAnswer>[BindingAnswer.add],
              ),
            ),
          ),
        );
        when(() => account.answerBinding(BindingAnswer.add)).thenAnswer(
          (_) async => const Right<ApplicationError, AccountState>(_signedIn),
        );
        final bloc = UnlockBloc(account)
          ..add(
            const UnlockEvent.passwordSubmitted(
              username: 'ana',
              password: _password,
            ),
          );
        await bloc.stream.firstWhere((state) => state.question != null);
        bloc.add(const UnlockEvent.bindingAnswered(BindingAnswer.add));
        await bloc.stream.firstWhere((state) => state.outcome != null);
        expect(bloc.state.outcome, AccountOutcome.signedIn);
        expect(bloc.state.question, isNull);
        await bloc.close();
      },
    );
  });

  group('UnlockBloc, another device’s approval', () {
    late StreamController<ApprovalWait> waits;

    setUp(() {
      waits = StreamController<ApprovalWait>();
      when(
        () => account.requestApproval('iPhone'),
      ).thenAnswer((_) => waits.stream);
    });

    test('the wait goes through its phases and ends approved', () async {
      when(account.state).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(_signedIn),
      );
      final bloc = UnlockBloc(account)
        ..add(const UnlockEvent.approvalRequested(deviceName: 'iPhone'));
      await bloc.stream.firstWhere((state) => state.approval != null);
      expect(bloc.state.approval?.phase, const ApprovalPhase.requesting());

      waits.add(_wait(const ApprovalPhase.waiting()));
      await bloc.stream.firstWhere(
        (state) => state.approval?.phase is ApprovalPhase_Waiting,
      );
      waits.add(_wait(const ApprovalPhase.compare(), code: '123 456'));
      await bloc.stream.firstWhere((state) => state.approval?.code != null);
      expect(bloc.state.approval?.code, '123 456');

      final unlocked = bloc.stream.firstWhere((state) => state.outcome != null);
      waits
        ..add(_wait(const ApprovalPhase.unlocking(), code: '123 456'))
        ..add(_wait(const ApprovalPhase.approved(), code: '123 456'));
      unawaited(waits.close());
      await unlocked;
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.approval, isNull);
      await bloc.close();
    });

    test('a wait that expired stays on screen, to be tried again', () async {
      final bloc = UnlockBloc(account)
        ..add(const UnlockEvent.approvalRequested(deviceName: 'iPhone'));
      final expired = bloc.stream.firstWhere(
        (state) => state.approval?.phase is ApprovalPhase_Expired,
      );
      waits.add(_wait(const ApprovalPhase.expired(), recovery: true));
      await expired;
      unawaited(waits.close());
      await pumpEventQueue();
      expect(bloc.state.approval?.offerRecovery, isTrue);
      expect(bloc.state.outcome, isNull);
      verifyNever(account.state);

      // Trying again asks again.
      final again = StreamController<ApprovalWait>();
      when(
        () => account.requestApproval('iPhone'),
      ).thenAnswer((_) => again.stream);
      bloc.add(const UnlockEvent.approvalRequested(deviceName: 'iPhone'));
      await bloc.stream.firstWhere(
        (state) => state.approval?.phase is ApprovalPhase_Requesting,
      );
      unawaited(again.close());
      await bloc.close();
    });

    test('a request that failed is told in its words', () async {
      final bloc = UnlockBloc(account)
        ..add(const UnlockEvent.approvalRequested(deviceName: 'iPhone'));
      final failed = bloc.stream.firstWhere(
        (state) => state.approval?.phase is ApprovalPhase_Failed,
      );
      waits.add(
        _wait(const ApprovalPhase.failed(message: 'Can’t reach the server.')),
      );
      await failed;
      unawaited(waits.close());
      await bloc.close();
    });

    test('Cancel ends the request and goes back to the password', () async {
      final bloc = UnlockBloc(account)
        ..add(const UnlockEvent.approvalRequested(deviceName: 'iPhone'));
      await bloc.stream.firstWhere((state) => state.approval != null);
      bloc.add(const UnlockEvent.approvalCancelled());
      await bloc.stream.firstWhere((state) => state.approval == null);
      verify(account.cancelApproval).called(1);

      // What the library still says of the cancelled request is not shown.
      waits.add(_wait(const ApprovalPhase.cancelled()));
      await pumpEventQueue();
      expect(bloc.state.approval, isNull);
      unawaited(waits.close());
      await bloc.close();
    });

    test(
      'a request the phone made earlier is picked up where it was',
      () async {
        final bloc = UnlockBloc(account)
          ..add(
            const UnlockEvent.started(
              resumeApproval: true,
              deviceName: 'iPhone',
            ),
          );
        await bloc.stream.firstWhere((state) => state.approval != null);
        verify(() => account.requestApproval('iPhone')).called(1);
        unawaited(waits.close());
        await bloc.close();
      },
    );

    test('with no earlier request the screen starts on the password', () async {
      final bloc = UnlockBloc(account)
        ..add(
          const UnlockEvent.started(
            resumeApproval: false,
            deviceName: 'iPhone',
          ),
        );
      await pumpEventQueue();
      expect(bloc.state.approval, isNull);
      verifyNever(() => account.requestApproval(any()));
      unawaited(waits.close());
      await bloc.close();
    });
  });

  group('UnlockBloc, signing out', () {
    test('ends the locked sign-in', () async {
      when(account.cancelLocked).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(
          AccountState.signedOut(),
        ),
      );
      final bloc = UnlockBloc(account)..add(const UnlockEvent.signedOut());
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedOut);
      await bloc.close();
    });

    test('a failure is the error', () async {
      when(account.cancelLocked).thenAnswer(
        (_) async => const Left<ApplicationError, AccountState>(
          ApplicationError.auth(message: 'The secret store refused.'),
        ),
      );
      final bloc = UnlockBloc(account)..add(const UnlockEvent.signedOut());
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });
  });
}
