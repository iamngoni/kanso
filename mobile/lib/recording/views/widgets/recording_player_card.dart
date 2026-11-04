// Kanso — recording_player_card.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../state/player/player_bloc.dart';
import '../../utils/player_clock.dart';

/// The card over a note's recording (issue #38, step 5):
///
/// ```text
/// ( ▶ )  ──────●─────────────
///        12:34         52:00
///  ↺5  5↻              1×   🔊 ━━━
/// ```
///
/// It draws what [PlayerBloc] says and sends it the taps; the sound is the
/// Rust player's. When the note holds no recording it takes no room. A tap
/// on a timestamp in the note scrolls the card into view.
class RecordingPlayerCard extends StatelessWidget {
  const RecordingPlayerCard({this.bloc, this.onSyncNow, super.key});

  /// The player's BLoC; the one in scope when null (tests pass one).
  final PlayerBloc? bloc;

  /// Fetches the audio that hasn't arrived yet.
  final VoidCallback? onSyncNow;

  @override
  Widget build(BuildContext context) {
    final source = bloc ?? context.read<PlayerBloc>();
    return BlocConsumer<PlayerBloc, PlayerState>(
      bloc: source,
      listenWhen: (a, b) => a.reveals != b.reveals,
      listener: (context, state) {
        // A timestamp was tapped: the card is the first thing on the page.
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (context.mounted) {
            Scrollable.ensureVisible(
              context,
              duration: const Duration(milliseconds: 250),
              curve: Curves.easeOutCubic,
              alignment: 0.1,
            );
          }
        });
      },
      buildWhen: (a, b) =>
          a.phase != b.phase ||
          a.playing != b.playing ||
          a.lengthMs != b.lengthMs ||
          a.speed != b.speed ||
          a.volume != b.volume ||
          a.message != b.message ||
          a.positionMs ~/ 1000 != b.positionMs ~/ 1000 ||
          a.fileIds != b.fileIds,
      builder: (context, state) {
        if (!state.shown) {
          return const SizedBox.shrink();
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
          child: _Frame(
            child: switch (state.phase) {
              PlayerPhase.notHere => _Unavailable(
                icon: KansoIcons.cloudOff,
                words: context.l10n.playerNotHere,
                action: KansoButton(
                  label: context.l10n.playerSyncNow,
                  variant: KansoButtonVariant.secondary,
                  compact: true,
                  expanded: false,
                  onPressed: onSyncNow,
                ),
              ),
              PlayerPhase.missing => _Unavailable(
                icon: KansoIcons.triangleAlert,
                words: context.l10n.playerMissing,
              ),
              PlayerPhase.failed => _Unavailable(
                icon: KansoIcons.triangleAlert,
                words: context.l10n.playerFailed,
                action: KansoButton(
                  label: context.l10n.playerTryAgain,
                  variant: KansoButtonVariant.secondary,
                  compact: true,
                  expanded: false,
                  onPressed: () => source.add(const PlayerEvent.retried()),
                ),
              ),
              _ => _Controls(bloc: source, state: state),
            },
          ),
        );
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
      label: context.l10n.playerCardLabel,
      child: Container(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: KansoRadii.largeBorder,
          border: Border.all(color: colors.divider),
        ),
        child: child,
      ),
    );
  }
}

class _Unavailable extends StatelessWidget {
  const _Unavailable({required this.icon, required this.words, this.action});

  final KansoIcons icon;
  final String words;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Row(
      children: <Widget>[
        KansoIcon(icon, color: colors.textSecondary),
        const SizedBox(width: KansoSpacing.xs),
        Expanded(child: Text(words, style: context.typography.secondary)),
        if (action != null) ...<Widget>[
          const SizedBox(width: KansoSpacing.xs),
          action!,
        ],
      ],
    );
  }
}

class _Controls extends StatelessWidget {
  const _Controls({required this.bloc, required this.state});

