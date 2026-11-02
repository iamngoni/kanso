// Kanso — resurface_pill.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_motion.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../state/resurface/resurface_bloc.dart';
import 'related_notes_sheet.dart';

/// A small pill, "2 related", offered when past notes are about the
/// paragraph being written. Tapping it opens the related notes sheet.
///
/// There is no pill when nothing matches. It fades in over 150 ms, once the
/// text has paused, and never pulses or bounces. It is above the keyboard's
/// bar while writing, and at the bottom right of the note when reading.
class ResurfacePill extends StatelessWidget {
  const ResurfacePill({required this.noteId, this.onLink, super.key});

  final String noteId;

  /// Puts a link to the note titled so at the caret; none while reading.
  final ValueChanged<String>? onLink;

  @override
  Widget build(BuildContext context) {
    final bloc = _maybeRead(context);
    if (bloc == null) {
      return const SizedBox.shrink();
    }
    return BlocBuilder<ResurfaceBloc, ResurfaceState>(
      bloc: bloc,
      buildWhen: (before, now) =>
          before.rows.length != now.rows.length ||
          before.day.length != now.day.length ||
          before.enabled != now.enabled,
      builder: (context, state) {
        final count = resurfaceCount(state);
        if (count == 0) {
          return const SizedBox.shrink();
        }
        final colors = context.colors;
        final label = context.l10n.resurfacePill(count);
        return TweenAnimationBuilder<double>(
          key: const ValueKey<String>('resurface pill'),
          tween: Tween<double>(begin: 0, end: 1),
          duration: KansoMotion.fast,
          builder: (context, opacity, child) =>
              Opacity(opacity: opacity, child: child),
          child: Semantics(
            button: true,
            label: label,
            child: Material(
              color: colors.fill,
              shape: const StadiumBorder(),
              child: InkWell(
                customBorder: const StadiumBorder(),
                onTap: () => RelatedNotesSheet.show(
                  context,
                  noteId: noteId,
                  resurface: bloc,
                  onLink: onLink,
                ),
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.sm,
                    vertical: KansoSpacing.xs,
                  ),
                  child: Text(
                    label,
                    style: context.typography.meta.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  static ResurfaceBloc? _maybeRead(BuildContext context) {
    try {
      return context.read<ResurfaceBloc>();
    } on ProviderNotFoundException {
      return null;
    }
  }
}

/// How many notes the pill offers: the past notes about the paragraph (when
/// they are on) and the notes of this day (on their own setting).
int resurfaceCount(ResurfaceState state) =>
    (state.enabled ? state.rows.length : 0) + state.day.length;

/// The pill, right-aligned in a row of its own above the keyboard's bar.
/// The row takes no room while there is no pill.
class ResurfacePillRow extends StatelessWidget {
  const ResurfacePillRow({required this.noteId, this.onLink, super.key});

  final String noteId;
  final ValueChanged<String>? onLink;

  @override
  Widget build(BuildContext context) {
    final bloc = ResurfacePill._maybeRead(context);
    if (bloc == null) {
      return const SizedBox.shrink();
    }
    return BlocBuilder<ResurfaceBloc, ResurfaceState>(
      bloc: bloc,
      buildWhen: (before, now) =>
          (resurfaceCount(before) == 0) != (resurfaceCount(now) == 0),
      builder: (context, state) {
        if (resurfaceCount(state) == 0) {
          return const SizedBox.shrink();
        }
        return Align(
          alignment: AlignmentDirectional.centerEnd,
          child: Padding(
            padding: const EdgeInsetsDirectional.only(
              end: KansoSpacing.md,
              bottom: KansoSpacing.xs,
            ),
            child: ResurfacePill(noteId: noteId, onLink: onLink),
          ),
        );
      },
    );
  }
}

/// A note being read: the pill sits at the bottom right of it, and past
/// notes are asked for about the note's first paragraph (there is no caret).
class ResurfaceReading extends StatefulWidget {
  const ResurfaceReading({
    required this.noteId,
    required this.title,
    required this.body,
    required this.child,
    super.key,
  });

  final String noteId;
  final String title;
  final String body;
  final Widget child;

  @override
  State<ResurfaceReading> createState() => _ResurfaceReadingState();
}

class _ResurfaceReadingState extends State<ResurfaceReading> {
  ResurfaceBloc? _bloc;

  @override
  void initState() {
    super.initState();
    _bloc = ResurfacePill._maybeRead(context);
    _ask();
  }

  @override
  void didUpdateWidget(ResurfaceReading oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.body != widget.body || oldWidget.title != widget.title) {
      _ask();
    }
  }

  void _ask() {
    _bloc?.add(
      ResurfaceEvent.edited(
        title: widget.title,
        body: widget.body,
        caret: 0,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        Positioned.fill(child: widget.child),
        PositionedDirectional(
          end: KansoSpacing.md,
          bottom: KansoSpacing.md,
          child: ResurfacePill(noteId: widget.noteId),
        ),
      ],
    );
  }
}
