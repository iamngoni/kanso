// Kanso — account_pairing_sheet.dart
// Created 2026-11-06.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../state/account_details/account_details_bloc.dart';

/// The single-use code and terminal command for a screenless computer.
class AccountPairingSheet extends StatelessWidget {
  const AccountPairingSheet({required this.pairing, super.key});

  final PairingCode pairing;

  @override
  Widget build(BuildContext context) {
    final typography = context.typography;
    final command = 'kanso account sign-in --pair ${pairing.code}';
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Text(
              'Add a computer without a screen',
              style: typography.largeTitle,
            ),
            const SizedBox(height: KansoSpacing.xs),
            Text(
              'Run this on the computer you’re adding.\n'
              'The code works once, for 10 minutes.',
              style: typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.lg),
            KansoGroupedSection(
              children: <Widget>[
                Padding(
                  padding: const EdgeInsets.all(KansoSpacing.lg),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      Center(
                        child: SelectableText(
                          pairing.code,
                          style: typography.largeTitle.copyWith(
                            fontFeatures: const <FontFeature>[
                              FontFeature.tabularFigures(),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(height: KansoSpacing.md),
                      SelectableText(
                        command,
                        textAlign: TextAlign.center,
                        style: typography.secondary,
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: KansoSpacing.md),
            BlocBuilder<AccountDetailsBloc, AccountDetailsState>(
              builder: (context, state) => Text(
                state.pairingUsed
                    ? <String>[
                        'The code was used. Compare the six-digit code in the',
                        'approval request.',
                      ].join(' ')
                    : 'Waiting for a computer to use this code…',
                textAlign: TextAlign.center,
                style: typography.secondary,
              ),
            ),
            const SizedBox(height: KansoSpacing.lg),
            KansoButton(
              label: 'Copy command',
              icon: KansoIcons.copy,
              variant: KansoButtonVariant.secondary,
              onPressed: () async {
                await Clipboard.setData(ClipboardData(text: command));
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Command copied')),
                  );
                }
              },
            ),
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: 'Done',
              variant: KansoButtonVariant.quiet,
              onPressed: () => Navigator.of(context).pop(),
            ),
          ],
        ),
      ),
    );
  }
}
