// Kanso: import_progress_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_card.dart';
import '../../models/data/import_stage.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import 'import_progress_bar.dart';

/// While the export is read, and while the notes are written: what is
/// being done, a bar and the library's own words for how far it has got. It
/// needs an [ImportFlowBloc] above it.
class ImportProgressView extends StatelessWidget {
  const ImportProgressView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      builder: (context, state) {
        final writing = state.stage == ImportStage.importing;
        final progress = state.progress;
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 340),
              child: KansoCard(
                padding: const EdgeInsets.all(KansoSpacing.lg),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    Text(
                      writing
                          ? l10n.importWritingTitle
                          : l10n.importReadingTitle,
                      textAlign: TextAlign.center,
                      style: typography.heading2,
                    ),
                    const SizedBox(height: KansoSpacing.xs),
                    Text(
                      writing
                          ? l10n.importWritingMessage
                          : l10n.importReadingMessage,
                      textAlign: TextAlign.center,
                      style: typography.secondary,
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    ImportProgressBar(fraction: progress?.fraction ?? 0),
                    const SizedBox(height: KansoSpacing.xs),
                    Semantics(
                      liveRegion: true,
                      child: Text(
                        progress?.label ?? '',
                        textAlign: TextAlign.center,
                        style: typography.meta,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
