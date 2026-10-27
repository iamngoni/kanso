// Kanso — account_server_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import '../../utils/account_labels.dart';

/// The server being signed in to, as a row: "Kanso sync" for the default and
/// the host for any other, which opens a field for another server's address.
/// It needs a [SignInBloc] above it.
class AccountServerRow extends StatelessWidget {
  const AccountServerRow({required this.state, super.key});

  final SignInState state;

  /// How far the row's hairline is inset: past its icon.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    final busy = state.busy != null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        KansoGroupedSection(
          dividerIndent: iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.globe,
              label: l10n.accountServerLabel,
              value: l10n.serverName(
                state.server,
                defaultServer: state.defaultServer,
              ),
              chevron: true,
              onTap: busy
                  ? null
                  : () => bloc.add(
                      SignInEvent.serverEdited(editing: !state.editingServer),
                    ),
            ),
          ],
        ),
        if (state.editingServer) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          KansoFieldControllers(
            count: 1,
            initialTexts: <String>[if (state.customServer) state.server],
            builder: (context, controllers) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  KansoTextField(
                    controller: controllers[0],
                    label: l10n.accountServerAddress,
                    hint: l10n.accountServerAddressHint,
                    keyboardType: TextInputType.url,
                    textInputAction: TextInputAction.done,
                    onSubmitted: (address) =>
                        bloc.add(SignInEvent.serverChosen(address)),
                  ),
                  const SizedBox(height: KansoSpacing.sm),
                  KansoButton(
                    label: l10n.accountServerUse,
                    variant: KansoButtonVariant.secondary,
                    onPressed: () =>
                        bloc.add(SignInEvent.serverChosen(controllers[0].text)),
                  ),
                  if (state.customServer)
                    KansoButton(
                      label: l10n.accountServerUseDefault,
                      variant: KansoButtonVariant.quiet,
                      onPressed: () =>
                          bloc.add(const SignInEvent.serverChosen('')),
                    ),
                ],
              );
            },
          ),
        ],
      ],
    );
  }
}
