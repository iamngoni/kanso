// Kanso — agent_status_dot.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/models/data/agent_session.dart';
import '../../../core/utils/context_extensions.dart';

/// The dot that says how a session stands: accent and pulsing while the AI
/// works, amber when it needs the person, grey when idle, red when failed.
/// The pulse stops when animations are turned off.
class AgentStatusDot extends StatefulWidget {
  const AgentStatusDot({
    required this.dot,
    this.size = KansoSizes.dot + 2,
    super.key,
  });

  final AgentSessionDot dot;
  final double size;

  @override
  State<AgentStatusDot> createState() => _AgentStatusDotState();
}

class _AgentStatusDotState extends State<AgentStatusDot>
    with SingleTickerProviderStateMixin {
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1100),
  );

  bool get _pulsing => widget.dot == AgentSessionDot.working;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sync();
  }

  @override
  void didUpdateWidget(AgentStatusDot old) {
    super.didUpdateWidget(old);
    _sync();
  }

  void _sync() {
    final animate = _pulsing && !MediaQuery.disableAnimationsOf(context);
    if (animate && !_pulse.isAnimating) {
      _pulse.repeat(reverse: true);
    } else if (!animate && _pulse.isAnimating) {
      _pulse
        ..stop()
        ..value = 0;
    }
  }

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final color = switch (widget.dot) {
      AgentSessionDot.working => colors.accent,
      AgentSessionDot.needsYou => colors.warning,
      AgentSessionDot.idle => colors.textMuted,
      AgentSessionDot.failed => colors.danger,
    };
    final size = widget.size;
    return ExcludeSemantics(
      child: SizedBox.square(
        dimension: size * 2,
        child: AnimatedBuilder(
          animation: _pulse,
          builder: (context, _) => Stack(
            alignment: Alignment.center,
            children: <Widget>[
              if (_pulsing)
                Container(
                  width: size + size * _pulse.value,
                  height: size + size * _pulse.value,
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.28 * (1 - _pulse.value)),
                    shape: BoxShape.circle,
                  ),
                ),
              Container(
                width: size,
                height: size,
                decoration: BoxDecoration(color: color, shape: BoxShape.circle),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
