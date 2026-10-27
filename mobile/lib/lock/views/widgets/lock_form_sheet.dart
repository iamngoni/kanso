// Kanso — lock_form_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// The layout every lock sheet shares: a [title], what it [message]s, the
/// [fields], a [notice] (an error or a result) and the primary button with
/// Cancel under it. It rises above the keyboard.
///
/// With no [onPrimary] the primary button is drawn dimmed; while [working]
/// neither button reacts.
class LockFormSheet extends StatelessWidget {
  const LockFormSheet({
    required this.title,
    required this.primaryLabel,
    required this.onPrimary,
    this.message,
    this.fields = const <Widget>[],
    this.notice,
    this.noticeIsError = true,
    this.working = false,
    this.danger = false,
    this.onCancel,
    this.cancelLabel,
    super.key,
  });

  final String title;
  final String? message;
  final List<Widget> fields;
  final String? notice;

  /// Whether [notice] is drawn as an error.
  final bool noticeIsError;
  final String primaryLabel;
  final VoidCallback? onPrimary;
  final bool working;
  final bool danger;

  /// What Cancel does; closing the sheet when null.
  final VoidCallback? onCancel;

  /// The label of the way out; Cancel when null.
  final String? cancelLabel;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final text = notice;
    final body = message;
    return SingleChildScrollView(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        KansoSpacing.md + MediaQuery.viewInsetsOf(context).bottom,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(title, style: typography.heading2),
          if (body != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            Text(body, style: typography.secondary),
          ],
          for (final field in fields) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            field,
          ],
          if (text != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            Semantics(
              liveRegion: true,
              child: Text(
                text,
                style: typography.secondary.copyWith(
                  color: noticeIsError ? colors.danger : null,
                ),
              ),
            ),
          ],
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: primaryLabel,
            variant: danger
                ? KansoButtonVariant.danger
                : KansoButtonVariant.primary,
            onPressed: working ? null : onPrimary,
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: cancelLabel ?? l10n.lockCancel,
            variant: KansoButtonVariant.quiet,
            onPressed: working
                ? null
                : (onCancel ?? () => Navigator.of(context).maybePop()),
          ),
        ],
      ),
    );
  }
}
