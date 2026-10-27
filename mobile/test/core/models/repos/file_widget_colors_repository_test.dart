// Kanso — file_widget_colors_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/library_folder.dart';
import 'package:kanso/core/models/repos/impl/file_widget_colors_repository.dart';
import 'package:kanso/core/services/widget_reloader.dart';
import 'package:mocktail/mocktail.dart';
import 'package:path/path.dart' as p;

class _MockReloader extends Mock implements WidgetReloader {}

void main() {
  late Directory directory;
  late LibraryFolder folder;
  late _MockReloader reloader;
  late FileWidgetColorsRepository repository;

  setUp(() async {
    directory = await Directory.systemTemp.createTemp('kanso_widgets_');
    folder = LibraryFolder(p.join(directory.path, 'kanso'));
    Directory(folder.path).createSync(recursive: true);
    reloader = _MockReloader();
    when(() => reloader.reload()).thenAnswer((_) async {});
    repository = FileWidgetColorsRepository(folder, reloader);
  });

  tearDown(() => directory.delete(recursive: true));

  Future<bool> publish([KansoColors? light]) async {
    final result = await repository.publish(
      light: light ?? KansoColors.light,
      dark: KansoColors.dark,
    );
    return result.getOrElse(() => throw StateError('failed: $result'));
  }

  group('FileWidgetColorsRepository', () {
    test(
      'writes widget.json beside the library and redraws the widgets',
      () async {
        expect(await publish(), isTrue);
        expect(folder.widgetColorsPath, endsWith('kanso/widget.json'));
        final json =
            jsonDecode(File(folder.widgetColorsPath).readAsStringSync())
                as Map<String, dynamic>;
        expect(json['version'], 1);
        expect(
          (json['light'] as Map<String, dynamic>)['background'],
          '#F4F1EA',
        );
        expect((json['dark'] as Map<String, dynamic>)['text'], '#F1EFE7');
        verify(() => reloader.reload()).called(1);
        // No temporary file is left.
        expect(File('${folder.widgetColorsPath}.tmp').existsSync(), isFalse);
      },
    );

    test(
      'colours already in the file change nothing and redraw nothing',
      () async {
        await publish();
        expect(await publish(), isFalse);
        verify(() => reloader.reload()).called(1);
      },
    );

    test('a changed colour is written and redraws again', () async {
      await publish();
      final changed = KansoColors.light.copyWith(
        accent: const Color(0xFF123456),
      );
      expect(await publish(changed), isTrue);
      expect(
        File(folder.widgetColorsPath).readAsStringSync(),
        contains('#123456'),
      );
      verify(() => reloader.reload()).called(2);
    });

    test('a folder that cannot be written is a storage error', () async {
      Directory(folder.path).deleteSync(recursive: true);
      final result = await repository.publish(
        light: KansoColors.light,
        dark: KansoColors.dark,
      );
      expect(
        result,
        isA<Left<ApplicationError, bool>>().having(
          (left) => left.value,
          'error',
          isA<StorageError>(),
        ),
      );
      verifyNever(() => reloader.reload());
    });

    test('a redraw that fails is an error, with the file written', () async {
      when(() => reloader.reload()).thenThrow(StateError('no widgets'));
      final result = await repository.publish(
        light: KansoColors.light,
        dark: KansoColors.dark,
      );
      expect(result.isLeft(), isTrue);
      expect(File(folder.widgetColorsPath).existsSync(), isTrue);
    });
  });
}
