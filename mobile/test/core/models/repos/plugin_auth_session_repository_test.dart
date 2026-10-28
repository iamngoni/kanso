// Kanso — plugin_auth_session_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: only_throw_errors

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/plugin_auth_session_repository.dart';
import 'package:kanso_auth_session/kanso_auth_session.dart';
import 'package:kanso_auth_session/src/messages.g.dart' show AuthSessionHostApi;

/// The platform’s session, recording what it is asked.
class _Session extends AuthSessionHostApi {
  String? answer;
  Object? failure;
  final List<String> opened = <String>[];
  final List<String?> schemes = <String?>[];
  int closes = 0;

  @override
  Future<String?> open(String url, String? callbackScheme) async {
    opened.add(url);
    schemes.add(callbackScheme);
    final broken = failure;
    if (broken != null) {
      throw broken;
    }
    return answer;
  }

  @override
  Future<void> close() async => closes++;
}

void main() {
  late _Session session;
  late PluginAuthSessionRepository repository;

  setUp(() {
    session = _Session();
    KansoAuthSession.debugApi = session;
    repository = PluginAuthSessionRepository();
  });

  tearDown(() => KansoAuthSession.debugApi = AuthSessionHostApi());

  group('PluginAuthSessionRepository', () {
    test(
      'opens the page, and gives the address the session ended at',
      () async {
        session.answer = 'kanso://auth/apple?code=abc';
        expect(
          await repository.open(
            'https://appleid.example.invalid/auth',
            callbackScheme: 'kanso',
          ),
          const Right<ApplicationError, String?>('kanso://auth/apple?code=abc'),
        );
        expect(session.opened, <String>[
          'https://appleid.example.invalid/auth',
        ]);
        expect(session.schemes, <String?>['kanso']);
      },
    );

    test('a session that was dismissed gives null', () async {
      expect(
        await repository.open('https://example.invalid/start'),
        const Right<ApplicationError, String?>(null),
      );
      expect(session.schemes, <String?>[null]);
    });

    test('a page that is not a web page is refused, not opened', () async {
      final result = await repository.open('file:///etc/passwd');
      expect(result.isLeft(), isTrue);
      expect(session.opened, isEmpty);
    });

    test('a platform failure is an error', () async {
      session.failure = PlatformException(code: 'no_activity');
      final result = await repository.open('https://example.invalid/start');
      expect(result.swap().toOption().toNullable(), isA<UnexpectedError>());
    });

    test('closing asks the platform to end the session', () async {
      expect(
        await repository.close(),
        const Right<ApplicationError, Unit>(unit),
      );
      expect(session.closes, 1);
    });
  });
}
