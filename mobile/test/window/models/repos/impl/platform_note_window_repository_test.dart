// Kanso — platform_note_window_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/library_folder.dart';
import 'package:kanso/window/configs/window_configs.dart';
import 'package:kanso/window/models/data/note_window_request.dart';
import 'package:kanso/window/models/repos/impl/platform_note_window_repository.dart';

const MethodChannel _channel = MethodChannel(WindowConfigs.channel);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final calls = <MethodCall>[];
  Object? Function(MethodCall call)? answer;

  setUp(() {
    calls.clear();
    answer = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_channel, (call) async {
          calls.add(call);
          final custom = answer;
          if (custom != null) {
            return custom(call);
          }
          return call.method == 'supported' ? true : null;
        });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_channel, null);
  });

  Future<PlatformNoteWindowRepository> make({String env = 'production'}) {
    return PlatformNoteWindowRepository.create(
      const LibraryFolder('/support/kanso'),
      environment: env,
    );
  }

  group('PlatformNoteWindowRepository', () {
    test(
      'asks the device once whether it shows more than one window',
      () async {
        final windows = await make();
        expect(windows.supported, isTrue);
        expect(calls.map((call) => call.method), <String>['supported']);
      },
    );

    test('a device that does not answer shows one window', () async {
      answer = (call) => throw PlatformException(code: 'nope');
      final windows = await make();
      expect(windows.supported, isFalse);
      final opened = await windows.open('n1');
      expect(opened.isLeft(), isTrue);
      // It was never sent: a device with one window opens none.
      expect(calls.where((call) => call.method == 'open'), isEmpty);
    });

    test('a missing native side is one window as well', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(_channel, null);
      final windows = await make();
      expect(windows.supported, isFalse);
    });

    test(
      'open names the note, the environment and the library folder',
      () async {
        final windows = await make(env: 'integration');
        final result = await windows.open('note-1');
        expect(result.isRight(), isTrue);
        final open = calls.singleWhere((call) => call.method == 'open');
        expect(open.arguments, <String, Object?>{
          'noteId': 'note-1',
          'environment': 'integration',
          'libraryFolder': '/support/kanso',
        });
      },
    );

    test('a failure of the native side is an error, not a throw', () async {
      final windows = await make();
      answer = (call) {
        if (call.method == 'open') {
          throw PlatformException(code: 'activation', message: 'refused');
        }
        return null;
      };
      final result = await windows.open('n1');
      expect(
        result.fold((error) => error, (_) => null),
        const ApplicationError.unexpected(message: 'refused'),
      );
    });

    test('closes this window, and names it', () async {
      final windows = await make();
      expect((await windows.closeThisWindow()).isRight(), isTrue);
      expect((await windows.setTitle('Plan')).isRight(), isTrue);
      expect(calls.map((call) => call.method), <String>[
        'supported',
        'closeThisWindow',
        'setTitle',
      ]);
      expect(calls.last.arguments, <String, Object?>{'title': 'Plan'});
    });

    test('hands every kind of request to the main window', () async {
      final windows = await make();
      const requests = <NoteWindowRequest>[
        NoteWindowRequest.newNote(),
        NoteWindowRequest.newNoteTitled('Weekly review'),
        NoteWindowRequest.search(),
        NoteWindowRequest.palette(),
        NoteWindowRequest.openNote('n9'),
      ];
      for (final request in requests) {
        expect((await windows.hand(request)).isRight(), isTrue);
      }
      expect(
        calls.where((call) => call.method == 'hand').map((c) => c.arguments),
        <Map<String, Object?>>[
          <String, Object?>{'kind': 'newNote'},
          <String, Object?>{'kind': 'newNoteTitled', 'title': 'Weekly review'},
          <String, Object?>{'kind': 'search'},
          <String, Object?>{'kind': 'palette'},
          <String, Object?>{'kind': 'openNote', 'noteId': 'n9'},
        ],
      );
    });

    test(
      'requests handed over by a note window arrive as the same requests',
      () async {
        final windows = await make();
        final heard = <NoteWindowRequest>[];
        final subscription = windows.requests().listen(heard.add);
        addTearDown(subscription.cancel);
        const requests = <NoteWindowRequest>[
          NoteWindowRequest.newNote(),
          NoteWindowRequest.newNoteTitled('Weekly review'),
          NoteWindowRequest.search(),
          NoteWindowRequest.palette(),
          NoteWindowRequest.openNote('n9'),
        ];
        for (final request in requests) {
          await TestDefaultBinaryMessengerBinding
              .instance
              .defaultBinaryMessenger
              .handlePlatformMessage(
                WindowConfigs.channel,
                const StandardMethodCodec().encodeMethodCall(
                  MethodCall(
                    'request',
                    PlatformNoteWindowRepository.encodeRequest(request),
                  ),
                ),
                (_) {},
              );
        }
        await Future<void>.delayed(Duration.zero);
        expect(heard, requests);
      },
    );

    test('a request it does not know is ignored', () async {
      expect(PlatformNoteWindowRepository.decodeRequest(null), isNull);
      expect(PlatformNoteWindowRepository.decodeRequest('x'), isNull);
      expect(
        PlatformNoteWindowRepository.decodeRequest(<String, Object?>{
          'kind': 'teleport',
        }),
        isNull,
      );
      expect(
        PlatformNoteWindowRepository.decodeRequest(<String, Object?>{
          'kind': 'openNote',
        }),
        isNull,
      );
    });
  });
}
