// Kanso — people_library_row.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../../injection.dart';
import '../../models/repos/abs/people_repository.dart';

/// The library's People & Projects row, with the number of suggestions
/// waiting.
class PeopleLibraryRow extends StatefulWidget {
  const PeopleLibraryRow({super.key});

  @override
  State<PeopleLibraryRow> createState() => _PeopleLibraryRowState();
}

class _PeopleLibraryRowState extends State<PeopleLibraryRow> {
  int _suggestions = 0;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final status = await getIt<PeopleRepository>().status();
      final read = status.toOption().toNullable();
      if (read != null && mounted) {
        setState(() => _suggestions = read.suggestions);
      }
    } on Object {
      // No library to ask (a preview): the row shows no count.
    }
  }

  @override
  Widget build(BuildContext context) {
    return KansoSidebarRow(
      icon: KansoIcons.users,
      label: context.l10n.peopleTitle,
      count: _suggestions == 0 ? null : _suggestions,
      onTap: () {
        KansoDrawerScope.maybeOf(context)?.close();
        unawaited(context.push<void>(KansoRoutes.people).then((_) => _load()));
      },
    );
  }
}
