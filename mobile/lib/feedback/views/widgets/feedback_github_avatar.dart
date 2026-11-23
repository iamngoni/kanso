// Kanso — feedback_github_avatar.dart
// Created 2026-11-23.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';

/// A GitHub user's avatar as a circle. While the picture loads, and if it
/// can't (offline, or GitHub's avatar host is unreachable), a circle with the
/// first letter of the login stands in, so the row is never broken.
class FeedbackGithubAvatar extends StatelessWidget {
  const FeedbackGithubAvatar({
    required this.login,
    required this.url,
    this.size = 28,
    super.key,
  });

  final String login;
  final String url;
  final double size;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final initial = login.isEmpty ? '?' : login.substring(0, 1).toUpperCase();
    final fallback = ColoredBox(
      color: colors.fill,
      child: Center(
        child: Text(
          initial,
          style: context.typography.secondary.copyWith(
            color: colors.textSecondary,
          ),
        ),
      ),
    );
    return ExcludeSemantics(
      child: SizedBox.square(
        dimension: size,
        child: ClipOval(
          child: Image.network(
            url,
            width: size,
            height: size,
            fit: BoxFit.cover,
            errorBuilder: (context, error, stackTrace) => fallback,
            loadingBuilder: (context, child, progress) =>
                progress == null ? child : fallback,
          ),
        ),
      ),
    );
  }
}
