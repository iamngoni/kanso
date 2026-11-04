// Kanso — about_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_stores.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/about/about_bloc.dart';
import 'settings_category_pane.dart';

/// Settings › About: what the app is (its version, its website, its release
/// notes, help, a review where the store takes one, its licences). It needs
/// an [AboutBloc] above it.
class AboutSettingsPane extends StatelessWidget {
  const AboutSettingsPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SettingsCategoryPane(
      title: l10n.settingsAbout,
      sections: <Widget>[
        BlocBuilder<AboutBloc, AboutState>(
          builder: (context, about) {
            final version = about.version;
            final bloc = context.read<AboutBloc>();
            return KansoGroupedSection(
              dividerIndent: SettingsCategoryPane.iconInset,
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.info,
                  label: l10n.settingsVersion,
                  value: version == null || about.status != LoadStatus.ready
                      ? null
                      : version.patch == null
                      ? l10n.settingsVersionValue(
                          version.version,
                          version.build,
                        )
                      : l10n.settingsVersionPatchValue(
                          version.version,
                          version.build,
                          version.patch!,
                        ),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.globe,
                  label: l10n.settingsWebsite,
                  external: true,
                  onTap: () => bloc.add(const AboutEvent.websiteOpened()),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.fileText,
                  label: l10n.settingsReleaseNotes,
                  external: true,
                  onTap: () => bloc.add(const AboutEvent.releaseNotesOpened()),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.circleHelp,
                  label: l10n.settingsHelp,
                  external: true,
                  onTap: () => bloc.add(const AboutEvent.helpOpened()),
                ),
                if (KansoStores.writeReviewFor(defaultTargetPlatform) != null)
                  KansoSettingsRow(
                    icon: KansoIcons.star,
                    label: l10n.settingsRate,
                    external: true,
                    onTap: () => bloc.add(const AboutEvent.rateOpened()),
                  ),
                KansoSettingsRow(
                  icon: KansoIcons.scale,
                  label: l10n.settingsLicences,
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.settingsLicences),
                ),
              ],
            );
          },
        ),
      ],
    );
  }
}
