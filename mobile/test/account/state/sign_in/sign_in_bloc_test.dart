// Kanso — sign_in_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/account_outcome.dart';
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/models/data/provider_code.dart';
import 'package:kanso/account/models/data/sign_in_action.dart';
import 'package:kanso/account/models/data/sign_in_mode.dart';
import 'package:kanso/account/state/sign_in/sign_in_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/mock_repositories.dart';

const String _server = 'https://sync.example.invalid';
const String _device = 'iPhone';
const String _password = 'correct horse battery';
const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: _server,
);
const AccountState _locked = AccountState.locked(
  LockedInfo(
    username: 'ana',
    server: _server,
    hasPassword: true,
    approvalPending: false,
  ),
);
const BindingQuestion _question = BindingQuestion(
  title: 'This device already has notes',
  body: '3 notes on this device came from another account.',
  answers: <BindingAnswer>[BindingAnswer.add, BindingAnswer.replace],
);
const ApplicationError _badCredentials = ApplicationError.auth(
  message: 'That username and password don’t match.',
);

void main() {
  late MockAccountRepository account;
  late MockSyncRepository sync;
  late MockAuthSessionRepository session;

  setUp(() {
    account = MockAccountRepository();
    sync = MockSyncRepository();
    session = MockAuthSessionRepository();
    when(
      sync.defaultServer,
    ).thenAnswer((_) async => const Right<ApplicationError, String>(_server));
    when(() => account.serverMethods(_server)).thenAnswer(
      (_) async => const Right<ApplicationError, ServerMethods>(allMethods),
    );
    when(session.close).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(
      () => session.open(any(), callbackScheme: any(named: 'callbackScheme')),
    ).thenAnswer((_) async => const Right<ApplicationError, String?>(null));
    when(() => account.providerPageEnded(any())).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(account.cancelProvider).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
  });

  SignInBloc make() => SignInBloc(account, sync, session);

  Future<SignInBloc> started() async {
    final bloc = make()..add(const SignInEvent.started());
    await bloc.stream.firstWhere(
      (state) => state.methodsStatus == LoadStatus.ready,
    );
    return bloc;
  }

  group('SignInBloc, the server', () {
    blocTest<SignInBloc, SignInState>(
      'starts on the default server and asks what it offers',
      build: make,
      act: (bloc) => bloc.add(const SignInEvent.started()),
      expect: () => <SignInState>[
        const SignInState(server: _server, defaultServer: _server),
        const SignInState(
          server: _server,
          defaultServer: _server,
          methodsStatus: LoadStatus.ready,
          methods: allMethods,
        ),
      ],
      verify: (_) => verify(() => account.serverMethods(_server)).called(1),
    );

    blocTest<SignInBloc, SignInState>(
      'a server that does not answer is failed, with why',
      build: () {
        when(() => account.serverMethods(_server)).thenAnswer(
          (_) async => const Left<ApplicationError, ServerMethods>(
            ApplicationError.network(
              message:
                  'Can’t reach the server. Your notes are safe on this '
                  'device.',
            ),
          ),
        );
        return make();
      },
      act: (bloc) => bloc.add(const SignInEvent.started()),
      verify: (bloc) {
        expect(bloc.state.methodsStatus, LoadStatus.failed);
        expect(bloc.state.methods, isNull);
        expect(bloc.state.error, isA<NetworkError>());
      },
    );

    test(
      'another server is asked, and used; an empty one is the default',
      () async {
        when(
          () => account.serverMethods('https://other.example.invalid'),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, ServerMethods>(
            ServerMethods(
              signupsOpen: false,
              password: true,
              passkeys: false,
              providers: <ProviderKind>[],
            ),
          ),
        );
        final bloc = await started();
        bloc.add(const SignInEvent.serverEdited(editing: true));
        await pumpEventQueue();
        expect(bloc.state.editingServer, isTrue);

        bloc.add(
          const SignInEvent.serverChosen('  https://other.example.invalid '),
        );
        await bloc.stream.firstWhere(
          (state) =>
              state.methodsStatus == LoadStatus.ready &&
              state.server == 'https://other.example.invalid' &&
              state.methods?.signupsOpen == false,
        );
        expect(bloc.state.server, 'https://other.example.invalid');
        expect(bloc.state.customServer, isTrue);
        expect(bloc.state.editingServer, isFalse);
        expect(bloc.state.methods?.passkeys, isFalse);

        bloc.add(const SignInEvent.serverChosen(''));
        await bloc.stream.firstWhere(
          (state) =>
              state.methodsStatus == LoadStatus.ready &&
              state.server == _server &&
              (state.methods?.passkeys ?? false),
        );
        expect(bloc.state.customServer, isFalse);
        await bloc.close();
      },
    );

    test('the mode is changed, and what was wrong is forgotten', () async {
      final bloc = await started();
      bloc.add(
        const SignInEvent.passwordSubmitted(
          username: 'x',
          password: 'y',
          deviceName: _device,
        ),
      );
      await pumpEventQueue();
      expect(bloc.state.problem, FormProblem.usernameInvalid);

      bloc.add(const SignInEvent.modeChanged(SignInMode.create));
      await pumpEventQueue();
      expect(bloc.state.mode, SignInMode.create);
      expect(bloc.state.problem, isNull);
      await bloc.close();
    });
  });

  group('SignInBloc, a username and password', () {
    test(
      'a username that breaks the rules is told at once, nothing sent',
      () async {
        final bloc = await started();
        bloc.add(
          const SignInEvent.passwordSubmitted(
            username: 'a b',
            password: _password,
            deviceName: _device,
          ),
        );
        await pumpEventQueue();
        expect(bloc.state.problem, FormProblem.usernameInvalid);
        expect(bloc.state.busy, isNull);
        verifyNever(
          () => account.signInPassword(
            server: any(named: 'server'),
            username: any(named: 'username'),
            password: any(named: 'password'),
            deviceName: any(named: 'deviceName'),
          ),
        );
        await bloc.close();
      },
    );

    test(
      'a password no account can have is answered like a wrong one',
      () async {
        final bloc = await started();
        bloc.add(
          const SignInEvent.passwordSubmitted(
            username: 'ana',
            password: 'short',
            deviceName: _device,
          ),
        );
        await pumpEventQueue();
        expect(bloc.state.problem, FormProblem.credentialsDontMatch);
        await bloc.close();
      },
    );

    test('signing in is busy, then ends the flow signed in', () async {
      final gate = Completer<void>();
      when(
        () => account.signInPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer((_) async {
        await gate.future;
        return const Right<ApplicationError, AccountState>(_signedIn);
      });
      final bloc = await started();
      bloc.add(
        const SignInEvent.passwordSubmitted(
          username: ' Ana ',
          password: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere(
        (state) => state.busy == SignInAction.password,
      );
      gate.complete();
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.busy, isNull);
      expect(bloc.state.recoveryKey, isNull);
      await bloc.close();
    });

    test('a refusal is the error, in the library’s words', () async {
      when(
        () => account.signInPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async =>
            const Left<ApplicationError, AccountState>(_badCredentials),
      );
      final bloc = await started();
      bloc.add(
        const SignInEvent.passwordSubmitted(
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.error, _badCredentials);
      expect(bloc.state.busy, isNull);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });

    test(
      'an account whose notes this phone cannot open yet is locked',
      () async {
        when(
          () => account.signInPassword(
            server: _server,
            username: 'ana',
            password: _password,
            deviceName: _device,
          ),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, AccountState>(_locked),
        );
        final bloc = await started();
        bloc.add(
          const SignInEvent.passwordSubmitted(
            username: 'ana',
            password: _password,
            deviceName: _device,
          ),
        );
        await bloc.stream.firstWhere((state) => state.outcome != null);
        expect(bloc.state.outcome, AccountOutcome.locked);
        await bloc.close();
      },
    );

    test(
      'notes of another account ask the question, and the answer goes on',
      () async {
        when(
          () => account.signInPassword(
            server: _server,
            username: 'ana',
            password: _password,
            deviceName: _device,
          ),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, AccountState>(
            AccountState.question(_question),
          ),
        );
        when(() => account.answerBinding(BindingAnswer.replace)).thenAnswer(
          (_) async => const Right<ApplicationError, AccountState>(_signedIn),
        );
        final bloc = await started();
        bloc.add(
          const SignInEvent.passwordSubmitted(
            username: 'ana',
            password: _password,
            deviceName: _device,
          ),
        );
        await bloc.stream.firstWhere((state) => state.question != null);
        expect(bloc.state.busy, isNull);
        expect(bloc.state.outcome, isNull);

        bloc.add(const SignInEvent.bindingAnswered(BindingAnswer.replace));
        await bloc.stream.firstWhere((state) => state.outcome != null);
        expect(bloc.state.outcome, AccountOutcome.signedIn);
        expect(bloc.state.question, isNull);
        // A sign-in makes no recovery key.
        verifyNever(account.pendingRecoveryKey);
        await bloc.close();
      },
    );

    test('cancelling the question leaves the phone signed out', () async {
      when(
        () => account.signInPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(
          AccountState.question(_question),
        ),
      );
      when(() => account.answerBinding(null)).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(
          AccountState.signedOut(),
        ),
      );
      final bloc = await started();
      bloc.add(
        const SignInEvent.passwordSubmitted(
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.question != null);
      bloc.add(const SignInEvent.bindingAnswered(null));
      await bloc.stream.firstWhere(
        (state) => state.question == null && state.busy == null,
      );
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });
  });

  group('SignInBloc, making an account', () {
    test(
      'the form is checked in the desktop’s order before anything is sent',
      () async {
        final bloc = await started();
        Future<FormProblem?> problemOf(
          String username,
          String password,
          String repeat,
        ) async {
          bloc.add(
            SignInEvent.signUpSubmitted(
              username: username,
              password: password,
              repeat: repeat,
              deviceName: _device,
            ),
          );
          await pumpEventQueue();
          return bloc.state.problem;
        }

        expect(
          await problemOf('a', _password, _password),
          FormProblem.usernameInvalid,
        );
        expect(
          await problemOf('admin', _password, _password),
          FormProblem.usernameReserved,
        );
        expect(
          await problemOf('ana', 'short', 'short'),
          FormProblem.passwordTooShort,
        );
        expect(
          await problemOf('ana', 'x' * 257, 'x' * 257),
          FormProblem.passwordTooLong,
        );
        expect(
          await problemOf('longname1234', 'longname1234', 'longname1234'),
          FormProblem.passwordSameAsUsername,
        );
        expect(
          await problemOf('ana', _password, '${_password}x'),
          FormProblem.passwordsDiffer,
        );
        verifyNever(
          () => account.signUpPassword(
            server: any(named: 'server'),
            username: any(named: 'username'),
            password: any(named: 'password'),
            deviceName: any(named: 'deviceName'),
          ),
        );
        await bloc.close();
      },
    );

    test('a new account ends with its recovery key to write down', () async {
      when(
        () => account.signUpPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(state: _signedIn, recoveryKey: sampleRecoveryKey),
        ),
      );
      final bloc = await started();
      bloc.add(
        const SignInEvent.signUpSubmitted(
          username: 'ana',
          password: _password,
          repeat: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      expect(bloc.state.creating, isTrue);
      await bloc.close();
    });

    test('over this phone’s own notes the question comes first, and the key '
        'after the answer', () async {
      when(
        () => account.signUpPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(
            state: AccountState.question(
              BindingQuestion(
                title: 'Upload to the new account?',
                body: 'This device’s notes came from another account.',
                answers: <BindingAnswer>[BindingAnswer.add],
              ),
            ),
            recoveryKey: RecoveryKeyText(key: ''),
          ),
        ),
      );
      when(() => account.answerBinding(BindingAnswer.add)).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(_signedIn),
      );
      when(account.pendingRecoveryKey).thenAnswer(
        (_) async => const Right<ApplicationError, RecoveryKeyText?>(
          sampleRecoveryKey,
        ),
      );
      final bloc = await started();
      bloc.add(
        const SignInEvent.signUpSubmitted(
          username: 'ana',
          password: _password,
          repeat: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.question != null);
      expect(bloc.state.recoveryKey, isNull);
      expect(bloc.state.question?.answers, <BindingAnswer>[BindingAnswer.add]);

      bloc.add(const SignInEvent.bindingAnswered(BindingAnswer.add));
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      await bloc.close();
    });

    test('a username taken is the error', () async {
      when(
        () => account.signUpPassword(
          server: _server,
          username: 'ana',
          password: _password,
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, SignedUp>(
          ApplicationError.invalid(message: 'That username is taken.'),
        ),
      );
      final bloc = await started();
      bloc.add(
        const SignInEvent.signUpSubmitted(
          username: 'ana',
          password: _password,
          repeat: _password,
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.busy, isNull);
      expect(bloc.state.creating, isFalse);
      await bloc.close();
    });
  });

  group('SignInBloc, passkeys', () {
    test('a sheet the person dismissed is not an error', () async {
      when(
        () => account.passkeySignIn(server: _server, deviceName: _device),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(
          AccountState.signedOut(),
        ),
      );
      final bloc = await started();
      bloc.add(const SignInEvent.passkeySignInPressed(deviceName: _device));
      await bloc.stream.firstWhere((state) => state.busy != null);
      await bloc.stream.firstWhere((state) => state.busy == null);
      expect(bloc.state.error, isNull);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });

    test('a passkey that signs in ends the flow', () async {
      when(
        () => account.passkeySignIn(server: _server, deviceName: _device),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(_locked),
      );
      final bloc = await started();
      bloc.add(const SignInEvent.passkeySignInPressed(deviceName: _device));
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.locked);
      await bloc.close();
    });

    test('a passkey that fails is the library’s words', () async {
      when(
        () => account.passkeySignIn(server: _server, deviceName: _device),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, AccountState>(
          ApplicationError.auth(message: 'The passkey didn’t work. Try again.'),
        ),
      );
      final bloc = await started();
      bloc.add(const SignInEvent.passkeySignInPressed(deviceName: _device));
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.busy, isNull);
      await bloc.close();
    });

    test('a passkey account needs a username that fits first', () async {
      final bloc = await started();
      bloc.add(
        const SignInEvent.passkeySignUpPressed(
          username: 'Ä',
          deviceName: _device,
        ),
      );
      await pumpEventQueue();
      expect(bloc.state.problem, FormProblem.usernameInvalid);
      verifyNever(
        () => account.passkeySignUp(
          server: any(named: 'server'),
          username: any(named: 'username'),
          deviceName: any(named: 'deviceName'),
        ),
      );

      when(
        () => account.passkeySignUp(
          server: _server,
          username: 'ana',
          deviceName: _device,
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(state: _signedIn, recoveryKey: sampleRecoveryKey),
        ),
      );
      bloc.add(
        const SignInEvent.passkeySignUpPressed(
          username: 'ana',
          deviceName: _device,
        ),
      );
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      await bloc.close();
    });
  });

  group('SignInBloc, a provider', () {
    late StreamController<ProviderEvent> steps;

    setUp(() {
      steps = StreamController<ProviderEvent>();
      when(
        () => account.startProvider(
          server: _server,
          provider: ProviderKind.apple,
          deviceName: _device,
        ),
      ).thenAnswer((_) => steps.stream);
      when(
        () => account.startProvider(
          server: _server,
          provider: ProviderKind.github,
          deviceName: _device,
        ),
      ).thenAnswer((_) => steps.stream);
    });

    tearDown(() async {
      if (!steps.isClosed) {
        unawaited(steps.close());
      }
    });

    test(
      'Apple’s page is opened, and its callback goes back to the library',
      () async {
        when(
          () => session.open(
            'https://appleid.example.invalid/auth',
            callbackScheme: 'kanso',
          ),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, String?>(
            'kanso://auth/apple?code=abc',
          ),
        );
        final bloc = await started();
        bloc.add(
          const SignInEvent.providerPressed(
            provider: ProviderKind.apple,
            deviceName: _device,
          ),
        );
        await bloc.stream.firstWhere(
          (state) => state.busy == SignInAction.apple,
        );
        steps.add(
          const ProviderEvent.openPage(
            url: 'https://appleid.example.invalid/auth',
            callbackScheme: 'kanso',
          ),
        );
        await pumpEventQueue();
        verify(
          () => account.providerPageEnded('kanso://auth/apple?code=abc'),
        ).called(1);

        steps
          ..add(const ProviderEvent.closePage())
          ..add(const ProviderEvent.finished(_signedIn));
        await bloc.stream.firstWhere((state) => state.outcome != null);
        expect(bloc.state.outcome, AccountOutcome.signedIn);
        expect(bloc.state.busy, isNull);
        unawaited(steps.close());
        await bloc.close();
      },
    );

    test(
      'a page the flow closed is not the person turning away from it',
      () async {
        final ended = Completer<Either<ApplicationError, String?>>();
        when(
          () => session.open(
            'http://127.0.0.1:1234/start',
            callbackScheme: any(named: 'callbackScheme'),
          ),
        ).thenAnswer((_) => ended.future);
        final bloc = await started();
        bloc.add(
          const SignInEvent.providerPressed(
            provider: ProviderKind.apple,
            deviceName: _device,
          ),
        );
        steps.add(
          const ProviderEvent.openPage(url: 'http://127.0.0.1:1234/start'),
        );
        await pumpEventQueue();
        // The flow has what it needs: the page is closed, and the session’s
        // open call completes with null because the app closed it.
        steps.add(const ProviderEvent.closePage());
        await pumpEventQueue();
        verify(session.close).called(greaterThanOrEqualTo(1));
        ended.complete(const Right<ApplicationError, String?>(null));
        steps.add(const ProviderEvent.finished(_signedIn));
        await bloc.stream.firstWhere((state) => state.outcome != null);
        await pumpEventQueue();
        verifyNever(() => account.providerPageEnded(any()));
        unawaited(steps.close());
        await bloc.close();
      },
    );

    test(
      'a page the person dismissed while the flow needs it cancels it',
      () async {
        when(
          () => session.open(
            'http://127.0.0.1:1234/start',
            callbackScheme: any(named: 'callbackScheme'),
          ),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, String?>(null),
        );
        final bloc = await started();
        bloc.add(
          const SignInEvent.providerPressed(
            provider: ProviderKind.apple,
            deviceName: _device,
          ),
        );
        steps.add(
          const ProviderEvent.openPage(url: 'http://127.0.0.1:1234/start'),
        );
        await pumpEventQueue();
        verify(() => account.providerPageEnded(null)).called(1);

        steps.add(const ProviderEvent.cancelled());
        await bloc.stream.firstWhere((state) => state.busy == null);
        expect(bloc.state.error, isNull);
        expect(bloc.state.outcome, isNull);
        unawaited(steps.close());
        await bloc.close();
      },
    );

    test(
      'GitHub shows its code, and a new identity is asked for a username',
      () async {
        when(
          () =>
              session.open(any(), callbackScheme: any(named: 'callbackScheme')),
        ).thenAnswer((_) async {
          // The page stays open until the app closes it.
          await Completer<void>().future;
          return const Right<ApplicationError, String?>(null);
        });
        when(() => account.finishProviderSignUp('ana')).thenAnswer(
          (_) async => const Right<ApplicationError, SignedUp>(
            SignedUp(state: _signedIn, recoveryKey: sampleRecoveryKey),
          ),
        );
        final bloc = await started();
        bloc.add(
          const SignInEvent.providerPressed(
            provider: ProviderKind.github,
            deviceName: _device,
          ),
        );
        steps.add(
          const ProviderEvent.showCode(
            userCode: 'ABCD-1234',
            verificationUri: 'https://github.example.invalid/login/device',
          ),
        );
        await bloc.stream.firstWhere((state) => state.code != null);
        expect(
          bloc.state.code,
          const ProviderCode(
            userCode: 'ABCD-1234',
            verificationUri: 'https://github.example.invalid/login/device',
          ),
        );

        bloc.add(const SignInEvent.githubOpened());
        await pumpEventQueue();
        verify(
          () => session.open('https://github.example.invalid/login/device'),
        ).called(1);

        steps.add(const ProviderEvent.needsUsername(suggested: 'ana'));
        await bloc.stream.firstWhere((state) => state.askingUsername);
        expect(bloc.state.suggestedUsername, 'ana');
        expect(bloc.state.code, isNull);
        expect(bloc.state.busy, isNull);

        bloc.add(const SignInEvent.usernameChosen('ana'));
        await bloc.stream.firstWhere((state) => state.outcome != null);
        expect(bloc.state.recoveryKey, sampleRecoveryKey);
        expect(bloc.state.suggestedUsername, isNull);
        unawaited(steps.close());
        await bloc.close();
      },
    );

    test(
      'a username that breaks the rules is told before it is sent',
      () async {
        final bloc = await started();
        bloc.add(const SignInEvent.usernameChosen('no way'));
        await pumpEventQueue();
        expect(bloc.state.problem, FormProblem.usernameInvalid);
        verifyNever(() => account.finishProviderSignUp(any()));
        await bloc.close();
      },
    );

    test('a flow that failed is the library’s words', () async {
      final bloc = await started();
      bloc.add(
        const SignInEvent.providerPressed(
          provider: ProviderKind.github,
          deviceName: _device,
        ),
      );
      steps.add(
        const ProviderEvent.failed(
          message: 'The sign-in was cancelled at the provider.',
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(
        bloc.state.error,
        const ApplicationError.invalid(
          message: 'The sign-in was cancelled at the provider.',
        ),
      );
      expect(bloc.state.busy, isNull);
      unawaited(steps.close());
      await bloc.close();
    });

    test('Cancel stops the flow and closes the page', () async {
      final bloc = await started();
      bloc.add(
        const SignInEvent.providerPressed(
          provider: ProviderKind.github,
          deviceName: _device,
        ),
      );
      steps.add(
        const ProviderEvent.showCode(
          userCode: 'ABCD-1234',
          verificationUri: 'https://github.example.invalid/login/device',
        ),
      );
      await bloc.stream.firstWhere((state) => state.code != null);
      bloc.add(const SignInEvent.providerCancelled());
      await bloc.stream.firstWhere((state) => state.code == null);
      verify(account.cancelProvider).called(1);
      verify(session.close).called(greaterThanOrEqualTo(1));
      expect(bloc.state.busy, isNull);
      unawaited(steps.close());
      await bloc.close();
    });
  });
}
