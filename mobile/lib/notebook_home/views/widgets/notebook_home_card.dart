// Kanso — notebook_home_card.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../models/repos/abs/notebook_home_repository.dart';
import 'home_sheets.dart';

/// Where notebook [bookId]'s homepage is, as the address of the app.
String notebookHomeLocation(String bookId) =>
    BrowseLocation.place(ListPlace.notebook(id: bookId), home: true).location;

/// Opens notebook [bookId]'s homepage: a page on a phone, the note pane
/// beside the list on a wide window ([beside]).
void openNotebookHome(
  BuildContext context,
  String bookId, {
  required bool beside,
  bool replace = false,
}) {
  final location = notebookHomeLocation(bookId);
  if (beside) {
    context.go(location);
  } else if (replace) {
    unawaited(context.pushReplacement<void>(location));
  } else {
    unawaited(context.push<void>(location));
  }
}

/// The card above a notebook's notes: its name, what it is about (or "Add a
/// Home note"), and "Open home ›".
///
/// It reads the card after the list is drawn and takes no room until it
/// has, so scrolling the list never waits on it. It draws nothing where the
/// core says there is no card (a locked notebook, or the setting off).
class NotebookHomeCard extends StatefulWidget {
  const NotebookHomeCard({
    required this.bookId,
    this.beside = false,
    super.key,
  });

  final String bookId;

  /// Whether the list is beside the note pane, which then shows the page.
  final bool beside;

  @override
  State<NotebookHomeCard> createState() => _NotebookHomeCardState();
}

class _NotebookHomeCardState extends State<NotebookHomeCard> {
  final NotebookHomeRepository _home = getIt<NotebookHomeRepository>();
  HomeCard? _card;
  HomeWords? _words;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  @override
  void didUpdateWidget(NotebookHomeCard old) {
    super.didUpdateWidget(old);
    if (old.bookId != widget.bookId) {
      _card = null;
      unawaited(_load());
    }
  }

  Future<void> _load() async {
    final id = widget.bookId;
    final card = (await _home.card(id)).toOption().toNullable();
    final words = _words ?? (await _home.words()).toOption().toNullable();
    if (mounted && id == widget.bookId) {
      setState(() {
        _card = card;
        _words = words;
      });
    }
  }

  Future<void> _addHome() async {
    final l10n = context.l10n;
    final bookId = widget.bookId;
    final choice = await showKansoBottomSheet<bool>(
      context,
      builder: (sheet) => Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.md),
        child: KansoSheetGroup(
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.fileText,
              label: l10n.notebookHomeChooseHome,
              onTap: () => Navigator.of(sheet).pop(true),
            ),
            KansoSettingsRow(
              icon: KansoIcons.plus,
              label: l10n.notebookHomeNewHome,
              onTap: () => Navigator.of(sheet).pop(false),
            ),
          ],
        ),
      ),
    );
    if (choice == null || !mounted) {
      return;
    }
    if (choice) {
      final found = (await _home.candidates(bookId)).toOption().toNullable();
      if (!mounted) {
        return;
      }
      final picked = await showHomeCandidatesSheet(
        context,
        found ?? const <HomeCandidate>[],
      );
      if (picked != null) {
        await _home.chooseHome(bookId, picked);
        await _load();
      }
    } else {
      final made = (await _home.newHome(bookId)).toOption().toNullable();
      if (made != null && mounted) {
        await context.push(citedNoteLocation(made));
        if (mounted) {
          await _load();
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final card = _card;
    final words = _words;
    if (card == null || words == null || !card.show_) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    final typography = context.typography;
    final about = card.aboutLine.isNotEmpty;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.sm,
      ),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: colors.fill,
          borderRadius: KansoRadii.mediumBorder,
        ),
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(card.name, style: typography.label),
              const SizedBox(height: KansoSpacing.xxs),
              if (about)
                Text(
                  card.aboutLine,
                  maxLines: 3,
                  overflow: TextOverflow.ellipsis,
                  style: typography.secondary.copyWith(
                    color: colors.textMuted,
                  ),
                )
              else
                KansoPressable(
                  onTap: () => unawaited(_addHome()),
                  child: Text(
                    words.addHomeNote,
                    style: typography.secondary.copyWith(color: colors.link),
                  ),
                ),
              const SizedBox(height: KansoSpacing.xs),
              KansoPressable(
                onTap: () => openNotebookHome(
                  context,
                  widget.bookId,
                  beside: widget.beside,
                ),
                child: Text(
                  '${words.openHome} ›',
                  style: typography.label.copyWith(color: colors.link),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
