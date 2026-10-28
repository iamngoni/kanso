// Kanso — account_pane_body.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/account_details/account_details_bloc.dart';
import '../../utils/account_actions.dart';
import '../../utils/account_labels.dart';
import 'account_device_row.dart';
import 'account_linking_card.dart';
import 'account_passkey_row.dart';
import 'account_provider_row.dart';

/// The account page once it is read.
class AccountPaneBody extends StatelessWidget {
  const AccountPaneBody({
    required this.state,
    required this.details,
    super.key,
  });

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  final AccountDetailsState state;
  final AccountDetails details;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final idle = state.busy == null;
    final notice = state.notice;
    final hasPassword = details.hasPassword;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Text(details.username, style: typography.largeTitle),
        const SizedBox(height: KansoSpacing.xxs),
        Text(hostOf(details.server), style: typography.secondary),
        const SizedBox(height: KansoSpacing.md),
        if (state.busy != null && state.linking == null) ...<Widget>[
          KansoInlineMessage(
            message: l10n.accountWorking,
            tone: KansoMessageTone.info,
          ),
          const SizedBox(height: KansoSpacing.md),
        ],
        if (state.problem != null || state.error != null) ...<Widget>[
          KansoInlineMessage(
            message: state.error != null
                ? l10n.accountError(state.error!)
                : l10n.accountDeleteMismatch,
          ),
          const SizedBox(height: KansoSpacing.md),
        ],
        if (notice != null && state.error == null) ...<Widget>[
          KansoInlineMessage(
            message: l10n.accountNotice(notice),
            tone: KansoMessageTone.success,
          ),
          const SizedBox(height: KansoSpacing.md),
        ],
        if (state.linking != null) ...<Widget>[
          AccountLinkingCard(provider: state.linking!, code: state.code),
          const SizedBox(height: KansoSpacing.lg),
        ],
        if (details.recoveryKeyUnsaved) ...<Widget>[
          KansoGroupedSection(
            dividerIndent: iconInset,
            footer: l10n.accountRecoveryKeyUnsavedMessage,
            children: <Widget>[
              KansoSettingsRow(
                icon: KansoIcons.shield,
                label: l10n.accountRecoveryKeyUnsavedTitle,
                value: l10n.accountRecoveryKeyShow,
                chevron: true,
                onTap: () => AccountActions.showRecoveryKey(context),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.lg),
        ],
        KansoGroupedSection(
          title: l10n.accountDevices,
          dividerIndent: iconInset,
          children: <Widget>[
            for (final device in details.devices)
              AccountDeviceRow(
                device: device,
                onSignOut: idle
                    ? () => AccountActions.revokeDevice(context, device)
                    : null,
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.lg),
        KansoGroupedSection(
          title: l10n.accountWaysToSignIn,
          dividerIndent: iconInset,
          footer: hasPassword ? null : l10n.accountNoProofHere,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.lock,
              label: l10n.accountPasswordRow,
              value: hasPassword ? null : l10n.accountPasswordNotSet,
              trailing: hasPassword
                  ? KansoButton(
                      label: l10n.accountPasswordChange,
                      variant: KansoButtonVariant.quiet,
                      expanded: false,
                      compact: true,
                      onPressed: idle
                          ? () => AccountActions.changePassword(context)
                          : null,
                    )
                  : null,
            ),
            for (final row in details.providers)
              AccountProviderRow(
                provider: row.provider,
                linked: row.linked,
                onPressed: idle && hasPassword
                    ? () => row.linked
                          ? AccountActions.unlinkProvider(context, row.provider)
                          : AccountActions.linkProvider(context, row.provider)
                    : null,
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.lg),
        KansoGroupedSection(
          title: l10n.accountPasskeys,
          dividerIndent: iconInset,
          children: <Widget>[
            for (final passkey in details.passkeys)
              AccountPasskeyRow(
                passkey: passkey,
                onRemove: idle && hasPassword
                    ? () => AccountActions.removePasskey(context, passkey)
                    : null,
              ),
            KansoSettingsRow(
              icon: KansoIcons.plus,
              label: l10n.accountPasskeyAdd,
              onTap: idle && hasPassword
                  ? () => AccountActions.addPasskey(context)
                  : null,
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.lg),
        KansoGroupedSection(
          title: l10n.accountRecoveryKeySection,
          footer: l10n.accountRecoveryKeyFooter,
          dividerIndent: iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.shield,
              label: l10n.accountRecoveryKeyNew,
              chevron: true,
              onTap: idle && hasPassword
                  ? () => AccountActions.newRecoveryKey(context)
                  : null,
            ),
          ],
        ),
        if (details.storageLine != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.lg),
          KansoGroupedSection(
            dividerIndent: iconInset,
            children: <Widget>[
              KansoDetailRow(
                icon: KansoIcons.cloud,
                title: l10n.accountStorage,
                subtitle: details.storageLine,
              ),
            ],
          ),
        ],
        const SizedBox(height: KansoSpacing.lg),
        KansoGroupedSection(
          children: <Widget>[
            KansoSettingsRow(
              label: l10n.accountSignOut,
              centered: true,
              onTap: idle
                  ? () => AccountActions.signOut(
                      context,
                      username: details.username,
                      keyUnsaved: details.recoveryKeyUnsaved,
                    )
                  : null,
            ),
            KansoSettingsRow(
              label: l10n.accountDeleteAccount,
              danger: true,
              centered: true,
              onTap: idle && hasPassword
                  ? () => AccountActions.deleteAccount(
                      context,
                      username: details.username,
                    )
                  : null,
            ),
          ],
        ),
      ],
    );
  }
}
