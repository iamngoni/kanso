// Kanso — account_action_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/account_action_field.dart';

/// A change to the account to be confirmed: a title, what it does, the
/// fields it needs (the current password, a new one, the account's name) and
/// a button that is pressed once every field has something in it.
///
/// [show] completes with what was typed, in the order of the fields (an empty
/// list when there are none), and with null if the sheet was turned away.
class AccountActionSheet extends StatelessWidget {
  const AccountActionSheet({
    required this.title,
    required this.confirmLabel,
    this.message,
    this.warning,
    this.fields = const <AccountActionField>[],
    this.danger = false,
    super.key,
  });

  final String title;
  final String confirmLabel;
  final String? message;

  /// A second paragraph, in the warning colour.
  final String? warning;
  final List<AccountActionField> fields;

  /// Whether the change cannot be undone: the button is drawn as such.
  final bool danger;

  /// Asks for the change in a sheet above the app.
  static Future<List<String>?> show(
    BuildContext context, {
    required String title,
    required String confirmLabel,
    String? message,
    String? warning,
    List<AccountActionField> fields = const <AccountActionField>[],
    bool danger = false,
  }) {
    return showKansoBottomSheet<List<String>>(
      context,
      builder: (context) => AccountActionSheet(
        title: title,
        confirmLabel: confirmLabel,
        message: message,
        warning: warning,
        fields: fields,
        danger: danger,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final note = message;
    final caution = warning;
    return KansoFieldControllers(
      count: fields.length,
      builder: (context, controllers) {
        return SingleChildScrollView(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.md,
            KansoSpacing.md,
            KansoSpacing.md + MediaQuery.viewInsetsOf(context).bottom,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text(title, style: typography.heading2),
              if (note != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                Text(note, style: typography.secondary),
              ],
              if (caution != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                Text(
                  caution,
                  style: typography.secondary.copyWith(color: colors.warning),
                ),
              ],
              for (var i = 0; i < fields.length; i++) ...<Widget>[
                const SizedBox(height: KansoSpacing.sm),
                KansoTextField(
                  controller: controllers[i],
                  label: fields[i].label,
                  obscure: fields[i].obscure,
                  autofocus: i == 0,
                  textInputAction: i == fields.length - 1
                      ? TextInputAction.done
                      : TextInputAction.next,
                ),
              ],
              const SizedBox(height: KansoSpacing.md),
              ListenableBuilder(
                listenable: Listenable.merge(controllers),
                builder: (context, _) {
                  final ready = controllers.every(
                    (controller) => controller.text.isNotEmpty,
                  );
                  return KansoButton(
                    label: confirmLabel,
                    variant: danger
                        ? KansoButtonVariant.danger
                        : KansoButtonVariant.primary,
                    onPressed: ready
                        ? () => Navigator.of(context).pop(<String>[
                            for (final controller in controllers)
                              controller.text,
                          ])
                        : null,
                  );
                },
              ),
              const SizedBox(height: KansoSpacing.xs),
              KansoButton(
                label: l10n.accountCancel,
                variant: KansoButtonVariant.quiet,
                onPressed: () => Navigator.of(context).pop(),
              ),
            ],
          ),
        );
      },
    );
  }
}
