// Kanso — discovery_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../state/discovery/discovery_settings_bloc.dart';
import 'discovery_setting_switch_row.dart';
import 'discovery_status_row.dart';
import 'settings_category_pane.dart';

/// Settings › Discovery: related notes and how close they must be to come
/// back, On this day, and searching the text in images.
class DiscoverySettingsPane extends StatelessWidget {
  const DiscoverySettingsPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SettingsCategoryPane(
      title: l10n.settingsDiscovery,
      sections: <Widget>[
        BlocBuilder<DiscoverySettingsBloc, DiscoverySettingsState>(
          builder: (context, discovery) {
            final bloc = context.read<DiscoverySettingsBloc>();
            return KansoGroupedSection(
              dividerIndent: SettingsCategoryPane.iconInset,
              children: <Widget>[
                DiscoverySettingSwitchRow(
                  title: l10n.discoveryRelated,
                  detail: l10n.discoveryRelatedDetail,
                  value: discovery.relatedEnabled,
                  onChanged: discovery.relatedBusy
                      ? null
                      : (enabled) => bloc.add(
                          DiscoverySettingsEvent.relatedChanged(
                            enabled: enabled,
                          ),
                        ),
                ),
                if (discovery.relatedEnabled) ...<Widget>[
                  DiscoverySettingSwitchRow(
                    title: l10n.discoveryResurface,
                    detail: l10n.discoveryResurfaceDetail,
                    value: discovery.resurfaceEnabled,
                    onChanged: (enabled) => bloc.add(
                      DiscoverySettingsEvent.resurfaceChanged(enabled: enabled),
                    ),
                  ),
                  if (discovery.resurfaceEnabled)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        0,
                        KansoSpacing.md,
                        KansoSpacing.sm,
                      ),
                      child: KansoSegmented<String>(
                        expanded: true,
                        semanticLabel: l10n.discoveryCloseness,
                        selected: discovery.closeness,
                        segments: <KansoSegment<String>>[
                          KansoSegment<String>(
                            value: 'closer',
                            label: l10n.discoveryClosenessCloser,
                          ),
                          KansoSegment<String>(
                            value: 'default',
                            label: l10n.discoveryClosenessDefault,
                          ),
                          KansoSegment<String>(
                            value: 'looser',
                            label: l10n.discoveryClosenessLooser,
                          ),
                        ],
                        onChanged: (closeness) => bloc.add(
                          DiscoverySettingsEvent.closenessChanged(closeness),
                        ),
                      ),
                    ),
                  DiscoveryStatusRow(
                    status: discovery.related,
                    onRetry: () => bloc.add(const RelatedRetryRequestedEvent()),
                  ),
                ],
                DiscoverySettingSwitchRow(
                  title: l10n.discoveryOnThisDay,
                  detail: l10n.discoveryOnThisDayDetail,
                  value: discovery.onThisDayEnabled,
                  onChanged: (enabled) => bloc.add(
                    DiscoverySettingsEvent.onThisDayChanged(enabled: enabled),
                  ),
                ),
                DiscoverySettingSwitchRow(
                  title: l10n.discoverySearchImages,
                  detail: l10n.discoverySearchImagesDetail,
                  value: discovery.ocrEnabled,
                  onChanged: (enabled) => bloc.add(
                    DiscoverySettingsEvent.ocrChanged(enabled: enabled),
                  ),
                ),
                if (discovery.ocrEnabled)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(
                      KansoSpacing.md,
                      0,
                      KansoSpacing.md,
                      KansoSpacing.sm,
                    ),
                    child: Text(
                      l10n.discoveryReadingImages(discovery.ocrWaiting),
                      style: context.typography.secondary,
                    ),
                  ),
              ],
            );
          },
        ),
      ],
    );
  }
}
