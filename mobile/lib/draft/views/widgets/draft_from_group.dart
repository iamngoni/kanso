// Kanso — draft_from_group.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../../library/views/widgets/library_message_sheet.dart';

/// Which notes a draft reads.
sealed class DraftSource {
  const DraftSource();
}

/// The notes chosen in a list.
class DraftSourceNotes extends DraftSource {
  const DraftSourceNotes(this.ids);

  final List<String> ids;
}

/// Every note of a notebook and the notebooks inside it.
class DraftSourceNotebook extends DraftSource {
  const DraftSourceNotebook(this.id);

  final String id;
}

/// Every note with a tag.
class DraftSourceTag extends DraftSource {
  const DraftSourceTag(this.id);

  final String id;
}

/// The notes [source] names, as the draft sheet reads them.
Future<List<String>> draftNoteIds(DraftSource source) {
  final drafting = getIt<KansoLibrary>().drafting();
  return switch (source) {
    DraftSourceNotes(:final ids) => Future<List<String>>.value(ids),
    DraftSourceNotebook(:final id) => drafting.notesOfNotebook(bookId: id),
    DraftSourceTag(:final id) => drafting.notesOfTag(tagId: id),
  };
}

/// "Draft from These Notes…" for a menu: one row that opens the draft page
/// for [source]. It is drawn only while AI is on; a notebook or tag with no
/// notes says so instead of opening an empty page.
class DraftFromGroup extends StatelessWidget {
  const DraftFromGroup({required this.source, required this.host, super.key});

  final DraftSource source;

  /// The root navigator's context, which outlives the menu.
  final BuildContext host;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<AiStatusBloc, AiStatusState>(
      buildWhen: (before, now) => before.enabled != now.enabled,
      builder: (context, state) {
        if (!state.enabled) {
          return const SizedBox.shrink();
        }
        final label = switch (source) {
          DraftSourceNotes(:final ids) => l10n.flashcardsDraftFromNotes(
            ids.length,
          ),
          DraftSourceNotebook() => l10n.flashcardsDraftFromNotebook,
          DraftSourceTag() => l10n.flashcardsDraftFromTag,
        };
        final router = GoRouter.of(host);
        return KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.penTool,
              label: label,
              onTap: () async {
                Navigator.of(context).pop();
                final emptyMessage = l10n.flashcardsDraftNoNotes;
                try {
                  final ids = await draftNoteIds(source);
                  if (ids.isEmpty) {
                    if (host.mounted) {
                      await LibraryMessageSheet.show(host, emptyMessage);
                    }
                    return;
                  }
                  await router.push<void>(KansoRoutes.draft, extra: ids);
                } on Object catch (_) {
                  if (host.mounted) {
                    await LibraryMessageSheet.show(host, emptyMessage);
                  }
                }
              },
            ),
          ],
        );
      },
    );
  }
}
