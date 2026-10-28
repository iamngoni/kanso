// Kanso — voice_recognition_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../dictation/models/data/dictation_capability.dart';

/// Says whether the phone turns speech into text by itself, for the
/// language it is set to.
class VoiceRecognitionRow extends StatelessWidget {
  const VoiceRecognitionRow({required this.capability, super.key});

  final DictationCapability capability;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final language = capability.language;
    return KansoSettingsRow(
      icon: capability.available ? KansoIcons.circleCheck : KansoIcons.circleX,
      label: !capability.available
          ? l10n.dictationRecognizerUnavailable(language)
          : capability.onDevice
          ? l10n.dictationRecognizerReady(language)
          : l10n.dictationRecognizerSystem(language),
    );
  }
}
