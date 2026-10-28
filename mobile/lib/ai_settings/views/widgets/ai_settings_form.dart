// Kanso — ai_settings_form.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../state/ai_settings/ai_settings_bloc.dart';
import 'ai_field_row.dart';
import 'ai_key_row.dart';
import 'ai_privacy_note.dart';
import 'ai_provider_sheet.dart';
import 'ai_test_row.dart';

/// The rows of Settings › AI: the switch, the provider with its address,
/// model and region, the key, the Test button and the line about where
/// questions go.
///
/// It owns the three fields' text, and saves it a moment after the person
/// pauses. [state] is what the `AiSettingsBloc` above it has.
class AiSettingsForm extends StatefulWidget {
  const AiSettingsForm({required this.state, super.key});

  final AiSettingsState state;

  /// How far a row's hairline is inset: past its label.
  static const double inset = KansoSpacing.md;

  @override
  State<AiSettingsForm> createState() => _AiSettingsFormState();
}

class _AiSettingsFormState extends State<AiSettingsForm> {
  late final TextEditingController address;
  late final TextEditingController model;
  late final TextEditingController region;

  @override
  void initState() {
    super.initState();
    final settings = widget.state.settings;
    address = TextEditingController(text: settings.baseUrl);
    model = TextEditingController(text: settings.model);
    region = TextEditingController(text: settings.region);
  }

  @override
  void dispose() {
    address.dispose();
    model.dispose();
    region.dispose();
    super.dispose();
  }

  AiFields get fields => AiFields(
    address: address.text,
    model: model.text,
    region: region.text,
  );

  void changed(String _) {
    context.read<AiSettingsBloc>().add(AiSettingsEvent.fieldsChanged(fields));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<AiSettingsBloc>();
    final state = widget.state;
    final settings = state.settings;
    final provider = state.provider;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ListView(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        bottom + KansoSpacing.lg,
      ),
      children: <Widget>[
        KansoGroupedSection(
          footer: l10n.aiEnabledFooter,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.sparkles,
              label: l10n.aiEnabled,
              trailing: KansoSwitch(
                value: settings.enabled,
                onChanged: (value) =>
                    bloc.add(AiSettingsEvent.enabledChanged(enabled: value)),
              ),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.lg),
        KansoGroupedSection(
          title: l10n.aiProviderSection,
          footer: provider?.description,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.cloud,
              label: l10n.aiProvider,
              value: provider?.label ?? settings.provider,
              chevron: true,
              onTap: state.providers.isEmpty
                  ? null
                  : () => showAiProviderSheet(context, bloc),
            ),
            AiFieldRow(
              label: l10n.aiAddress,
              hint: provider?.defaultBaseUrl ?? '',
              controller: address,
              onChanged: changed,
              keyboardType: TextInputType.url,
            ),
            AiFieldRow(
              label: l10n.aiModel,
              hint: provider?.defaultModel ?? '',
              controller: model,
              onChanged: changed,
            ),
            if (provider?.usesRegion ?? false)
              AiFieldRow(
                label: l10n.aiRegion,
                hint: provider!.defaultRegion,
                controller: region,
                onChanged: changed,
              ),
          ],
        ),
        if (provider?.acceptsKey ?? false) ...<Widget>[
          const SizedBox(height: KansoSpacing.lg),
          KansoGroupedSection(
            footer: l10n.aiKeyFooter,
            children: <Widget>[
              AiKeyRow(
                label: provider!.needsKey
                    ? l10n.aiApiKey
                    : l10n.aiApiKeyOptional,
                saved: settings.hasKey,
                busy: state.keyBusy,
                onSaved: (key) => bloc.add(AiSettingsEvent.keySaved(key)),
                onRemoved: () => bloc.add(const AiSettingsEvent.keyRemoved()),
              ),
            ],
          ),
        ],
        const SizedBox(height: KansoSpacing.lg),
        AiTestRow(
          status: state.test,
          message: state.testMessage,
          onTest: () => bloc.add(AiSettingsEvent.testRequested(fields)),
        ),
        if (state.error != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
            child: Text(
              l10n.aiSaveFailed,
              style: context.typography.meta.copyWith(
                color: context.colors.danger,
              ),
            ),
          ),
        ],
        if (settings.privacy.isNotEmpty) ...<Widget>[
          const SizedBox(height: KansoSpacing.lg),
          AiPrivacyNote(text: settings.privacy),
        ],
      ],
    );
  }
}
