// Kanso — note_twin_card.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../state/note_discovery/note_discovery_bloc.dart';

/// A note that is nearly the same as the one currently open.
class NoteTwinCard extends StatelessWidget {
  const NoteTwinCard({required this.twin, super.key});

  final TwinRow twin;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.all(KansoSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(
            l10n.noteMeaningTwin(twin.title, twin.percent),
            style: context.typography.body,
          ),
          const SizedBox(height: KansoSpacing.xs),
          Wrap(
            spacing: KansoSpacing.xs,
            children: <Widget>[
              TextButton(
                onPressed: () => context.read<NoteDiscoveryBloc>().add(
                  NoteDiscoveryEvent.twinDismissed(twin.id),
                ),
                child: Text(l10n.noteMeaningDismiss),
              ),
              TextButton(
                onPressed: () async {
                  final merge = await showDialog<bool>(
                    context: context,
                    builder: (dialogContext) => AlertDialog(
                      title: Text(l10n.noteMeaningMergeTitle),
                      content: Text(l10n.noteMeaningMergeWarning(twin.title)),
                      actions: <Widget>[
                        TextButton(
                          onPressed: () =>
                              Navigator.of(dialogContext).pop(false),
                          child: Text(l10n.cancel),
                        ),
                        TextButton(
                          onPressed: () =>
                              Navigator.of(dialogContext).pop(true),
                          child: Text(l10n.noteMeaningMerge),
                        ),
                      ],
                    ),
                  );
                  if ((merge ?? false) && context.mounted) {
                    context.read<NoteDiscoveryBloc>().add(
                      NoteDiscoveryEvent.notesMerged(twin.id),
                    );
                  }
                },
                child: Text(l10n.noteMeaningMerge),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
