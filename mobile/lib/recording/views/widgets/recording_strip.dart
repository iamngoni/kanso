// Kanso — recording_strip.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/recording_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../injection.dart';
import '../../state/recording/recording_bloc.dart';
import '../../utils/recording_clock.dart';

/// The strip over every screen while a recording runs, or after a crash
/// left one:
///
/// ```text
/// ● 00:42:17  ▁▃▅▂▆▁          Pause  Stop  Discard
/// ```
///
/// It takes no room when there is nothing to say, so the layout beneath
/// does not move.
class RecordingStrip extends StatelessWidget {
  const RecordingStrip({this.bloc, super.key});

  /// The recording bloc; the app's when null (tests pass one).
  final RecordingBloc? bloc;

  @override
  Widget build(BuildContext context) {
    final source = bloc ?? getIt<RecordingBloc>();
    return BlocBuilder<RecordingBloc, RecordingState>(
      bloc: source,
      buildWhen: (a, b) =>
          a.phase != b.phase ||
          a.seconds != b.seconds ||
          a.level != b.level ||
          a.paused != b.paused ||
          a.found != b.found,
      builder: (context, state) {
        if (state.active) {
          return _Bar(bloc: source, state: state);
        }
        if (state.found.isNotEmpty) {
          return _Recover(bloc: source, found: state.found.first);
        }
        return const SizedBox.shrink();
      },
    );
  }
}

class _Frame extends StatelessWidget {
  const _Frame({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      container: true,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: colors.elevated,
          border: Border(bottom: BorderSide(color: colors.divider)),
        ),
        child: Padding(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            MediaQuery.paddingOf(context).top + KansoSpacing.xs,
            KansoSpacing.xs,
            KansoSpacing.xs,
          ),
          child: child,
        ),
      ),
    );
  }
}

class _Bar extends StatelessWidget {
  const _Bar({required this.bloc, required this.state});

  final RecordingBloc bloc;
  final RecordingState state;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final saving = state.phase == RecordingPhase.saving;
    return _Frame(
      child: Row(
        children: <Widget>[
          Icon(
            Icons.circle,
            size: 10,
            color: state.paused ? colors.textMuted : colors.danger,
          ),
          const SizedBox(width: KansoSpacing.xs),
          Semantics(
            label: l10n.recordingClockLabel,
            child: Text(
              recordingClock(state.seconds),
              style: context.typography.mono,
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          _Meter(level: state.level, color: colors.danger),
          const Spacer(),
          if (saving)
            Text(l10n.recordingSaving, style: context.typography.secondary)
          else ...<Widget>[
            KansoIconButton(
              icon: state.paused
                  ? KansoIcons.circleDot
                  : KansoIcons.circlePause,
              semanticLabel: state.paused
                  ? l10n.recordingResume
                  : l10n.recordingPause,
              onPressed: () => bloc.add(
                state.paused
                    ? const RecordingEvent.resumed()
                    : const RecordingEvent.paused(),
              ),
            ),
            KansoIconButton(
              icon: KansoIcons.square,
              semanticLabel: l10n.recordingStop,
              color: colors.danger,
              onPressed: () => bloc.add(const RecordingEvent.stopped()),
            ),
            KansoIconButton(
              icon: KansoIcons.trash,
              semanticLabel: l10n.recordingDiscard,
              onPressed: () => _confirmDiscard(context, bloc),
            ),
          ],
        ],
      ),
    );
  }
}

/// Asks once, then deletes the recording.
Future<void> _confirmDiscard(BuildContext context, RecordingBloc bloc) async {
  final l10n = context.l10n;
  final sure = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(l10n.recordingDiscardTitle),
      content: Text(l10n.recordingDiscardBody),
      actions: <Widget>[
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: Text(l10n.recordingCancel),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(true),
          child: Text(l10n.recordingDiscard),
        ),
      ],
    ),
  );
  if (sure ?? false) {
    bloc.add(const RecordingEvent.discarded());
  }
}

class _Meter extends StatelessWidget {
  const _Meter({required this.level, required this.color});

  final double level;
  final Color color;

  @override
  Widget build(BuildContext context) {
    const weights = <double>[0.4, 0.7, 1, 0.8, 0.55, 0.9, 0.6];
    final shaped = level.clamp(0.0, 1.0);
    return SizedBox(
      height: 16,
      child: Row(
        children: <Widget>[
          for (final weight in weights)
            Padding(
              padding: const EdgeInsets.only(right: 2),
              child: Container(
                width: 3,
                height: 3 + 13 * shaped * weight,
                decoration: BoxDecoration(
                  color: color,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _Recover extends StatelessWidget {
  const _Recover({required this.bloc, required this.found});

  final RecordingBloc bloc;
  final RecordingFound found;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return _Frame(
      child: Row(
        children: <Widget>[
          Expanded(
            child: Text(found.text, style: context.typography.secondary),
          ),
          const SizedBox(width: KansoSpacing.xs),
          KansoButton(
            label: l10n.recordingRecover,
            expanded: false,
            compact: true,
            onPressed: () => bloc.add(RecordingEvent.recovered(found.id)),
          ),
          const SizedBox(width: KansoSpacing.xxs),
          KansoButton(
            label: l10n.recordingDiscard,
            variant: KansoButtonVariant.quiet,
            expanded: false,
            compact: true,
            onPressed: () => bloc.add(RecordingEvent.foundDiscarded(found.id)),
          ),
        ],
      ),
    );
  }
}

/// Puts the [RecordingStrip] over the whole app, and looks for a recording
/// a crash left when the app starts. The pages beneath lose their top
/// padding while the strip has taken it.
class RecordingHost extends StatefulWidget {
  const RecordingHost({required this.child, this.bloc, super.key});

  final Widget child;
  final RecordingBloc? bloc;

  @override
  State<RecordingHost> createState() => _RecordingHostState();
}

class _RecordingHostState extends State<RecordingHost> {
  late final RecordingBloc? _bloc =
      widget.bloc ??
      (getIt.isRegistered<RecordingBloc>() ? getIt<RecordingBloc>() : null);

  @override
  void initState() {
    super.initState();
    _bloc?.add(const RecordingEvent.foundChecked());
  }

  @override
  Widget build(BuildContext context) {
    final bloc = _bloc;
    if (bloc == null) {
      return widget.child;
    }
    return BlocBuilder<RecordingBloc, RecordingState>(
      bloc: bloc,
      buildWhen: (a, b) =>
          a.active != b.active || a.found.isEmpty != b.found.isEmpty,
      builder: (context, state) {
        final shown = state.active || state.found.isNotEmpty;
        return Column(
          children: <Widget>[
            RecordingStrip(bloc: bloc),
            Expanded(
              // Always wrapped, so the app below keeps its state when the
              // strip comes and goes.
              child: MediaQuery.removePadding(
                context: context,
                removeTop: shown,
                child: widget.child,
              ),
            ),
          ],
        );
      },
    );
  }
}
