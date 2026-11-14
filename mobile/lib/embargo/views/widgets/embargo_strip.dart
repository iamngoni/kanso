// Kanso — embargo_strip.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/embargo_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../injection.dart';
import '../../models/repos/abs/embargo_repository.dart';

/// The banner at the top of a note under embargo: "Embargoed until Wed 18
/// Nov, 09:00 New York", and how far off that is. Nothing is shown for any
/// other note, so the note's layout does not move. It reads the note again
/// when its revision changes, and goes when the embargo lifts.
class EmbargoStrip extends StatefulWidget {
  const EmbargoStrip({
    required this.noteId,
    this.revision,
    this.gutter = KansoSpacing.md,
    this.repository,
    super.key,
  });

  /// The open note.
  final String noteId;

  /// The note's revision: the strip reads the note again when it changes.
  final String? revision;

  /// The space at either side, as the page's.
  final double gutter;

  /// The embargo repository; the app's when null (tests pass one).
  final EmbargoRepository? repository;

  @override
  State<EmbargoStrip> createState() => _EmbargoStripState();
}

class _EmbargoStripState extends State<EmbargoStrip> {
  /// Null where the app has no library (a page built without it, as some
  /// tests do): the strip shows nothing.
  late final EmbargoRepository? _repository =
      widget.repository ??
      (getIt.isRegistered<EmbargoRepository>()
          ? getIt<EmbargoRepository>()
          : null);
  EmbargoView? _embargo;
  int _generation = 0;
  Timer? _lift;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  @override
  void didUpdateWidget(EmbargoStrip old) {
    super.didUpdateWidget(old);
    if (old.noteId != widget.noteId) {
      _embargo = null;
      unawaited(_load());
    } else if (old.revision != widget.revision) {
      unawaited(_load());
    }
  }

  @override
  void dispose() {
    _lift?.cancel();
    super.dispose();
  }

  Future<void> _load() async {
    final repository = _repository;
    if (repository == null) {
      return;
    }
    final generation = ++_generation;
    final read = await repository.embargo(widget.noteId);
    if (!mounted || generation != _generation) {
      return;
    }
    final embargo = read.toOption().toNullable();
    setState(() => _embargo = embargo);
    // Looks again when it lifts, so the banner goes then. Further off than
    // a day, the next revision or visit reads it again first.
    _lift?.cancel();
    if (embargo != null) {
      final wait = Duration(
        milliseconds:
            embargo.liftsAtMs - DateTime.now().millisecondsSinceEpoch + 500,
      );
      if (wait <= const Duration(days: 1)) {
        _lift = Timer(
          wait.isNegative ? Duration.zero : wait,
          () => unawaited(_load()),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final embargo = _embargo;
    if (embargo == null) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    final l10n = context.l10n;
    final title = l10n.embargoBanner(embargo.lifts);
    final lifts = l10n.embargoBannerLifts(embargo.liftsIn);
    return Semantics(
      container: true,
      label: '$title. $lifts',
      excludeSemantics: true,
      child: Container(
        key: const ValueKey<String>('embargo-strip'),
        margin: EdgeInsets.fromLTRB(
          widget.gutter,
          KansoSpacing.xs,
          widget.gutter,
          0,
        ),
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.sm,
          vertical: KansoSpacing.xs,
        ),
        decoration: BoxDecoration(
          color: colors.warning.withValues(alpha: 0.12),
          borderRadius: KansoRadii.mediumBorder,
        ),
        child: Row(
          children: <Widget>[
            Container(
              width: 28,
              height: 28,
              decoration: BoxDecoration(
                color: colors.warning.withValues(alpha: 0.18),
                shape: BoxShape.circle,
              ),
              alignment: Alignment.center,
              child: KansoIcon(
                KansoIcons.pause,
                size: 15,
                color: colors.warning,
              ),
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  Text(
                    title,
                    style: context.typography.label.copyWith(
                      color: colors.text,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  Text(
                    lifts,
                    style: context.typography.meta.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
