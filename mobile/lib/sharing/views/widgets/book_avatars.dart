// Kanso — book_avatars.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../utils/book_labels.dart';

/// The faces of a shared notebook's people, overlapping: each later face
/// is cut out of the one before it by a ring of the surface it sits on.
/// Each face is the person's initial on their own colour, the colour their
/// cursor has in a note. Past [most], a last face says how many more.
class BookAvatars extends StatelessWidget {
  const BookAvatars({
    required this.names,
    this.size = 20,
    this.most = 3,
    this.ring,
    super.key,
  });

  final List<String> names;
  final double size;
  final int most;

  /// The surface behind the faces, so the ring disappears into it.
  final Color? ring;

  @override
  Widget build(BuildContext context) {
    final shown = names.take(most).toList();
    final more = names.length - shown.length;
    final faces = <Widget>[
      for (final name in shown) _Face(size: size, name: name),
      if (more > 0) _Face(size: size, more: more),
    ];
    final step = size * 0.68;
    const ringWidth = 1.5;
    final ringColor = ring ?? context.colors.sidebar;
    return Semantics(
      label: names.join(', '),
      child: ExcludeSemantics(
        child: SizedBox(
          width: faces.isEmpty ? 0 : size + (faces.length - 1) * step,
          height: size,
          child: Stack(
            clipBehavior: Clip.none,
            children: <Widget>[
              for (final (index, face) in faces.indexed) ...<Widget>[
                if (index > 0)
                  Positioned(
                    left: index * step - ringWidth,
                    top: -ringWidth,
                    child: Container(
                      width: size + ringWidth * 2,
                      height: size + ringWidth * 2,
                      decoration: BoxDecoration(
                        color: ringColor,
                        shape: BoxShape.circle,
                      ),
                    ),
                  ),
                Positioned(left: index * step, top: 0, child: face),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

class _Face extends StatelessWidget {
  const _Face({required this.size, this.name, this.more});

  final double size;
  final String? name;
  final int? more;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final dark = context.brightness == Brightness.dark;
    final who = name;
    final (light, deep) = who == null
        ? (colors.textMuted, colors.textMuted)
        : avatarColours[avatarTone(who)];
    final hue = dark ? deep : light;
    return Container(
      width: size,
      height: size,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: hue.withValues(alpha: dark ? 0.28 : 0.16),
      ),
      child: Text(
        who == null ? '+$more' : avatarInitial(who),
        maxLines: 1,
        style: context.typography.meta.copyWith(
          color: who == null ? colors.textSecondary : hue,
          fontSize: size * (who == null ? 0.36 : 0.46),
          fontWeight: FontWeight.w700,
          height: 1,
        ),
      ),
    );
  }
}
