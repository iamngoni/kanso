// Kanso — note_html_exporter_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/note/state/export/note_html_exporter.dart';

import '../../../helpers/theme_tokens.dart';

/// Answers the two calls an export makes, and records what it was asked.
class _FakeRenderer extends Fake implements RendererRepository {
  _FakeRenderer({this.tokens, this.page});

  final Either<ApplicationError, ThemeTokens>? tokens;
  final Either<ApplicationError, ExportedPage>? page;

  final List<({String id, PaletteDto palette, PreviewOptions options})> drawn =
      <({String id, PaletteDto palette, PreviewOptions options})>[];

  @override
  Future<Either<ApplicationError, ThemeTokens>> builtinThemeTokens(
    String id,
  ) async => tokens!;

  @override
  Future<Either<ApplicationError, ExportedPage>> pageHtml(
    String id,
    PaletteDto palette,
    PreviewOptions options,
  ) async {
    drawn.add((id: id, palette: palette, options: options));
    return page!;
  }
}

void main() {
  final tokens = tokensFor(KansoPalette.kansoLight);
  const page = ExportedPage(html: '<p>Hello</p>', fileName: 'Hello.html');

  late Directory folder;

  setUp(() async {
    folder = await Directory.systemTemp.createTemp('kanso-html-export');
  });

  tearDown(() async {
    if (folder.existsSync()) {
      await folder.delete(recursive: true);
    }
  });

  test('the note is written in its theme, with its page name', () async {
    final renderer = _FakeRenderer(
      tokens: Right<ApplicationError, ThemeTokens>(tokens),
      page: const Right<ApplicationError, ExportedPage>(page),
    );
    final exporter = NoteHtmlExporter(renderer, folder: () async => folder);

    final written = await exporter.export(noteId: 'note:1', themeId: 'nord');

    final path = written.fold((_) => fail('a path'), (path) => path);
    expect(path, '${folder.path}/Hello.html');
    expect(File(path).readAsStringSync(), '<p>Hello</p>');
    expect(renderer.drawn.single.id, 'note:1');
    expect(renderer.drawn.single.palette, tokens.palette);
    expect(renderer.drawn.single.options.code, tokens.syntax);
  });

  test('a locked note is the error it came as', () async {
    const error = ApplicationError.locked(message: 'sealed');
    final exporter = NoteHtmlExporter(
      _FakeRenderer(
        tokens: Right<ApplicationError, ThemeTokens>(tokens),
        page: const Left<ApplicationError, ExportedPage>(error),
      ),
      folder: () async => folder,
    );

    expect(
      await exporter.export(noteId: 'note:1', themeId: 'nord'),
      const Left<ApplicationError, String>(error),
    );
  });

  test('a folder that cannot be written is a storage error', () async {
    final exporter = NoteHtmlExporter(
      _FakeRenderer(
        tokens: Right<ApplicationError, ThemeTokens>(tokens),
        page: const Right<ApplicationError, ExportedPage>(page),
      ),
      folder: () async => Directory('${folder.path}/gone'),
    );

    final written = await exporter.export(noteId: 'note:1', themeId: 'nord');

    expect(written.isLeft(), isTrue);
    expect(
      written.fold((error) => error, (_) => fail('a path')),
      isA<StorageError>(),
    );
  });
}
