// Kanso — note_discovery_section.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_chip.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/note_discovery/note_discovery_bloc.dart';
import 'note_twin_card.dart';

/// Related notes, suggested tags and a close note twin for an open note.
class NoteDiscoverySection extends StatelessWidget {
  const NoteDiscoverySection({required this.note, super.key});

  final NoteView note;

  @override
  Widget build(BuildContext context) {
    if (note.lock is LockState_Closed ||
        (note.body?.contains('#ai-exclude') ?? false)) {
      return const SizedBox.shrink();
    }
    return BlocBuilder<NoteDiscoveryBloc, NoteDiscoveryState>(
      builder: (context, state) {
        final l10n = context.l10n;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoGroupedSection(
              title: l10n.noteRelatedTitle,
              children: <Widget>[
                if (!state.relatedEnabled)
                  Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(
                      l10n.noteRelatedTurnOn,
                      style: context.typography.secondary,
                    ),
                  )
                else if (state.related.isEmpty)
                  Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(
                      l10n.noteRelatedEmpty,
                      style: context.typography.secondary,
                    ),
                  )
                else
                  for (final related in state.related)
                    KansoSettingsRow(
                      label: related.title,
                      value: related.label,
                      chevron: true,
                      onTap: () {
                        Navigator.of(context).pop();
                        GoRouter.of(
                          context,
                        ).push('${KansoRoutes.noteById}/${related.id}');
                      },
                    ),
              ],
            ),
            if (state.meaning case final meaning?) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              KansoGroupedSection(
                title: l10n.noteMeaning,
                children: <Widget>[
                  if (meaning.tags.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: Wrap(
                        spacing: KansoSpacing.xs,
                        runSpacing: KansoSpacing.xs,
                        children: <Widget>[
                          for (final tag in meaning.tags.where(
                            (tag) => !note.tags.contains(tag.tagId),
                          ))
                            KansoChip(
                              label: tag.name,
                              onTap: () =>
                                  context.read<NoteDiscoveryBloc>().add(
                                    NoteDiscoveryEvent.tagAdded(tag.tagId),
                                  ),
                            ),
                        ],
                      ),
                    ),
                  if (meaning.twin case final twin?) NoteTwinCard(twin: twin),
                ],
              ),
            ],
            const SizedBox(height: KansoSpacing.sm),
            KansoSettingsRow(
              label: l10n.graphLocal,
              chevron: true,
              onTap: () =>
                  GoRouter.of(context).push(KansoRoutes.graphLocal(note.id)),
            ),
          ],
        );
      },
    );
  }
}
