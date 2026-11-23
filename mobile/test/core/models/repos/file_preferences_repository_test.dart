// Kanso — file_preferences_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/library_folder.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/models/repos/impl/file_preferences_repository.dart';
import 'package:path/path.dart' as p;

void main() {
  late Directory directory;
  late LibraryFolder folder;

  setUp(() async {
    directory = await Directory.systemTemp.createTemp('kanso_prefs_');
    folder = LibraryFolder(p.join(directory.path, 'kanso'));
  });

  tearDown(() => directory.delete(recursive: true));

  File file() => File(folder.preferencesPath);

  Preferences loaded(Either<Object, Preferences> result) =>
      result.getOrElse(() => throw StateError('failed'));

  group('FilePreferencesRepository', () {
    test('loads the defaults when nothing was saved', () async {
      final repository = FilePreferencesRepository(folder);
      expect(loaded(await repository.load()), const Preferences());
      expect(file().existsSync(), isFalse);
    });

    test('saves in preferences.json beside the library, as JSON', () async {
      final repository = FilePreferencesRepository(folder);
      await repository.update(
        (current) => current.copyWith(textSize: 1.15),
      );
      expect(folder.preferencesPath, endsWith('kanso/preferences.json'));
      final json = jsonDecode(file().readAsStringSync());
      expect(json, containsPair('textSize', 1.15));
      expect(json, containsPair('version', preferencesFileVersion));
    });

    test('what one repository saved the next one loads', () async {
      await FilePreferencesRepository(folder).update(
        (current) => current.copyWith(
          sort: SortKey.title,
          descending: false,
          backupNoticeSeen: true,
          themeSelection: const ThemeSelection(light: 'github-light'),
        ),
      );
      expect(
        loaded(await FilePreferencesRepository(folder).load()),
        const Preferences(
          sort: SortKey.title,
          descending: false,
          backupNoticeSeen: true,
          themeSelection: ThemeSelection(light: 'github-light'),
        ),
      );
    });

    test('every order of the notebooks is kept for the next run', () async {
      for (final sort in BookSort.values) {
        await FilePreferencesRepository(folder).update(
          (current) => current.copyWith(bookSort: sort),
        );
        final next = loaded(await FilePreferencesRepository(folder).load());
        expect(next.bookSort, sort);
      }
    });

    test(
      'the notebooks are in the default order until one is chosen',
      () async {
        final repository = FilePreferencesRepository(folder);
        expect(loaded(await repository.load()).bookSort, BookSort.manual);
        await repository.update((c) => c.copyWith(descending: false));
        expect(loaded(await repository.load()).bookSort, BookSort.manual);
      },
    );

    test('an order this version does not know is the default order', () async {
      Directory(folder.path).createSync(recursive: true);
      file().writeAsStringSync(
        jsonEncode(<String, Object>{
          'version': preferencesFileVersion,
          'bookSort': 'byColour',
        }),
      );
      final saved = loaded(await FilePreferencesRepository(folder).load());
      expect(saved.bookSort, BookSort.manual);
    });

    test('leaves no temporary file behind', () async {
      final repository = FilePreferencesRepository(folder);
      await repository.update((current) => current);
      expect(
        Directory(folder.path).listSync().map((e) => p.basename(e.path)),
        <String>['preferences.json'],
      );
    });

    test('changes made together are all kept, in order', () async {
      final repository = FilePreferencesRepository(folder);
      final results = await Future.wait(<Future<Object>>[
        repository.update((c) => c.copyWith(sort: SortKey.created)),
        repository.update((c) => c.copyWith(descending: false)),
        repository.update((c) => c.copyWith(textSize: 1.3)),
      ]);
      expect(results, hasLength(3));
      final saved = Preferences.fromJson(
        jsonDecode(file().readAsStringSync()) as Map<String, dynamic>,
      );
      expect(
        saved,
        const Preferences(
          sort: SortKey.created,
          descending: false,
          textSize: 1.3,
        ),
      );
    });

    test(
      'a file that is not JSON loads as the defaults and is replaced',
      () async {
        Directory(folder.path).createSync(recursive: true);
        file().writeAsStringSync('{ not json');
        final repository = FilePreferencesRepository(folder);
        expect(loaded(await repository.load()), const Preferences());
        await repository.update((c) => c.copyWith(descending: false));
        final saved = Preferences.fromJson(
          jsonDecode(file().readAsStringSync()) as Map<String, dynamic>,
        );
        expect(saved.descending, isFalse);
      },
    );

    test(
      'a file that is JSON but not an object loads as the defaults',
      () async {
        Directory(folder.path).createSync(recursive: true);
        file().writeAsStringSync('[1, 2]');
        expect(
          loaded(await FilePreferencesRepository(folder).load()),
          const Preferences(),
        );
      },
    );

    test('a folder that cannot be written is a storage error', () async {
      // The folder's place is taken by a file.
      File(folder.path).writeAsStringSync('in the way');
      final result = await FilePreferencesRepository(
        folder,
      ).update((c) => c.copyWith(descending: false));
      expect(result.isLeft(), isTrue);
    });
  });
}
