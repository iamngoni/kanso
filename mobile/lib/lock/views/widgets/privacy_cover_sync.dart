// Kanso — privacy_cover_sync.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/privacy_cover/privacy_cover_bloc.dart';

/// Keeps the phone's cover over the app (see `PrivacyCoverBloc`) exactly as
/// long as [secure] is true and this is on screen: it is turned off when
/// [secure] ends, and when this leaves the screen.
class PrivacyCoverSync extends StatefulWidget {
  const PrivacyCoverSync({
    required this.secure,
    required this.child,
    super.key,
  });

  /// Whether what is on screen is an open locked note.
  final bool secure;
  final Widget child;

  @override
  State<PrivacyCoverSync> createState() => _PrivacyCoverSyncState();
}

class _PrivacyCoverSyncState extends State<PrivacyCoverSync> {
  late PrivacyCoverBloc cover;
  bool started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    cover = context.read<PrivacyCoverBloc>();
    if (!started) {
      started = true;
      if (widget.secure) {
        cover.add(const PrivacyCoverEvent.changed(secure: true));
      }
    }
  }

  @override
  void didUpdateWidget(PrivacyCoverSync oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.secure != widget.secure) {
      cover.add(PrivacyCoverEvent.changed(secure: widget.secure));
    }
  }

  @override
  void dispose() {
    if (!cover.isClosed) {
      cover.add(const PrivacyCoverEvent.changed(secure: false));
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
