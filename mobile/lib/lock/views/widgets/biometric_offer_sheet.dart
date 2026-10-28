// Kanso — biometric_offer_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../injection.dart';
import '../../models/data/biometrics_status.dart';
import '../../state/biometrics/biometrics_bloc.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../utils/biometric_kinds.dart';
import 'lock_form_sheet.dart';

/// The offer, made once after the first password unlock, to unlock the
/// locked notes with a biometric on this phone.
class BiometricOfferSheet extends StatelessWidget {
  const BiometricOfferSheet({super.key});

  /// Shows the offer over the whole app.
  static Future<void> show(
    BuildContext context, {
    required LockStatusBloc status,
  }) async {
    final bloc = getIt<BiometricsBloc>();
    try {
      await showKansoBottomSheet<void>(
        context,
        builder: (_) => MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<BiometricsBloc>.value(value: bloc),
            BlocProvider<LockStatusBloc>.value(value: status),
          ],
          child: const BiometricOfferSheet(),
        ),
      );
    } finally {
      await bloc.close();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final method = biometricName(
      l10n,
      context.select<LockStatusBloc, BiometricKind>(
        (bloc) => bloc.state.kind,
      ),
    );
    return BlocConsumer<BiometricsBloc, BiometricsState>(
      listener: (context, state) {
        if (state.status == BiometricsStatus.enabled) {
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed());
          Navigator.of(context).maybePop();
        }
      },
      builder: (context, state) {
        return LockFormSheet(
          title: l10n.lockOfferTitle(method),
          message: l10n.lockOfferMessage(method),
          notice: state.status == BiometricsStatus.failed
              ? l10n.settingsBiometricsFailed(method)
              : null,
          primaryLabel: l10n.lockOfferAccept(method),
          cancelLabel: l10n.lockOfferDecline,
          working: state.status == BiometricsStatus.working,
          onPrimary: () => context.read<BiometricsBloc>().add(
            BiometricsEvent.turnedOn(l10n.biometricReasonEnroll),
          ),
        );
      },
    );
  }
}
