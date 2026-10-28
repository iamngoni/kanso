// Kanso — share_status_labels_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, ListAnys, StringAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';
import 'package:kanso/sharing/utils/share_labels.dart';

import '../../helpers/property.dart';
import '../../helpers/sharing_property.dart';

final AppLocalizationsEn _l10n = AppLocalizationsEn();

LiveStatus _status({
  ShareRole role = ShareRole.editor,
  bool connected = true,
  bool ended = false,
  bool nearLimit = false,
  List<String> problems = const <String>[],
}) => LiveStatus(
  role: role,
  connected: connected,
  ended: ended,
  nearLimit: nearLimit,
  problems: problems,
);

void main() {
  group('sharedStatusLine', () {
    test('says it in the desktop footer’s words', () {
      expect(_l10n.sharedStatusLine(_status()), 'Shared · Can edit · Live');
      expect(
        _l10n.sharedStatusLine(
          _status(role: ShareRole.viewer, connected: false, nearLimit: true),
        ),
        'Shared · Can view · Read only · Offline · '
        "Near this shared note's size limit",
      );
      expect(
        _l10n.sharedStatusLine(_status(role: ShareRole.commenter)),
        'Shared · Can comment · Read only · Live',
      );
      expect(
        _l10n.sharedStatusLine(_status(ended: true)),
        'Private copy · No longer shared',
      );
    });

    Glados<LiveStatus>(any.liveStatus, kansoExploreConfig()).test(
      'a still-shared note names its role, says read only exactly when the '
      'role can’t edit, and live or offline',
      (status) {
        final line = _l10n.sharedStatusLine(status);
        if (status.ended) {
          expect(line, _l10n.sharedStatusPrivateCopy);
          return;
        }
        final parts = line.split(' · ');
        expect(parts.first, 'Shared');
        expect(parts[1], _l10n.shareRoleName(status.role));
        final canEdit =
            status.role == ShareRole.owner || status.role == ShareRole.editor;
        expect(parts.contains('Read only'), !canEdit);
        expect(parts.contains('Live'), status.connected);
        expect(parts.contains('Offline'), !status.connected);
        expect(
          parts.contains(_l10n.sharedStatusNearLimit),
          status.nearLimit,
        );
      },
    );
  });

  group('sharedProblemsBanner', () {
    test('the newest one’s words, and how many more', () {
      expect(_l10n.sharedProblemsBanner(const <String>[]), isNull);
      expect(_l10n.sharedProblemsBanner(<String>[' Too big. ']), 'Too big.');
      expect(
        _l10n.sharedProblemsBanner(<String>['Too big.', 'Refused']),
        'Too big. (1 more)',
      );
      expect(
        _l10n.sharedProblemsBanner(<String>['Too big.', 'a', 'b']),
        'Too big. (2 more)',
      );
    });

    Glados<List<String>>(
      any.listWithLengthInRange(0, 6, any.letterOrDigits),
      kansoExploreConfig(),
    ).test('there is a banner exactly when there is a problem, and it starts '
        'with the newest', (problems) {
      final banner = _l10n.sharedProblemsBanner(problems);
      expect(banner == null, problems.isEmpty);
      if (banner != null) {
        expect(banner, startsWith(problems.first.trim()));
        expect(banner.endsWith('more)'), problems.length > 1);
      }
    });
  });
}
