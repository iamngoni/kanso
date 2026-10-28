// Kanso — fixture_renderer_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart' show Fake;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';

import 'theme_tokens.dart';

/// A simple square icon, for a fixture icon theme.
const String fixtureIconSvg =
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">\n'
    '<rect x="4" y="4" width="16" height="16" rx="3" fill="none" '
    'stroke="black" stroke-width="2"/>\n'
    '</svg>';

/// The colours of the runs [FixtureRendererRepository] makes.
abstract final class FixtureRunColors {
  static const int heading = 0x8A4B14;
  static const int code = 0x2F7469;
  static const int quote = 0x6B6258;
  static const int marker = 0x9A8F80;
  static const int link = 0x2F5F9E;
}

/// A renderer that styles Markdown the plain way, for tests that must not
/// depend on the Rust core: headings, fenced code, quotes, list markers,
/// bold, inline code and note links. Every other call is an error.
class FixtureRendererRepository extends Fake implements RendererRepository {
  /// The themes it was asked for, in order.
  final List<String> themes = <String>[];

  /// What it answers instead of runs, to test a text that cannot be styled.
  ApplicationError? failure;

  /// What the icon themes call answers instead of the fixture's themes, and
  /// what holds it back until it completes.
  Either<ApplicationError, List<IconThemeRow>>? iconThemesResult;
  Completer<void>? iconThemesGate;

  @override
  Future<Either<ApplicationError, List<IconThemeRow>>> iconThemes() async {
    await iconThemesGate?.future;
    return iconThemesResult ??
        const Right<ApplicationError, List<IconThemeRow>>(
          <IconThemeRow>[
            IconThemeRow(
              id: 'lucide',
              name: 'Lucide',
              author: 'Lucide Contributors',
              license: 'ISC',
              iconCount: 1,
              preview: <ThemedIcon>[ThemedIcon(name: 'notebook')],
            ),
            IconThemeRow(
              id: 'outline',
              name: 'Outline',
              author: 'Someone',
              license: 'MIT',
              iconCount: 1,
              preview: <ThemedIcon>[
                ThemedIcon(name: 'notebook', svg: fixtureIconSvg),
              ],
            ),
          ],
        );
  }

  @override
  Future<Either<ApplicationError, List<ThemedIcon>>> iconThemeIcons(
    String id,
  ) async => Right<ApplicationError, List<ThemedIcon>>(
    id == 'outline'
        ? const <ThemedIcon>[ThemedIcon(name: 'notebook', svg: fixtureIconSvg)]
        : const <ThemedIcon>[ThemedIcon(name: 'notebook')],
  );

  @override
  Future<Either<ApplicationError, List<BuiltinTheme>>> builtinThemes() async =>
      Right<ApplicationError, List<BuiltinTheme>>(<BuiltinTheme>[
        for (final palette in <KansoPalette>[
          KansoPalette.kansoLight,
          KansoPalette.kansoDark,
        ])
          BuiltinTheme(
            id: palette.id,
            name: palette.name,
            author: 'Kanso',
            license: 'MIT',
            dark: palette.dark,
            tokens: tokensFor(palette),
          ),
      ]);

  @override
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String themeId,
  }) async {
    themes.add(themeId);
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, List<StyledRun>>(error);
    }
    return Right<ApplicationError, List<StyledRun>>(
      fixtureRuns(text.substring(start, end), offset: start),
    );
  }
}

/// The runs a plain pass over [text] makes, starting at [offset].
List<StyledRun> fixtureRuns(String text, {int offset = 0}) {
  final runs = <StyledRun>[];
  var at = 0;
  var fenced = false;
  for (final line in text.split('\n')) {
    final start = offset + at;
    final end = start + line.length;
    if (line.startsWith('```')) {
      fenced = !fenced;
      runs.add(_run(start, end, FixtureRunColors.marker));
    } else if (fenced) {
      runs.add(_run(start, end, FixtureRunColors.code));
    } else if (line.startsWith('#')) {
      runs.add(_run(start, end, FixtureRunColors.heading, weight: 700));
    } else if (line.startsWith('>')) {
      runs.add(_run(start, end, FixtureRunColors.quote, italic: true));
    } else {
      for (final match in RegExp(r'\*\*[^*]+\*\*').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, null, weight: 700),
        );
      }
      for (final match in RegExp('`[^`]+`').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, FixtureRunColors.code),
        );
      }
      for (final match in RegExp(r'\[\[[^\]]+\]\]').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, FixtureRunColors.link),
        );
      }
    }
    at += line.length + 1;
  }
  return runs;
}

StyledRun _run(int start, int end, int? color, {bool? italic, int? weight}) {
  return StyledRun(
    range: Range16(start: start, end: end),
    color: color,
    italic: italic,
    weight: weight,
  );
}
