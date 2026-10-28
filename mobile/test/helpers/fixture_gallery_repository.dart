// Kanso — fixture_gallery_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/repos/abs/gallery_repository.dart';
import 'package:kanso/core/utils/theme_changes.dart';

import 'gallery_fixtures.dart';
import 'theme_tokens.dart';

/// A gallery for widget tests that behaves like the real one: it lists the
/// [items] it is given, installing one makes it installed (and its colour
/// theme readable as an installed theme), removing one undoes that, and a
/// test can make any call fail or hold one back.
class FixtureGalleryRepository implements GalleryRepository {
  FixtureGalleryRepository({List<GalleryItem>? items})
    : items = items ?? sampleGallery();

  /// The gallery's items now.
  List<GalleryItem> items;

  /// What each call to [index] was asked: with `refresh` or without.
  final List<bool> indexCalls = <bool>[];

  /// The ids installed, in order, and the ids removed.
  final List<String> installs = <String>[];
  final List<String> removals = <String>[];

  /// When set, the call without `refresh` fails with it.
  ApplicationError? cachedFailure;

  /// When set, the call with `refresh` fails with it.
  ApplicationError? refreshFailure;

  /// When set, installing fails with it.
  ApplicationError? installFailure;

  /// When set, [index] waits for it before it answers.
  Completer<void>? gate;

  /// When set, installing waits for it before it answers.
  Completer<void>? installGate;

  /// The palette an installed colour theme has, by id; Kanso Light renamed
  /// for the others.
  final Map<String, KansoPalette> palettes = <String, KansoPalette>{};

  final ThemeChanges _changes = ThemeChanges();

  @override
  Stream<String> themeChanges() => _changes.changes;

  KansoPalette _paletteOf(GalleryItem item) =>
      palettes[item.id] ??
      fakePalette(item.id, dark: item.dark).copyWith(name: item.name);

  /// The installed colour themes as rows in the themes folder.
  List<InstalledTheme> installedColorThemes() => <InstalledTheme>[
    for (final item in items)
      if (item.installed &&
          !item.disabled &&
          item.kind == GalleryKind.colorTheme)
        InstalledTheme(
          id: item.id,
          name: item.name,
          author: item.author,
          license: item.license,
          dark: item.dark,
          origin: InstalledThemeOrigin.gallery,
          tokens: tokensFor(_paletteOf(item)),
        ),
  ];

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> index({
    bool refresh = false,
  }) async {
    indexCalls.add(refresh);
    await gate?.future;
    final failure = refresh ? refreshFailure : cachedFailure;
    if (failure != null) {
      return Left<ApplicationError, List<GalleryItem>>(failure);
    }
    _changes.seen(items);
    return Right<ApplicationError, List<GalleryItem>>(
      List<GalleryItem>.of(items),
    );
  }

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> install(
    String id,
  ) async {
    await installGate?.future;
    final failure = installFailure;
    if (failure != null) {
      return Left<ApplicationError, List<GalleryItem>>(failure);
    }
    installs.add(id);
    items = <GalleryItem>[
      for (final item in items)
        if (item.id == id)
          galleryItem(
            item.id,
            name: item.name,
            kind: item.kind,
            author: item.author,
            official: item.official,
            version: item.updateAvailable ? '${item.version}+' : item.version,
            updatedMs: item.updatedMs,
            source: item.source,
            dark: item.dark,
            headline: item.headline,
            installed: true,
          )
        else
          item,
    ];
    _changes.seen(items, touched: <String>{id});
    return Right<ApplicationError, List<GalleryItem>>(items);
  }

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> uninstall(
    String id,
  ) async {
    removals.add(id);
    items = <GalleryItem>[
      for (final item in items)
        if (item.id == id)
          galleryItem(
            item.id,
            name: item.name,
            kind: item.kind,
            author: item.author,
            official: item.official,
            version: item.version,
            updatedMs: item.updatedMs,
            source: item.source,
            dark: item.dark,
            headline: item.headline,
          )
        else
          item,
    ];
    _changes.seen(items, touched: <String>{id});
    return Right<ApplicationError, List<GalleryItem>>(items);
  }

  @override
  Future<Either<ApplicationError, List<BuiltinTheme>>> installedThemes() async {
    return Right<ApplicationError, List<BuiltinTheme>>(<BuiltinTheme>[
      for (final item in items)
        if (item.installed &&
            !item.disabled &&
            item.kind == GalleryKind.colorTheme)
          BuiltinTheme(
            id: item.id,
            name: item.name,
            author: item.author,
            license: item.license,
            dark: item.dark,
            tokens: tokensFor(_paletteOf(item)),
          ),
    ]);
  }

  @override
  Future<Either<ApplicationError, ThemeTokens>> installedThemeTokens(
    String id,
  ) async {
    final found = items.where(
      (item) =>
          item.id == id &&
          item.installed &&
          item.kind == GalleryKind.colorTheme,
    );
    if (found.isEmpty) {
      return const Left<ApplicationError, ThemeTokens>(
        ApplicationError.notFound(message: 'Not installed.'),
      );
    }
    return Right<ApplicationError, ThemeTokens>(
      tokensFor(_paletteOf(found.first)),
    );
  }
}
