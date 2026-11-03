// Kanso: apple_notes_guide_sheet.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/configs/kanso_links.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// The honest ways to bring Apple Notes onto a phone or tablet, which can't
/// read the Notes library: import on a Mac and sync (recommended), share
/// notes one at a time, or choose notes exported as files. Pops with `true`
/// when the person chooses to pick exported files.
class AppleNotesGuideSheet extends StatelessWidget {
  const AppleNotesGuideSheet({super.key});

  /// Shows the sheet; completes with `true` to choose exported files, else
  /// `null`.
  static Future<bool?> open(BuildContext context) {
    return showKansoBottomSheet<bool>(
      context,
      builder: (_) => const AppleNotesGuideSheet(),
    );
  }

  Future<void> _openHelp() async {
    try {
      await launchUrl(
        KansoLinks.importAppleNotes,
        mode: LaunchMode.externalApplication,
      );
    } on Object {
      // The guide is still readable if no browser can open the link.
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return SingleChildScrollView(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        bottom + KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(l10n.importAppleNotesTitle, style: typography.heading2),
          const SizedBox(height: KansoSpacing.xs),
          Text(l10n.importAppleNotesIntro, style: typography.body),
          const SizedBox(height: KansoSpacing.md),
          _Option(
            title: l10n.importAppleNotesMacTitle,
            body: l10n.importAppleNotesMacBody,
          ),
          const SizedBox(height: KansoSpacing.md),
          _Option(
            title: l10n.importAppleNotesShareTitle,
            body: l10n.importAppleNotesShareBody,
          ),
          const SizedBox(height: KansoSpacing.md),
          _Option(
            title: l10n.importAppleNotesFilesTitle,
            body: l10n.importAppleNotesFilesBody,
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: l10n.importAppleNotesFilesAction,
            onPressed: () => Navigator.of(context).pop(true),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.importAppleNotesHelp,
            variant: KansoButtonVariant.quiet,
            onPressed: _openHelp,
          ),
        ],
      ),
    );
  }
}

class _Option extends StatelessWidget {
  const _Option({required this.title, required this.body});

  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    final typography = context.typography;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Text(title, style: typography.label),
        const SizedBox(height: KansoSpacing.xxs),
        Text(body, style: typography.secondary),
      ],
    );
  }
}
