// Kanso — note_body.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/cupertino.dart' show CupertinoTheme, CupertinoThemeData;
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/list_labels.dart';
import '../../../core/utils/status_label.dart';
import '../../../injection.dart';
import '../../../people/models/repos/abs/people_repository.dart';
import '../../../people/views/widgets/page_panel.dart';
import '../../models/data/source_line.dart';
import 'note_metadata_block.dart';
import 'note_preview.dart';
import 'preview/preview_style.dart';

/// A note that is open: its title, a compact block of its facts, and its
/// text, which scrolls with them. All of it can be selected: the selection
/// area is round the scroll, so it spans the text's blocks, which are built
/// only near the screen. What is selected, and the handles, are in the
/// preview theme's accent.
class NoteBody extends StatelessWidget {
  const NoteBody({
    required this.note,
    required this.lines,
    required this.index,
    required this.gutter,
    this.onEditAt,
    super.key,
  });

  final NoteView note;
  final List<SourceLine> lines;

  /// The notebooks and tags the facts are looked up in.
  final LibraryIndexState index;

  /// The space either side of the text.
  final double gutter;

  /// Called with the offset in the text a double tap on it fell on.
  final ValueChanged<int>? onEditAt;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final now = clock.now();
    return _PreviewSelectionTheme(
      child: SelectionArea(
        child: CustomScrollView(
          slivers: <Widget>[
            SliverPadding(
              padding: EdgeInsets.fromLTRB(
                gutter,
                KansoSpacing.xxs,
                gutter,
                KansoSpacing.md,
              ),
              sliver: SliverToBoxAdapter(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(
                      note.title.trim().isEmpty
                          ? l10n.noteUntitled
                          : note.title,
                      style: context.typography.previewTitle,
                    ),
                    const SizedBox(height: KansoSpacing.sm),
                    NoteMetadataBlock(
                      notebook: index.notebookPath(note.notebookId) ?? '',
                      status: noteStatusOf(note.status),
                      tags: <TagRow>[
                        for (final id in note.tags)
                          if (index.tag(id) case final tag?) tag,
                      ],
                      created: dateLabel(l10n, note.createdMs),
                      updated: dateTimeLabel(l10n, note.updatedMs, now: now),
                    ),
                  ],
                ),
              ),
            ),
            SliverPadding(
              padding: EdgeInsets.fromLTRB(
                gutter,
                0,
                gutter,
                MediaQuery.paddingOf(context).bottom + KansoSpacing.lg,
              ),
              sliver: NotePreview(note: note, lines: lines, onEditAt: onEditAt),
            ),
            // A person's or project's live sections; nothing for other notes.
            if (getIt.isRegistered<PeopleRepository>())
              SliverToBoxAdapter(child: PeoplePagePanel(noteId: note.id)),
          ],
        ),
      ),
    );
  }
}

/// The selection under it in the accent of the preview theme in force: its
/// highlight, its caret and its handles, which the selection area draws
/// from the themes above it. Without an [AppearanceBloc] (a widget test
/// that has none) it is the app's.
class _PreviewSelectionTheme extends StatelessWidget {
  const _PreviewSelectionTheme({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final dark = context.brightness == Brightness.dark;
    final palette = context.select<AppearanceBloc?, PaletteDto?>(
      (bloc) => switch (bloc?.state) {
        null => null,
        final state => (dark ? state.dark : state.light).preview,
      },
    );
    if (palette == null) {
      return child;
    }
    final accent = previewColor(palette.accent);
    final highlight = accent.withValues(alpha: 0.3);
    return DefaultSelectionStyle(
      selectionColor: highlight,
      cursorColor: accent,
      child: TextSelectionTheme(
        data: TextSelectionThemeData(
          cursorColor: accent,
          selectionColor: highlight,
          selectionHandleColor: accent,
        ),
        child: CupertinoTheme(
          data: CupertinoThemeData(selectionHandleColor: accent),
          child: child,
        ),
      ),
    );
  }
}
