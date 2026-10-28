// Kanso — ai_settings_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/ai_settings/ai_settings_bloc.dart';
import 'ai_settings_form.dart';

/// Settings › AI: a back button and the form, once the settings are read. It
/// reads the `AiSettingsBloc` above it.
class AiSettingsPane extends StatelessWidget {
  const AiSettingsPane({super.key});

  /// How many bars the page shows while the settings are read.
  static const int loadingRows = 6;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.aiTitle,
          ),
          Expanded(
            child: BlocBuilder<AiSettingsBloc, AiSettingsState>(
              builder: (context, state) {
                return switch (state.status) {
                  LoadStatus.loading => Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Column(
                      children: <Widget>[
                        for (var i = 0; i < loadingRows; i++)
                          const Padding(
                            padding: EdgeInsets.symmetric(
                              vertical: KansoSpacing.sm,
                            ),
                            child: KansoSkeleton(height: 14),
                          ),
                      ],
                    ),
                  ),
                  LoadStatus.failed => Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        icon: KansoIcons.sparkles,
                        title: l10n.aiLoadFailedTitle,
                        message: l10n.aiLoadFailedMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => context.read<AiSettingsBloc>().add(
                              const AiSettingsEvent.started(),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  // A different provider is a different form: its fields
                  // start empty, from the provider's own defaults.
                  LoadStatus.ready => AiSettingsForm(
                    key: ValueKey<String>(state.settings.provider),
                    state: state,
                  ),
                };
              },
            ),
          ),
        ],
      ),
    );
  }
}
