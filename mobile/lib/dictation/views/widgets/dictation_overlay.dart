// Kanso — dictation_overlay.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/dictation_status.dart';
import '../../models/data/dictation_unavailable_reason.dart';
import '../../state/dictation/dictation_bloc.dart';
import 'dictation_pill.dart';

/// Shows the [DictationPill] above the keyboard's accessory bar while the
/// editor's key is dictating, and the notice when it could not: what the
/// editor's own files do not have to know about.
class DictationOverlay extends StatelessWidget {
  const DictationOverlay({required this.preview, super.key});

  /// The words being spoken, as they would be written in the note.
  final ValueListenable<String> preview;

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
    final bloc = context.read<DictationBloc>();
    final l10n = context.l10n;
    return BlocBuilder<DictationBloc, DictationState>(
      builder: (context, state) {
        if (!state.showOverlay) {
          return const SizedBox.shrink();
        }
        final Widget? pill = switch (state.status) {
          DictationStatus.idle => null,
          DictationStatus.requesting ||
          DictationStatus.listening => DictationPill(
            text: state.partial.trim().isEmpty
                ? l10n.dictationListening
                : state.partial.trim(),
            quiet: state.partial.trim().isEmpty,
            actionLabel: l10n.dictationStop,
            onAction: () => bloc.add(const DictationEvent.stopped()),
          ),
          DictationStatus.denied => DictationPill(
            icon: KansoIcons.micOff,
            text: state.restricted
                ? l10n.dictationRestrictedMessage
                : l10n.dictationDeniedMessage,
            actionLabel: state.restricted
                ? l10n.dictationDismiss
                : l10n.dictationOpenSettings,
            onAction: () => bloc.add(
              state.restricted
                  ? const DictationEvent.cancelled()
                  : const DictationEvent.settingsRequested(),
            ),
          ),
          DictationStatus.unavailable => DictationPill(
            icon: KansoIcons.micOff,
            text: switch (state.reason) {
              DictationUnavailableReason.notOnDevice =>
                l10n.dictationUnavailableNotOnDevice,
              DictationUnavailableReason.unsupportedLanguage =>
                l10n.dictationUnavailableLanguage,
              _ => l10n.dictationUnavailableNoRecognizer,
            },
            actionLabel: l10n.dictationDismiss,
            onAction: () => bloc.add(const DictationEvent.cancelled()),
          ),
          DictationStatus.error => DictationPill(
            icon: KansoIcons.info,
            text: l10n.dictationErrorMessage,
            actionLabel: l10n.dictationDismiss,
            onAction: () => bloc.add(const DictationEvent.cancelled()),
          ),
        };
        if (pill == null) {
          return const SizedBox.shrink();
        }
        // Above the keyboard and the accessory bar that sits on it.
        final floor = media.viewInsets.bottom > 0
            ? media.viewInsets.bottom + KansoSizes.touchTarget
            : media.padding.bottom;
        return Positioned(
          left: KansoSpacing.md,
          right: KansoSpacing.md,
          bottom: floor + KansoSpacing.sm,
          child: Align(child: pill),
        );
      },
    );
  }
}
