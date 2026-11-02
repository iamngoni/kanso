// Kanso — review_library_row.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';

/// The library's Review row, with the number of cards waiting. It is drawn
/// only while some note holds a card.
class ReviewLibraryRow extends StatefulWidget {
  const ReviewLibraryRow({super.key});

  @override
  State<ReviewLibraryRow> createState() => _ReviewLibraryRowState();
}

class _ReviewLibraryRowState extends State<ReviewLibraryRow> {
  int _total = 0;
  int _waiting = 0;

  @override
  void initState() {
    super.initState();
    try {
      getIt<KansoLibrary>()
          .cards()
          .counts(
            scope: FlashcardsText.everything,
            utcOffsetMinutes: DateTime.now().timeZoneOffset.inMinutes,
          )
          .then((counts) {
            if (mounted) {
              setState(() {
                _total = counts.total;
                _waiting = counts.waiting;
              });
            }
          })
          .catchError((Object _) {});
    } on Object {
      // No library to ask (a preview): the row stays hidden.
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_total == 0) {
      return const SizedBox.shrink();
    }
    return KansoSidebarRow(
      icon: KansoIcons.bookOpen,
      label: FlashcardsText.reviewTitle,
      count: _waiting == 0 ? null : _waiting,
      onTap: () {
        KansoDrawerScope.maybeOf(context)?.close();
        context.push(KansoRoutes.flashcards);
      },
    );
  }
}
