// Kanso — locked_notes_section.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../injection.dart';
import '../../../lock/models/data/biometrics_status.dart';
import '../../../lock/state/biometrics/biometrics_bloc.dart';
import '../../../lock/state/files_migration/files_migration_bloc.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../lock/utils/auto_lock.dart';
import '../../../lock/utils/biometric_kinds.dart';
import '../../../lock/views/widgets/change_password_sheet.dart';
import '../../../lock/views/widgets/files_migration_sheet.dart';

/// Settings › Locked notes, once a lock password exists: unlock with Face ID
/// (Touch ID, fingerprint) when the phone has it, how long until the notes
/// lock again (fixed), Change lock password… and Lock notes now.
///
/// It reads the `LockStatusBloc` and the `BiometricsBloc` above it.
class LockedNotesSection extends StatelessWidget {
  const LockedNotesSection({required this.dividerIndent, super.key});

  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return _MigrationScope(
      builder: (context, migration) =>
          BlocBuilder<LockStatusBloc, LockStatusState>(
            builder: (context, lock) {
              if (!lock.hasLockPassword) {
                return const SizedBox.shrink();
              }
              final method = biometricName(l10n, lock.kind);
              final status = context.read<LockStatusBloc>();
              return BlocConsumer<BiometricsBloc, BiometricsState>(
                listenWhen: (before, after) =>
                    before.status != after.status &&
                    (after.status == BiometricsStatus.enabled ||
                        after.status == BiometricsStatus.disabled),
                listener: (context, state) =>
                    status.add(const LockStatusEvent.refreshed()),
                builder: (context, biometrics) {
                  final bloc = context.read<BiometricsBloc>();
                  final working = biometrics.status == BiometricsStatus.working;
                  void toggle({required bool on}) => bloc.add(
                    on
                        ? BiometricsEvent.turnedOn(l10n.biometricReasonEnroll)
                        : const BiometricsEvent.turnedOff(),
                  );
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      KansoGroupedSection(
                        title: l10n.settingsLockedNotes,
                        footer: switch (biometrics.status) {
                          BiometricsStatus.notesClosed =>
                            l10n.settingsBiometricsClosed(method),
                          BiometricsStatus.enabled => l10n.settingsBiometricsOn(
                            method,
                          ),
                          BiometricsStatus.disabled =>
                            l10n.settingsBiometricsOff,
                          BiometricsStatus.failed =>
                            l10n.settingsBiometricsFailed(method),
                          _ => null,
                        },
                        dividerIndent: dividerIndent,
                        children: <Widget>[
                          if (biometricsAvailable(lock.kind))
                            KansoSettingsRow(
                              icon:
                                  lock.kind == BiometricKind.faceId ||
                                      lock.kind == BiometricKind.face
                                  ? KansoIcons.scanFace
                                  : KansoIcons.fingerprint,
                              label: l10n.settingsUnlockWith(method),
                              trailing: KansoSwitch(
                                value: lock.biometricsOn,
                                onChanged: working
                                    ? null
                                    : (on) => toggle(on: on),
                              ),
                              onTap: working
                                  ? null
                                  : () => toggle(on: !lock.biometricsOn),
                            ),
                          KansoSettingsRow(
                            icon: KansoIcons.clock,
                            label: l10n.settingsLockAfter,
                            value: l10n.settingsTenMinutes(autoLockMinutes),
                          ),
                          if (migration != null &&
                              (migration.state.info?.notes ?? 0) > 0)
                            KansoSettingsRow(
                              icon: KansoIcons.lock,
                              label: l10n.settingsSealAttachments,
                              value: l10n.settingsSealAttachmentsValue(
                                migration.state.info?.notes ?? 0,
                              ),
                              chevron: true,
                              onTap: () {
                                migration.add(
                                  const FilesMigrationEvent.opened(),
                                );
                                unawaited(
                                  FilesMigrationSheet.show(context, migration),
                                );
                              },
                            ),
                          KansoSettingsRow(
                            icon: KansoIcons.lock,
                            label: l10n.settingsChangeLockPassword,
                            chevron: true,
                            onTap: () => ChangePasswordSheet.show(
                              context,
                              status: status,
                            ),
                          ),
                          KansoSettingsRow(
                            icon: KansoIcons.lockOpen,
                            label: l10n.settingsLockNow,
                            onTap: lock.unlocked
                                ? () => status.add(
                                    const LockStatusEvent.lockedNow(),
                                  )
                                : null,
                          ),
                        ],
                      ),
                      const SizedBox(height: KansoSpacing.lg),
                    ],
                  );
                },
              );
            },
          ),
    );
  }
}

/// Makes the bloc that knows which locked notes still have attachments that
/// are not sealed, for the row that seals them, and hands it to [builder]
/// (null where none is registered, such as a test that does not care). The
/// row follows the bloc, so it goes once everything is sealed.
class _MigrationScope extends StatefulWidget {
  const _MigrationScope({required this.builder});

  final Widget Function(BuildContext context, FilesMigrationBloc? migration)
  builder;

  @override
  State<_MigrationScope> createState() => _MigrationScopeState();
}

class _MigrationScopeState extends State<_MigrationScope> {
  FilesMigrationBloc? bloc;

  @override
  void initState() {
    super.initState();
    if (getIt.isRegistered<FilesMigrationBloc>()) {
      bloc = getIt<FilesMigrationBloc>()
        ..add(const FilesMigrationEvent.checked());
    }
  }

  @override
  void dispose() {
    unawaited(bloc?.close());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final migration = bloc;
    if (migration == null) {
      return widget.builder(context, null);
    }
    return BlocBuilder<FilesMigrationBloc, FilesMigrationState>(
      bloc: migration,
      builder: (context, _) => widget.builder(context, migration),
    );
  }
}
