// Kanso — voice_settings_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../dictation/models/data/dictation_permission.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/voice_settings/voice_settings_bloc.dart';
import 'voice_recognition_row.dart';

/// Settings › Voice: how dictation works, the switch for spoken punctuation
/// and commands, whether the phone can recognize speech by itself, and,
/// when the microphone is refused, the way to Settings to allow it. There is
/// no voice model to download: the phone's own recognizer does the work.
class VoiceSettingsPane extends StatelessWidget {
  const VoiceSettingsPane({super.key});

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  /// Takes the person to the phone's Settings, to allow the microphone.
  static void openSettings(BuildContext context) {
    const request = VoiceSettingsEvent.systemSettingsRequested();
    context.read<VoiceSettingsBloc>().add(request);
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.dictationSettingsTitle,
          ),
          Expanded(
            child: BlocBuilder<VoiceSettingsBloc, VoiceSettingsState>(
              builder: (context, state) {
                if (state.status == LoadStatus.failed) {
                  return Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        title: l10n.dictationErrorTitle,
                        message: l10n.dictationErrorMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => context
                                .read<VoiceSettingsBloc>()
                                .add(const VoiceSettingsEvent.started()),
                          ),
                        ],
                      ),
                    ),
                  );
                }
                final capability = state.capability;
                final refused =
                    capability != null &&
                    (capability.permission == DictationPermission.denied ||
                        capability.permission ==
                            DictationPermission.restricted);
                return ListView(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    KansoGroupedSection(
                      title: l10n.dictationSettingsInputSection,
                      footer: l10n.dictationSettingsInputDetail,
                      dividerIndent: iconInset,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.mic,
                          label: l10n.dictationSettingsInput,
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.hash,
                          label: l10n.dictationSettingsCommands,
                          trailing: KansoSwitch(
                            value: state.settings.spokenCommands,
                            onChanged: state.status == LoadStatus.ready
                                ? (on) => context.read<VoiceSettingsBloc>().add(
                                    VoiceSettingsEvent.spokenCommandsChanged(
                                      on: on,
                                    ),
                                  )
                                : null,
                          ),
                          onTap: state.status == LoadStatus.ready
                              ? () => context.read<VoiceSettingsBloc>().add(
                                  VoiceSettingsEvent.spokenCommandsChanged(
                                    on: !state.settings.spokenCommands,
                                  ),
                                )
                              : null,
                        ),
                      ],
                    ),
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        0,
                      ),
                      child: Text(
                        l10n.dictationSettingsCommandsDetail,
                        style: context.typography.meta.copyWith(
                          color: colors.textSecondary,
                        ),
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    if (capability != null)
                      KansoGroupedSection(
                        title: l10n.dictationSettingsRecognitionSection,
                        dividerIndent: iconInset,
                        children: <Widget>[
                          VoiceRecognitionRow(capability: capability),
                          if (refused)
                            KansoSettingsRow(
                              icon: KansoIcons.micOff,
                              label: l10n.dictationSettingsMicDenied,
                              value: l10n.dictationOpenSettings,
                              external: true,
                              onTap:
                                  capability.permission ==
                                      DictationPermission.restricted
                                  ? null
                                  : () => openSettings(context),
                            ),
                        ],
                      ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
