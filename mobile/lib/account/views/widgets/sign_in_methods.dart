// Kanso — sign_in_methods.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../models/data/sign_in_action.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import '../../utils/account_labels.dart';

/// The buttons of the ways in that take no typing, for the ways the server
/// offers: Apple, Google and GitHub, and a passkey. It needs a [SignInBloc]
/// above it.
///
/// [creating] words the passkey button for making an account.
class SignInMethods extends StatelessWidget {
  const SignInMethods({
    required this.state,
    required this.methods,
    this.creating = false,
    this.enabled = true,
    this.passkeyUsername,
    super.key,
  });

  final SignInState state;
  final ServerMethods methods;
  final bool creating;

  /// Whether the buttons can be pressed at all.
  final bool enabled;

  /// The username typed so far, for a passkey sign-up.
  final String Function()? passkeyUsername;

  /// Whether any of these buttons shows for [methods].
  static bool shows(ServerMethods methods) =>
      methods.providers.isNotEmpty || methods.passkeys;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    final deviceName = deviceNameOf(context);
    final idle = enabled && state.busy == null;
    final children = <Widget>[
      if (!creating)
        for (final provider in methods.providers)
          KansoButton(
            label: state.busy == SignInAction.ofProvider(provider)
                ? l10n.accountSigningIn
                : l10n.continueWith(provider),
            icon: provider == ProviderKind.github ? KansoIcons.github : null,
            variant: KansoButtonVariant.secondary,
            onPressed: idle
                ? () => bloc.add(
                    SignInEvent.providerPressed(
                      provider: provider,
                      deviceName: deviceName,
                    ),
                  )
                : null,
          ),
      if (methods.passkeys)
        KansoButton(
          label: state.busy == SignInAction.passkey
              ? l10n.accountSigningIn
              : creating
              ? l10n.accountCreateWithPasskey
              : l10n.accountSignInWithPasskey,
          icon: KansoIcons.fingerprint,
          variant: KansoButtonVariant.secondary,
          onPressed: idle
              ? () => bloc.add(
                  creating
                      ? SignInEvent.passkeySignUpPressed(
                          username: passkeyUsername?.call() ?? '',
                          deviceName: deviceName,
                        )
                      : SignInEvent.passkeySignInPressed(
                          deviceName: deviceName,
                        ),
                )
              : null,
        ),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        for (var i = 0; i < children.length; i++) ...<Widget>[
          if (i > 0) const SizedBox(height: KansoSpacing.xs),
          children[i],
        ],
      ],
    );
  }
}
