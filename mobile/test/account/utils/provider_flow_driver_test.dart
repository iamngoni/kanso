// Kanso — provider_flow_driver_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/utils/provider_flow_driver.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: 'https://s',
);

void main() {
  late MockAuthSessionRepository session;
  late List<String?> ended;
  late ProviderFlowDriver driver;
  late Completer<Either<ApplicationError, String?>> open;

  setUp(() {
    session = MockAuthSessionRepository();
    ended = <String?>[];
    open = Completer<Either<ApplicationError, String?>>();
    when(
      () => session.open(any(), callbackScheme: any(named: 'callbackScheme')),
    ).thenAnswer((_) => open.future);
    when(session.close).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    driver = ProviderFlowDriver(session, (callback) async {
      ended.add(callback);
      return const Right<ApplicationError, Unit>(unit);
    });
  });

  group('ProviderFlowDriver', () {
    test('opens the page the flow asks for, with its callback scheme', () {
      driver.handle(
        const ProviderEvent.openPage(
          url: 'https://example.invalid/auth',
          callbackScheme: 'kanso',
        ),
      );
      verify(
        () => session.open(
          'https://example.invalid/auth',
          callbackScheme: 'kanso',
        ),
      ).called(1);
    });

    test('a callback address goes back to the library', () async {
      driver.handle(const ProviderEvent.openPage(url: 'https://x.invalid/a'));
      open.complete(
        const Right<ApplicationError, String?>('kanso://auth/apple?code=1'),
      );
      await pumpEventQueue();
      expect(ended, <String?>['kanso://auth/apple?code=1']);
    });

    test('a page the person dismissed goes back as null', () async {
      driver.handle(const ProviderEvent.openPage(url: 'https://x.invalid/a'));
      open.complete(const Right<ApplicationError, String?>(null));
      await pumpEventQueue();
      expect(ended, <String?>[null]);
    });

    test(
      'a page that could not be opened ends the flow like a dismissed one',
      () async {
        driver.handle(const ProviderEvent.openPage(url: 'https://x.invalid/a'));
        open.complete(
          const Left<ApplicationError, String?>(
            ApplicationError.unexpected(message: 'no activity'),
          ),
        );
        await pumpEventQueue();
        expect(ended, <String?>[null]);
      },
    );

    test('a page the app closed is not told to the library', () async {
      driver
        ..handle(const ProviderEvent.openPage(url: 'https://x.invalid/a'))
        ..handle(const ProviderEvent.closePage());
      verify(session.close).called(1);
      // The session’s open call completes with null because the app closed
      // it, whenever it gets to.
      open.complete(const Right<ApplicationError, String?>(null));
      await pumpEventQueue();
      expect(ended, isEmpty);
    });

    test('every last step of the flow closes the page', () {
      for (final last in <ProviderEvent>[
        const ProviderEvent.needsUsername(suggested: 'ana'),
        const ProviderEvent.finished(_signedIn),
        const ProviderEvent.failed(message: 'No.'),
        const ProviderEvent.cancelled(),
      ]) {
        clearInteractions(session);
        driver.handle(last);
        verify(session.close).called(1);
      }
    });

    test('a code to show opens and closes nothing', () {
      driver.handle(
        const ProviderEvent.showCode(
          userCode: 'ABCD-1234',
          verificationUri: 'https://x.invalid/device',
        ),
      );
      verifyNever(
        () => session.open(any(), callbackScheme: any(named: 'callbackScheme')),
      );
      verifyNever(session.close);
    });

    test('a page opened again after one was closed is told again', () async {
      driver.handle(const ProviderEvent.closePage());
      final second = Completer<Either<ApplicationError, String?>>();
      when(
        () => session.open(
          'https://x.invalid/b',
          callbackScheme: any(named: 'callbackScheme'),
        ),
      ).thenAnswer((_) => second.future);
      driver.handle(const ProviderEvent.openPage(url: 'https://x.invalid/b'));
      second.complete(const Right<ApplicationError, String?>(null));
      await pumpEventQueue();
      expect(ended, <String?>[null]);
    });
  });
}
