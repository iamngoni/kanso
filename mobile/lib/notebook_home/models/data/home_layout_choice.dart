// Kanso — home_layout_choice.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the person chose in Customise home: the sections in their order and
/// which of them are hidden, or to go back to the default layout.
class HomeLayoutChoice {
  const HomeLayoutChoice({
    this.order = const <String>[],
    this.hidden = const <String>[],
    this.reset = false,
  });

  /// Every section's key, in the order chosen.
  final List<String> order;

  /// The keys of the sections to hide.
  final List<String> hidden;

  /// Go back to the default layout; [order] and [hidden] are then empty.
  final bool reset;
}
