// Kanso — kanso_icons.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The icons Kanso draws: Lucide, the same set the desktop app uses.
///
/// Each value names an SVG under `assets/icons/lucide/`. Draw one with
/// `KansoIcon`; a future icon theme can swap the source by changing
/// [assetPath].
enum KansoIcons {
  archive('archive'),
  arrowLeft('arrow-left'),
  arrowRight('arrow-right'),
  arrowUpDown('arrow-up-down'),
  bell('bell'),
  bold('bold'),
  book('book'),
  bookOpen('book-open'),
  calendar('calendar'),
  check('check'),
  chevronDown('chevron-down'),
  chevronLeft('chevron-left'),
  chevronRight('chevron-right'),
  circleCheck('circle-check'),
  circleDot('circle-dot'),
  circlePause('circle-pause'),
  circleHelp('circle-question-mark'),
  circleX('circle-x'),
  clock('clock'),
  cloud('cloud'),
  download('download'),
  ellipsis('ellipsis'),
  externalLink('external-link'),
  eye('eye'),
  fileText('file-text'),
  files('files'),
  fingerprint('fingerprint-pattern'),
  github('github'),
  globe('globe'),
  hash('hash'),
  image('image'),
  inbox('inbox'),
  indentDecrease('list-indent-decrease'),
  indentIncrease('list-indent-increase'),
  info('info'),
  layoutTemplate('layout-template'),
  link('link'),
  list('list'),
  listChecks('list-checks'),
  lock('lock'),
  lockOpen('lock-open'),
  logOut('log-out'),
  menu('menu'),
  monitor('monitor'),
  moon('moon'),
  network('network'),
  notebook('notebook'),
  palette('palette'),
  panelLeft('panel-left'),
  pencil('pencil'),
  pin('pin'),
  plus('plus'),
  refreshCw('refresh-cw'),
  scanFace('scan-face'),
  scale('scale'),
  search('search'),
  settings('settings'),
  shapes('shapes'),
  shield('shield'),
  sparkles('sparkles'),
  star('star'),
  sun('sun'),
  tag('tag'),
  textSize('a-large-small'),
  trash('trash'),
  triangleAlert('triangle-alert'),
  typeface('type'),
  undo('undo-2'),
  user('user'),
  close('x');

  const KansoIcons(this.lucideName);

  /// The icon's name in the Lucide set.
  final String lucideName;

  /// Where the SVG is bundled.
  String get assetPath => 'assets/icons/lucide/$lucideName.svg';
}
