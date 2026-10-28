// Kanso — ai_provider_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../state/ai_settings/ai_settings_bloc.dart';

/// Opens the list of providers over the page; choosing one saves it and
/// closes the list. The sheet is above the page's navigator, so [bloc] is
/// handed to it.
Future<void> showAiProviderSheet(BuildContext context, AiSettingsBloc bloc) {
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => BlocProvider<AiSettingsBloc>.value(
      value: bloc,
      child: const AiProviderSheet(),
    ),
  );
}

/// The providers answers can come from, each with its description; the one
/// in use has a tick.
class AiProviderSheet extends StatelessWidget {
  const AiProviderSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return BlocBuilder<AiSettingsBloc, AiSettingsState>(
      builder: (context, state) {
        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: Text(
                context.l10n.aiProviderSheetTitle,
                style: typography.heading2,
              ),
            ),
            Flexible(
              child: ListView(
                shrinkWrap: true,
                children: <Widget>[
                  for (final info in state.providers) ...<Widget>[
                    const KansoDivider(indent: KansoSpacing.md),
                    KansoPressable(
                      onTap: () {
                        if (info.key != state.settings.provider) {
                          context.read<AiSettingsBloc>().add(
                            AiSettingsEvent.providerChosen(info.key),
                          );
                        }
                        Navigator.of(context).maybePop();
                      },
                      child: ConstrainedBox(
                        constraints: const BoxConstraints(
                          minHeight: KansoSizes.settingsRow,
                        ),
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: KansoSpacing.md,
                            vertical: KansoSpacing.sm,
                          ),
                          child: Row(
                            children: <Widget>[
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: <Widget>[
                                    Text(info.label, style: typography.body),
                                    const SizedBox(height: KansoSpacing.xxs),
                                    Text(
                                      info.description,
                                      style: typography.meta,
                                    ),
                                  ],
                                ),
                              ),
                              if (info.key == state.settings.provider)
                                KansoIcon(
                                  KansoIcons.check,
                                  color: colors.accent,
                                ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.md),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}