  final PlayerBloc bloc;
  final PlayerState state;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final opening = state.phase == PlayerPhase.opening;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        Row(
          children: <Widget>[
            _PlayButton(
              playing: state.playing,
              busy: opening,
              onPressed: () {
                unawaited(HapticFeedback.selectionClick());
                bloc.add(const PlayerEvent.toggled());
              },
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: _Scrubber(
                positionMs: state.positionMs,
                lengthMs: state.lengthMs,
                onSeek: (ms) => bloc.add(PlayerEvent.seeked(ms)),
              ),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.xxs),
        Row(
          children: <Widget>[
            KansoIconButton(
              icon: KansoIcons.rotateCcw,
              semanticLabel: l10n.playerBack,
              iconSize: 18,
              onPressed: () => bloc.add(const PlayerEvent.skipped(-5000)),
            ),
            KansoIconButton(
              icon: KansoIcons.rotateCw,
              semanticLabel: l10n.playerForward,
              iconSize: 18,
              onPressed: () => bloc.add(const PlayerEvent.skipped(5000)),
            ),
            const Spacer(),
            _SpeedPill(
              speed: state.speed,
              onPressed: () => bloc.add(const PlayerEvent.speedCycled()),
            ),
            const SizedBox(width: KansoSpacing.xxs),
            KansoIconButton(
              icon: state.volume <= 0 ? KansoIcons.volumeX : KansoIcons.volume2,
              semanticLabel: state.volume <= 0
                  ? l10n.playerUnmute
                  : l10n.playerMute,
              iconSize: 18,
              color: state.volume <= 0 ? colors.textMuted : null,
              onPressed: () => bloc.add(const PlayerEvent.muteToggled()),
            ),
            SizedBox(
              width: 72,
              child: _Track(
                value: state.volume,
                semanticLabel: l10n.playerVolume,
                onChanged: (v) => bloc.add(PlayerEvent.volumeChanged(v)),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _PlayButton extends StatelessWidget {
  const _PlayButton({
    required this.playing,
    required this.busy,
    required this.onPressed,
  });

  final bool playing;
  final bool busy;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Semantics(
      button: true,
      label: playing ? l10n.playerPause : l10n.playerPlay,
      excludeSemantics: true,
      onTap: onPressed,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onPressed,
        child: SizedBox.square(
          dimension: 52,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colors.button,
              shape: BoxShape.circle,
            ),
            child: Center(
              child: busy
                  ? SizedBox.square(
                      dimension: 20,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: colors.buttonText,
                      ),
                    )
                  : KansoIcon(
                      playing ? KansoIcons.pause : KansoIcons.play,
                      size: 22,
                      color: colors.buttonText,
                    ),
            ),
          ),
        ),
      ),
    );
  }
}

class _SpeedPill extends StatelessWidget {
  const _SpeedPill({required this.speed, required this.onPressed});

  final double speed;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final label = playerSpeedLabel(speed);
    return Semantics(
      button: true,
      label: '${context.l10n.playerSpeed}, $label',
      excludeSemantics: true,
      onTap: onPressed,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onPressed,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: 44, minWidth: 56),
          child: Center(
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: colors.fill,
                borderRadius: KansoRadii.pillBorder,
              ),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.sm,
                  vertical: KansoSpacing.xxs + 2,
                ),
                child: Text(
                  label,
                  style: context.typography.monoSmall.copyWith(
                    color: colors.text,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// The scrubber and the time at both ends. Dragging moves the thumb alone;
/// the recording seeks when the finger lifts.
class _Scrubber extends StatefulWidget {
  const _Scrubber({
    required this.positionMs,
    required this.lengthMs,
    required this.onSeek,
  });

  final int positionMs;
  final int lengthMs;
  final ValueChanged<int> onSeek;

  @override
  State<_Scrubber> createState() => _ScrubberState();
}

class _ScrubberState extends State<_Scrubber> {
  double? _dragging;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final progress =
        _dragging ?? playerProgress(widget.positionMs, widget.lengthMs);
    final shownMs = _dragging == null
        ? widget.positionMs
        : playerMomentAt(_dragging!, widget.lengthMs);
    final elapsed = playerClock(shownMs);
    final total = widget.lengthMs > 0 ? playerClock(widget.lengthMs) : '–:––';
    final times = context.typography.monoSmall.copyWith(
      color: colors.textSecondary,
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        _Track(
          value: progress,
          semanticLabel: l10n.playerPosition,
          semanticValue: l10n.playerPositionValue(elapsed, total),
          onChangeStart: (v) => setState(() => _dragging = v),
          onChanged: (v) => setState(() => _dragging = v),
          onChangeEnd: (v) {
            setState(() => _dragging = null);
            widget.onSeek(playerMomentAt(v, widget.lengthMs));
          },
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: <Widget>[
              Text(elapsed, style: times),
              Text(total, style: times),
            ],
          ),
        ),
      ],
    );
  }
}

/// A thin rail with a round thumb: the scrubber and the volume.
class _Track extends StatelessWidget {
  const _Track({
    required this.value,
    required this.semanticLabel,
    this.semanticValue,
    this.onChangeStart,
    this.onChanged,
    this.onChangeEnd,
  });

  final double value;
  final String semanticLabel;
  final String? semanticValue;
  final ValueChanged<double>? onChangeStart;
  final ValueChanged<double>? onChanged;
  final ValueChanged<double>? onChangeEnd;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return SliderTheme(
      data: SliderTheme.of(context).copyWith(
        trackHeight: 4,
        activeTrackColor: colors.accent,
        inactiveTrackColor: colors.divider,
        thumbColor: colors.accent,
        overlayColor: colors.accent.withValues(alpha: 0.12),
        thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7),
        overlayShape: const RoundSliderOverlayShape(overlayRadius: 18),
        trackShape: const RoundedRectSliderTrackShape(),
      ),
      child: Semantics(
        label: semanticLabel,
        value: semanticValue,
        child: Slider(
          value: value.clamp(0.0, 1.0),
          onChangeStart: onChangeStart,
          onChanged: onChanged,
          onChangeEnd: onChangeEnd,
        ),
      ),
    );
  }
}
