// Kanso — note_pdf_exporter_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/note/state/export/note_pdf_exporter.dart';
import 'package:kanso_print/kanso_print.dart';

import '../../../helpers/theme_tokens.dart';

/// Answers the two calls an export makes, and records what it was asked.
class _FakeRenderer extends Fake implements RendererRepository {
  _FakeRenderer({this.tokens, this.page});

  final Either<ApplicationError, ThemeTokens>? tokens;
  final Either<ApplicationError, ExportedPage>? page;

  final List<({String id, PaletteDto palette, PreviewOptions options})> drawed =
      <({String id, PaletteDto palette, PreviewOptions options})>[];

  @override
  Future<Either<ApplicationError, ThemeTokens>> builtinThemeTokens(
    String id,
  ) async => tokens!;

  @override
  Future<Either<ApplicationError, ExportedPage>> pagePdf(
    String id,
    PaletteDto palette,
    PreviewOptions options,
  ) async {
    drawed.add((id: id, palette: palette, options: options));
    return page!;
  }
}

void main() {
  final tokens = tokensFor(KansoPalette.kansoLight);
  const page = ExportedPage(html: '<p>Hello</p>', fileName: 'Hello.pdf');

  test('a locale name gives its country, or none', () {
    expect(countryOf('en_GB'), 'GB');
    expect(countryOf('en-GB'), 'GB');
    expect(countryOf('pt_BR'), 'BR');
    expect(countryOf('en'), isNull);
    expect(countryOf('C'), isNull);
  });

  test('the paper follows the country, A4 elsewhere', () {
    expect(
      NotePdfExporter(
        _FakeRenderer(),
        locale: 'en_US',
      ).paper,
      KansoPaper.letter,
    );
    expect(
      NotePdfExporter(_FakeRenderer(), locale: 'en_GB').paper,
      KansoPaper.a4,
    );
    expect(NotePdfExporter(_FakeRenderer(), locale: 'en').paper, KansoPaper.a4);
  });

  test('the note is written in its theme, with its page name', () async {
    final renderer = _FakeRenderer(
      tokens: Right<ApplicationError, ThemeTokens>(tokens),
      page: const Right<ApplicationError, ExportedPage>(page),
    );
    final printed = <({String html, String path, KansoPaper paper})>[];
    final exporter = NotePdfExporter(
      renderer,
      folder: () async => Directory('/tmp/kanso-export-test'),
      print: ({required html, required path, required paper}) async {
        printed.add((html: html, path: path, paper: paper));
        return 2;
      },
      locale: 'en_US',
    );

    final written = await exporter.export(noteId: 'note:1', themeId: 'nord');

    expect(
      written,
      const Right<ApplicationError, String>('/tmp/kanso-export-test/Hello.pdf'),
    );
    expect(printed.single.html, '<p>Hello</p>');
    expect(printed.single.paper, KansoPaper.letter);
    expect(renderer.drawed.single.id, 'note:1');
    expect(renderer.drawed.single.palette, tokens.palette);
    expect(renderer.drawed.single.options.code, tokens.syntax);
  });

  test('a locked note is the error it came as', () async {
    const error = ApplicationError.locked(message: 'sealed');
    final exporter = NotePdfExporter(
      _FakeRenderer(
        tokens: Right<ApplicationError, ThemeTokens>(tokens),
        page: const Left<ApplicationError, ExportedPage>(error),
      ),
      folder: () async => Directory('/tmp/kanso-export-test'),
      print: ({required html, required path, required paper}) async => 1,
      locale: 'en_GB',
    );

    expect(
      await exporter.export(noteId: 'note:1', themeId: 'nord'),
      const Left<ApplicationError, String>(error),
    );
  });

  test(
    'a writer that fails is a storage error, and nothing to share',
    () async {
      final exporter = NotePdfExporter(
        _FakeRenderer(
          tokens: Right<ApplicationError, ThemeTokens>(tokens),
          page: const Right<ApplicationError, ExportedPage>(page),
        ),
        folder: () async => Directory('/tmp/kanso-export-test'),
        print: ({required html, required path, required paper}) async =>
            throw const FileSystemException('no room'),
        locale: 'en_GB',
      );

      final written = await exporter.export(noteId: 'note:1', themeId: 'nord');

      expect(written.isLeft(), isTrue);
      expect(
        written.fold((error) => error, (_) => fail('a path')),
        isA<StorageError>(),
      );
    },
  );
}
