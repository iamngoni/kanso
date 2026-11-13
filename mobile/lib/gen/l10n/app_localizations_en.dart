// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'Kanso';

  @override
  String get libraryAllNotes => 'All Notes';

  @override
  String get libraryTemplates => 'Templates';

  @override
  String get libraryPinned => 'Pinned';

  @override
  String get libraryRecent => 'Recent';

  @override
  String get libraryNotebooks => 'Notebooks';

  @override
  String get libraryTags => 'Tags';

  @override
  String get libraryTrash => 'Trash';

  @override
  String get openSettings => 'Settings';

  @override
  String get openLibrary => 'Library';

  @override
  String get searchHint => 'Search notes';

  @override
  String get searchClear => 'Clear search';

  @override
  String get searchCancel => 'Cancel';

  @override
  String get searchFilterAll => 'All';

  @override
  String get searchFilterTitles => 'Titles';

  @override
  String get searchFilterTags => 'Tags';

  @override
  String get searchFilterNotebook => 'This notebook';

  @override
  String searchResultCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count results',
      one: '1 result',
    );
    return '$_temp0';
  }

  @override
  String get searchMeaningTitle => 'Found by meaning';

  @override
  String noteListCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$_temp0';
  }

  @override
  String get noteListMore => 'More';

  @override
  String get noteBack => 'Back';

  @override
  String get noteForward => 'Forward';

  @override
  String get noteSearch => 'Search';

  @override
  String get notePreview => 'Preview';

  @override
  String get noteEdit => 'Edit';

  @override
  String get noteLive => 'Live';

  @override
  String get noteInfo => 'Note info';

  @override
  String get noteMenu => 'More';

  @override
  String get noteOpenInNewWindow => 'Open in New Window';

  @override
  String get noteWindowLockedTitle => 'Locked note';

  @override
  String get noteWindowInTrash => 'This note is in the trash.';

  @override
  String get noteWindowNewNote => 'New Note';

  @override
  String get noteWindowSearch => 'Search All Notes';

  @override
  String get noteWindowClose => 'Close Window';

  @override
  String get noteWindowRestore => 'Restore';

  @override
  String get noteRelatedTurnOn =>
      'Turn on Related notes in Settings to see notes about the same things.';

  @override
  String get noteRelatedTitle => 'Related';

  @override
  String get noteRelatedEmpty => 'No related notes yet.';

  @override
  String get noteMeaning => 'What the meaning says';

  @override
  String noteMeaningTwin(String title, int percent) {
    return 'Nearly the same as $title — $percent%';
  }

  @override
  String get noteMeaningDismiss => 'Dismiss';

  @override
  String get noteMeaningMerge => 'Merge';

  @override
  String get noteMeaningMergeTitle => 'Merge notes?';

  @override
  String noteMeaningMergeWarning(String title) {
    return '$title will go to the trash. Its contents are kept in this note.';
  }

  @override
  String get cancel => 'Cancel';

  @override
  String get graphTitle => 'Graph';

  @override
  String get graphLocal => 'Local graph';

  @override
  String get graphFilterAll => 'All notes';

  @override
  String get graphFilterNotebook => 'Notebook';

  @override
  String get graphFilterTag => 'Tag';

  @override
  String get graphFilterTitle => 'Show the graph of';

  @override
  String get graphFilterNotebooks => 'Notebooks';

  @override
  String get graphFilterTags => 'Tags';

  @override
  String get graphEmpty => 'No linked notes to show.';

  @override
  String get graphFailed => 'Couldn\'t load the graph.';

  @override
  String get graphFit => 'Fit graph to view';

  @override
  String get graphZoomIn => 'Zoom in';

  @override
  String get graphZoomOut => 'Zoom out';

  @override
  String get noteMetaCreated => 'Created';

  @override
  String get noteMetaUpdated => 'Updated';

  @override
  String get statusActive => 'Active';

  @override
  String get statusOnHold => 'On hold';

  @override
  String get statusCompleted => 'Completed';

  @override
  String get statusDropped => 'Dropped';

  @override
  String editProperties(int count) {
    return 'Properties · $count';
  }

  @override
  String get accessoryHeading => 'Heading';

  @override
  String get accessoryList => 'List';

  @override
  String get accessoryTask => 'Task';

  @override
  String get accessoryBold => 'Bold';

  @override
  String get accessoryLink => 'Link';

  @override
  String get accessoryImage => 'Image';

  @override
  String get accessoryWikiLink => 'Link to a note';

  @override
  String get accessoryIndent => 'Indent';

  @override
  String get accessoryOutdent => 'Outdent';

  @override
  String get accessoryRedo => 'Redo';

  @override
  String get accessoryUndo => 'Undo';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsClose => 'Close';

  @override
  String get settingsJournal => 'Daily notes and reminders';

  @override
  String get settingsCalendar => 'Calendar';

  @override
  String get calendarPermissionTitle => 'Show your calendar on Overview?';

  @override
  String get calendarPermissionCopy =>
      'Kanso reads your events to list today\'s meetings and link them to your notes. It never changes your calendar, and events stay on this device.';

  @override
  String get calendarAllowAccess => 'Continue';

  @override
  String get calendarPrivacyCopy =>
      'Selected calendars are read-only. Events stay on this device and are never sent to Kanso\'s servers.';

  @override
  String get calendarConnect => 'Choose system calendars';

  @override
  String get calendarNoCalendars =>
      'No visible calendars are configured on this device.';

  @override
  String get calendarSources => 'System calendars';

  @override
  String get settingsPrivacySecurity => 'Privacy and security';

  @override
  String get settingsGeneral => 'General';

  @override
  String get settingsDefaultNotebook => 'Default notebook';

  @override
  String get defaultNotebookFirst => 'First notebook';

  @override
  String get defaultNotebookFooter =>
      'New notes go here, except those made from a notebook\'s own list. A locked notebook can\'t be chosen.';

  @override
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsTextSize => 'Text size';

  @override
  String get settingsInterfaceTextSize => 'Interface text size';

  @override
  String get settingsSync => 'Sync';

  @override
  String get settingsDiscovery => 'Discovery';

  @override
  String get discoveryRelated => 'Related notes';

  @override
  String get discoveryRelatedDetail =>
      'Lists notes about the same things as the open one, beside it. Turning this on downloads a small language model (about 30 MB) once. It runs on this device: nothing about your notes is sent anywhere.';

  @override
  String get discoveryTryAgain => 'Try Again';

  @override
  String get discoverySearchImages => 'Search text in images';

  @override
  String get discoverySearchImagesDetail =>
      'Reads the words in pictures you attach, on this device, so search finds the notes they\'re in. The words sync with the picture.';

  @override
  String discoveryReadingImages(int count) {
    return 'Reading images… $count to go.';
  }

  @override
  String get discoveryResurface => 'Related notes while writing';

  @override
  String get discoveryResurfaceDetail =>
      'After you pause, a small pill offers up to three past notes about the paragraph you are writing. It works on this device: nothing about your notes is sent anywhere.';

  @override
  String get discoveryCloseness => 'How close';

  @override
  String get discoveryClosenessCloser => 'Closer';

  @override
  String get discoveryClosenessDefault => 'Default';

  @override
  String get discoveryClosenessLooser => 'Looser';

  @override
  String get discoveryOnThisDay => 'Show “On this day” on daily notes';

  @override
  String get discoveryOnThisDayDetail =>
      'A daily note lists up to three notes written on its date in earlier years. It needs no language model.';

  @override
  String resurfacePill(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count related',
      one: '1 related',
    );
    return '$_temp0';
  }

  @override
  String get resurfaceAboutParagraph => 'About this paragraph';

  @override
  String get resurfaceAboutNote => 'About this note';

  @override
  String get resurfaceOnThisDay => 'On this day';

  @override
  String get resurfaceLink => 'Link';

  @override
  String get resurfaceNotRelated => 'Not related';

  @override
  String resurfaceNoteMeta(String notebook, String when) {
    return '$notebook · $when';
  }

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsVersion => 'Version';

  @override
  String get settingsWebsite => 'Website';

  @override
  String get settingsReleaseNotes => 'Release notes';

  @override
  String get settingsHelp => 'Help';

  @override
  String get settingsRate => 'Rate Kanso Pocket';

  @override
  String get settingsLicences => 'Licences';

  @override
  String get themesTitle => 'Themes';

  @override
  String get themesBadgeBuiltIn => 'Built in';

  @override
  String get themesCurrent => 'In use';

  @override
  String get lockedTitle => 'This note is locked';

  @override
  String get errorSignedOutTitle => 'Signed out on this device';

  @override
  String get errorSignedOutMessage =>
      'Your session on this device ended. Sign in again to keep syncing; your notes stay here.';

  @override
  String get errorSignInAgain => 'Sign in again';

  @override
  String get errorNotNow => 'Not now';

  @override
  String get designGalleryTitle => 'Design proposal';

  @override
  String get designGalleryIntro =>
      'Static screens for review. Nothing here saves or syncs.';

  @override
  String get designGalleryPhone => 'Phone';

  @override
  String get designGalleryTablet => 'Tablet';

  @override
  String get appearanceSystem => 'System';

  @override
  String get appearanceLight => 'Light';

  @override
  String get appearanceDark => 'Dark';

  @override
  String get screenError => 'Error card';

  @override
  String get themesSampleHeading => 'Heading';

  @override
  String get themesSampleLine => 'Some **bold** and *italic*';

  @override
  String get themesSampleCode => '`code`, [a link](url)';

  @override
  String get themesSampleQuote => 'a quote';

  @override
  String get themesBadgeYours => 'Yours';

  @override
  String get themesBadgeGallery => 'From the gallery';

  @override
  String get themesSameAsTheme => 'Same as theme';

  @override
  String get themesSectionTheme => 'Theme';

  @override
  String get themesSectionSyntax => 'Editor syntax';

  @override
  String get themesSectionPreview => 'Preview';

  @override
  String get themesSectionThemeDetail =>
      'Colors the sidebar, the note list, menus and every control.';

  @override
  String get themesSectionSyntaxDetail => 'Markdown and code in the editor.';

  @override
  String get themesSectionPreviewDetail => 'A note after it is rendered.';

  @override
  String get themesThumbSome => 'Some';

  @override
  String get themesThumbBoldComma => 'bold,';

  @override
  String get themesThumbLinkComma => 'a link,';

  @override
  String get themesThumbInlineCode => 'code';

  @override
  String get themesThumbBold => 'bold';

  @override
  String get themesThumbAnd => 'and';

  @override
  String get themesThumbItalic => 'italic';

  @override
  String get themesThumbLink => 'link';

  @override
  String get themesThumbDone => 'Done';

  @override
  String get settingsFonts => 'Fonts';

  @override
  String get settingsIcons => 'Icons';

  @override
  String get settingsIconThemes => 'Icon themes';

  @override
  String get iconsTitle => 'Icons';

  @override
  String get iconsFooter =>
      'The icon themes of the desktop app. Choosing one changes every icon at once; an icon a theme doesn’t draw stays Kanso’s own.';

  @override
  String iconsCredit(String author, String license) {
    return '$author, $license';
  }

  @override
  String iconsPreview(String name) {
    return 'Icons in $name';
  }

  @override
  String get iconsLoading => 'Loading icon themes';

  @override
  String get iconsFailedTitle => 'The icon themes didn’t load';

  @override
  String get iconsFailedMessage =>
      'Something went wrong reading the icon themes. Try again.';

  @override
  String get iconsEmptyTitle => 'No icon themes';

  @override
  String get iconsEmptyMessage => 'Kanso’s own icons are in use.';

  @override
  String get settingsBack => 'Back';

  @override
  String get fontsTitle => 'Fonts';

  @override
  String get googleFontSection => 'Google Fonts';

  @override
  String get googleFontFooter =>
      'Enter a family from Google Fonts. Once downloaded it is used for writing and reading, shows up in every font list above, and works offline.';

  @override
  String get googleFontHint => 'A family, like Lora';

  @override
  String get googleFontDownload => 'Download';

  @override
  String get googleFontLoading => 'Downloading…';

  @override
  String get googleFontOffline =>
      'You’re offline. Connect to download this font; saved fonts remain available offline.';

  @override
  String get googleFontFailed =>
      'This font could not be loaded. Check the family name and try again.';

  @override
  String get fontsInterfaceSection => 'Interface font';

  @override
  String get fontsInterfaceFooter =>
      'The font of menus, lists, labels and buttons.';

  @override
  String get fontsEditorSection => 'Editor font';

  @override
  String get fontsEditorFooter => 'The font you write in.';

  @override
  String get fontsPreviewSection => 'Preview font';

  @override
  String get fontsPreviewFooter => 'The font a note is read in.';

  @override
  String get fontsPreviewTextSizeSection => 'Preview text size';

  @override
  String get fontsPreviewTextSizeFooter =>
      'The preview’s text, in Split and Preview.';

  @override
  String get fontsCodeSection => 'Code font';

  @override
  String get fontsCodeFooter => 'Used for code in notes and in theme previews.';

  @override
  String get fontsSystemName => 'System — your device’s font';

  @override
  String get fontsSystemShort => 'System';

  @override
  String get fontsSystemSample => 'San Francisco on iOS, Roboto on Android';

  @override
  String get fontsSampleText => 'Calm notes, clearly written';

  @override
  String get fontsSampleCode => 'let note = merge(a, b);';

  @override
  String get fontsInterfaceTextSizeSection => 'Interface text size';

  @override
  String get fontsInterfaceTextSizeFooter =>
      'Changes labels, menus, buttons and the rest of Kanso. Notes keep their own text sizes.';

  @override
  String get fontsInterfaceTextSizeSample =>
      'Notebooks · Search · Tasks · Settings';

  @override
  String get interfaceTextSizeControlLabel => 'Interface text size';

  @override
  String get interfaceTextSizeDecrease => 'Decrease interface text size';

  @override
  String get interfaceTextSizeIncrease => 'Increase interface text size';

  @override
  String get fontsTextSizeSection => 'Text size';

  @override
  String get fontsTextSizeFooter =>
      'Changes the text of notes only. The rest of Kanso follows your device’s text size.';

  @override
  String get fontsTextSizeSample =>
      'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.';

  @override
  String textSizePercent(int percent) {
    return '$percent%';
  }

  @override
  String get textSizeControlLabel => 'Text size';

  @override
  String get textSizeDecrease => 'Decrease text size';

  @override
  String get textSizeIncrease => 'Increase text size';

  @override
  String get editorTitleHint => 'Title';

  @override
  String get editorChangedMessage =>
      'This note was changed elsewhere while you were editing. The version you don\'t keep stays in its history.';

  @override
  String get editorKeepMine => 'Keep mine';

  @override
  String get editorUseTheirs => 'Use theirs';

  @override
  String get editorDeletedMessage =>
      'This note was deleted elsewhere while you were editing.';

  @override
  String get editorKeepIt => 'Keep it';

  @override
  String get editorDiscard => 'Discard';

  @override
  String get editorSaveFailed => 'Your changes aren\'t saved yet.';

  @override
  String get editorOpeningLabel => 'Opening the note';

  @override
  String get propertiesAdd => 'Add property';

  @override
  String get propertiesName => 'Name';

  @override
  String get propertiesValue => 'Value';

  @override
  String propertiesRemove(String name) {
    return 'Remove $name';
  }

  @override
  String get propertiesConfirmAdd => 'Add';

  @override
  String get propertiesCancelAdd => 'Cancel';

  @override
  String get propertiesFailed => 'That property couldn\'t be set.';

  @override
  String get propertiesShow => 'Show properties';

  @override
  String get propertiesHide => 'Hide properties';

  @override
  String get noteModeLabel => 'View mode';

  @override
  String get noteListLoading => 'Loading notes';

  @override
  String emptyNotebookTitle(String notebook) {
    return 'No notes in $notebook yet';
  }

  @override
  String get emptyNotebookMessage =>
      'Notes added here will appear in this list.';

  @override
  String get noNoteSelectedTitle => 'No note selected';

  @override
  String get noNoteSelectedMessage =>
      'Choose a note from the list to read it here.';

  @override
  String get libraryDesignGallery => 'Design gallery';

  @override
  String get retry => 'Try again';

  @override
  String get libraryLoading => 'Loading the library';

  @override
  String get libraryFailedTitle => 'The library didn\'t load';

  @override
  String get libraryFailedMessage =>
      'Something went wrong reading the notebooks and tags on this device. Try again.';

  @override
  String get libraryNoNotebooks => 'No notebooks yet';

  @override
  String get libraryNoTags => 'No tags yet';

  @override
  String get dayToday => 'Today';

  @override
  String get dayYesterday => 'Yesterday';

  @override
  String get noteUntitled => 'Untitled';

  @override
  String get noteListLocked => 'Locked';

  @override
  String get noteListFailedTitle => 'These notes didn\'t load';

  @override
  String get noteListFailedMessage =>
      'Something went wrong reading the notes on this device. Try again.';

  @override
  String get emptyAllTitle => 'No notes yet';

  @override
  String get emptyAllMessage => 'Notes will appear here.';

  @override
  String get emptyPinnedTitle => 'Nothing pinned';

  @override
  String get emptyPinnedMessage =>
      'Pin a note by swiping it, and it will wait here.';

  @override
  String get emptyRecentTitle => 'Nothing recent';

  @override
  String get emptyRecentMessage =>
      'Notes you have changed lately will appear here.';

  @override
  String get emptyTrashTitle => 'The trash is empty';

  @override
  String get emptyTrashMessage =>
      'Notes you trash wait here until you move them out.';

  @override
  String emptyTagTitle(String tag) {
    return 'No notes tagged $tag';
  }

  @override
  String get emptyTagMessage => 'Notes with this tag will appear in this list.';

  @override
  String get emptyNarrowedTitle => 'No notes match';

  @override
  String get emptyNarrowedMessage =>
      'Try another status, or turn off Pinned Only.';

  @override
  String get listOptionsTitle => 'View options';

  @override
  String get listOptionsSortBy => 'Sort by';

  @override
  String get listOptionsDateUpdated => 'Date Updated';

  @override
  String get listOptionsDateCreated => 'Date Created';

  @override
  String get listOptionsTitleSort => 'Title';

  @override
  String get listOptionsNewestFirst => 'Newest First';

  @override
  String get listOptionsOldestFirst => 'Oldest First';

  @override
  String get listOptionsAToZ => 'A to Z';

  @override
  String get listOptionsZToA => 'Z to A';

  @override
  String get listOptionsStatus => 'Status';

  @override
  String get listOptionsAnyStatus => 'Any Status';

  @override
  String get listOptionsPinnedOnly => 'Pinned Only';

  @override
  String get noteListDone => 'Done';

  @override
  String noteListSelected(int count) {
    return '$count selected';
  }

  @override
  String get noteListPin => 'Pin';

  @override
  String get noteListUnpin => 'Unpin';

  @override
  String get noteListMove => 'Move';

  @override
  String get noteListTrash => 'Trash';

  @override
  String moveSheetTitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Move $count notes to',
      one: 'Move 1 note to',
    );
    return '$_temp0';
  }

  @override
  String get noteListCancel => 'Cancel';

  @override
  String get noteListChangeFailed => 'That didn\'t work. Try again.';

  @override
  String get noteListChangeLocked =>
      'Locked notebooks can\'t take or give up notes here yet.';

  @override
  String get libraryNewNotebook => 'New notebook';

  @override
  String get newNotebookTitle => 'New notebook';

  @override
  String get newNotebookNameHint => 'Name';

  @override
  String get newNotebookCreate => 'Create';

  @override
  String get newNotebookInvalid => 'Give the notebook a name.';

  @override
  String get newNotebookFailed => 'The notebook couldn\'t be made. Try again.';

  @override
  String get noteListNewNote => 'New note';

  @override
  String get noteListNewNoteLocked =>
      'Unlock this notebook to add notes to it.';

  @override
  String get noteListNewNoteFailed => 'The note couldn\'t be made. Try again.';

  @override
  String get noteListDismiss => 'OK';

  @override
  String get undoNoticeUndo => 'Undo';

  @override
  String get noteLoading => 'Loading the note';

  @override
  String get noteNotFoundTitle => 'This note isn\'t here';

  @override
  String get noteNotFoundMessage =>
      'It may have been deleted, or it hasn\'t reached this device yet.';

  @override
  String get noteFailedTitle => 'This note didn\'t load';

  @override
  String get noteFailedMessage =>
      'Something went wrong reading the note on this device. Try again.';

  @override
  String get lockedMessageSealed =>
      'Its text stays sealed. Only its title can be shown here.';

  @override
  String get menuCut => 'Cut';

  @override
  String get menuCopy => 'Copy';

  @override
  String get menuPaste => 'Paste';

  @override
  String get menuSelectAll => 'Select All';

  @override
  String get menuDelete => 'Delete';

  @override
  String get menuOpenLink => 'Open link';

  @override
  String completionAnnouncement(int count, String label) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count suggestions',
      one: '1 suggestion',
    );
    return '$_temp0, $label selected';
  }

  @override
  String get whenHintAction => 'Set';

  @override
  String whenHintSemantics(String date) {
    return '$date. Double tap to write this date into the task.';
  }

  @override
  String get searchIdleTitle => 'Search your notes';

  @override
  String get searchIdleMessage => 'Find words in titles and text, or a tag.';

  @override
  String get searchNoResultsTitle => 'No notes match';

  @override
  String searchNoResultsMessage(String query) {
    return 'Nothing matched “$query”. Try fewer or different words.';
  }

  @override
  String get searchFailedTitle => 'The search didn\'t run';

  @override
  String get searchFailedMessage =>
      'Something went wrong searching the notes on this device. Try again.';

  @override
  String get searchLoading => 'Searching';

  @override
  String settingsThemeBoth(String light, String dark) {
    return '$light, $dark';
  }

  @override
  String get settingsSyncStatus => 'Status';

  @override
  String get settingsSyncOff => 'Off';

  @override
  String get settingsSyncOn => 'On';

  @override
  String get settingsSyncSyncing => 'Syncing…';

  @override
  String get settingsSyncSynced => 'Synced';

  @override
  String get settingsSyncAttention => 'Needs attention';

  @override
  String get settingsSyncBackupFooter =>
      'The notes on this device aren’t included in device backups.';

  @override
  String get settingsPushWake => 'Wake this device when notes change elsewhere';

  @override
  String get settingsPushWakeNote =>
      'Kanso syncs in the background when a note changes on another device. Apple or Google only learn that something changed.';

  @override
  String get settingsPushUnavailable =>
      'Not available on this device. Kanso syncs when you open it.';

  @override
  String get settingsPushChangeFailed =>
      'That couldn’t be changed. Try again in a moment.';

  @override
  String get settingsPushDismiss => 'OK';

  @override
  String settingsVersionValue(String version, String build) {
    return '$version ($build)';
  }

  @override
  String get licencesLoading => 'Loading licences';

  @override
  String get licencesFailedTitle => 'The licences didn’t load';

  @override
  String get licencesFailedMessage =>
      'Something went wrong reading the licences. Try again.';

  @override
  String get licencesEmptyTitle => 'No licences';

  @override
  String get licencesEmptyMessage =>
      'The packages Kanso is made of will be listed here.';

  @override
  String licencesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count licences',
      one: '1 licence',
    );
    return '$_temp0';
  }

  @override
  String get themesFollowSystem =>
      'Switch between a light and a dark theme with the system';

  @override
  String get themesFollowSystemOn =>
      'Showing the light theme in light mode and the dark one in dark mode.';

  @override
  String get themesFollowSystemOff =>
      'Off: the theme stays as picked whatever the system’s mode.';

  @override
  String get themesFailedTitle => 'The themes didn’t load';

  @override
  String get themesFailedMessage =>
      'Something went wrong reading the themes. Try again.';

  @override
  String get themesLoading => 'Loading themes';

  @override
  String get themesFiles => 'Theme files';

  @override
  String get themesImportDetail =>
      'A theme package (its folder or .tgz) or a Zed theme (.json), converted into the themes folder.';

  @override
  String get themesImport => 'Import theme';

  @override
  String get themesImporting => 'Importing…';

  @override
  String get themesImportButton => 'Import Theme…';

  @override
  String themesImportedOne(String names) {
    return 'Imported $names.';
  }

  @override
  String themesImportFailed(String message) {
    return 'Couldn’t import the theme. $message';
  }

  @override
  String get settingsPreview => 'Preview';

  @override
  String get settingsDiagrams => 'Diagrams';

  @override
  String get settingsDiagramsDetail =>
      'What draws Mermaid diagrams, here and in exports. Pele, Obsidian\'s renderer, lays them out tidily in the theme\'s colors; Kanso\'s own also draws ZenUML.';

  @override
  String get diagramRendererPele => 'Pele';

  @override
  String get diagramRendererKanso => 'Kanso';

  @override
  String iconThemeAuthor(String author) {
    return 'By $author';
  }

  @override
  String get iconThemesTitle => 'Icon themes';

  @override
  String get iconThemesFailedTitle => 'The icon themes didn’t load';

  @override
  String get iconThemesFailedMessage =>
      'Something went wrong reading the icon themes. Try again.';

  @override
  String get iconThemesEmptyTitle => 'No icon themes yet';

  @override
  String get iconThemesEmptyMessage => 'Import one to see it here.';

  @override
  String get iconThemesListSection => 'Installed icon themes';

  @override
  String get iconThemesSection => 'Add an icon theme';

  @override
  String get iconThemesImportDetail =>
      'A Zed icon theme (its extension folder or .tar.gz), a theme package carrying one, or a Kanso icon theme folder, converted into the icon themes folder.';

  @override
  String get iconThemesImport => 'Import icon theme';

  @override
  String get iconThemesImportButton => 'Import…';

  @override
  String iconThemesImported(String names, String summary) {
    return 'Imported $names. $summary';
  }

  @override
  String iconThemesImportFailed(String message) {
    return 'Couldn’t import the icon theme. $message';
  }

  @override
  String get metricsNoticeTitle => 'Kanso counts, it doesn’t watch';

  @override
  String get metricsNoticeMessage =>
      'To know how many people use Kanso and on which devices, Kanso sends a small anonymous count every hour while it is open, when you open it and when you close it: the kind of device, Kanso’s version, your language, theme and editor mode as a plain label, how many notes, notebooks and plugins you have and how much room attachments take as ranges, how long it was open, and how often features were used, as ranges. Never your notes, titles, tags, files, name or anything you type, and never the name of a theme or plugin. Devices signed in to the same account count as one person, with an ID that can’t be traced back to the account.';

  @override
  String get metricsNoticeWhat => 'What’s sent, and what isn’t';

  @override
  String get metricsNoticeCode => 'The code';

  @override
  String get metricsNoticeOk => 'OK';

  @override
  String get backupNoticeTitle => 'Notes stay on this device';

  @override
  String get backupNoticeMessage =>
      'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.';

  @override
  String get backupNoticeOk => 'OK';

  @override
  String settingsVersionPatchValue(String version, String build, int patch) {
    return '$version ($build), patch $patch';
  }

  @override
  String get settingsPrivacy => 'Privacy';

  @override
  String get settingsPrivacyFooter =>
      'Kanso sends a crash report only when you choose to.';

  @override
  String get crashReportsTitle => 'Crash reports';

  @override
  String get crashReportsLoading => 'Loading crash reports';

  @override
  String get crashReportsEmptyTitle => 'No crash reports';

  @override
  String get crashReportsEmptyMessage =>
      'If Kanso closes unexpectedly, a report waits here until you decide whether to send it.';

  @override
  String get crashReportsIntro =>
      'A report holds only where in Kanso’s code something went wrong. It never holds note text, titles or file names. Nothing is sent unless you send it.';

  @override
  String get crashReportsSendAll => 'Send all';

  @override
  String crashReportSubtitle(String day, String kind) {
    return '$day · $kind';
  }

  @override
  String get crashReportSend => 'Send';

  @override
  String get crashReportDelete => 'Delete';

  @override
  String get crashReportShow => 'Show report';

  @override
  String get crashReportHide => 'Hide report';

  @override
  String get crashReportSending => 'Sending…';

  @override
  String get crashKindDart => 'App';

  @override
  String get crashKindIosNative => 'iOS';

  @override
  String get crashKindAndroidNative => 'Android native';

  @override
  String get crashKindAndroidAnr => 'Android not responding';

  @override
  String get crashKindAndroidJvm => 'Android';

  @override
  String get crashNoticeTitle => 'Kanso closed unexpectedly';

  @override
  String get crashNoticeMessage =>
      'Send a crash report? It holds only where in Kanso’s code it happened, never note text, titles or file names.';

  @override
  String get crashNoticeSend => 'Send';

  @override
  String get crashNoticeDiscard => 'Don’t send';

  @override
  String get crashOutcomeSent => 'Report sent. Thank you.';

  @override
  String get crashOutcomeRejected => 'The report couldn’t be sent.';

  @override
  String get crashOutcomeTryLater =>
      'Too many reports just now. Try again later.';

  @override
  String get crashOutcomeUnreachable =>
      'Couldn’t reach Kanso. Try again later.';

  @override
  String get crashReportsFailedMessage => 'The crash reports couldn’t be read.';

  @override
  String get screenNoteEdit => 'Note, edit';

  @override
  String get libraryTasks => 'Tasks';

  @override
  String libraryTasksOverdue(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count overdue tasks',
      one: '1 overdue task',
    );
    return '$_temp0';
  }

  @override
  String tasksCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count tasks',
      one: '1 task',
    );
    return '$_temp0';
  }

  @override
  String get tasksFilter => 'Filter tasks';

  @override
  String get tasksFilterTitle => 'Filter tasks';

  @override
  String get tasksAllNotebooks => 'All notebooks';

  @override
  String get tasksShowCompleted => 'Show Completed';

  @override
  String get tasksNotebooksHeader => 'Notebook';

  @override
  String get tasksEmptyTitle => 'No tasks yet';

  @override
  String get tasksEmptyMessage =>
      'Write “- [ ] Call Sam @2026-10-25” in any note, and it shows here.';

  @override
  String tasksEmptyNotebookTitle(String notebook) {
    return 'No tasks in $notebook';
  }

  @override
  String get tasksAllDoneTitle => 'All done';

  @override
  String get tasksAllDoneMessage => 'Every task is ticked off.';

  @override
  String get tasksLoading => 'Loading tasks';

  @override
  String get tasksFailedTitle => 'The tasks didn\'t load';

  @override
  String get tasksFailedMessage =>
      'Something went wrong reading the tasks on this device. Try again.';

  @override
  String get tasksMoved =>
      'That task changed in its note. The list is up to date now.';

  @override
  String get tasksChangeFailed => 'Couldn\'t change the task. Try again.';

  @override
  String get tasksTickOff => 'Tick off';

  @override
  String get tasksTickBack => 'Mark not done';

  @override
  String get tasksOpenNote => 'Open note';

  @override
  String tasksRowContext(String note, String notebook) {
    return '$note · $notebook';
  }

  @override
  String tasksRowDue(String due) {
    return 'Due $due';
  }

  @override
  String get previewLabel => 'The note\'s preview';

  @override
  String get previewFailed => 'The preview couldn\'t be drawn.';

  @override
  String get previewCopyCode => 'Copy code';

  @override
  String get previewTask => 'Task';

  @override
  String get pdfNotesTitle => 'Highlights';

  @override
  String get pdfNotesEmpty => 'This note has no highlights.';

  @override
  String pdfNotesPage(int page) {
    return 'Page $page';
  }

  @override
  String pdfNotesScanned(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count scanned pages were read on a computer and are searchable',
      one: '1 scanned page was read on a computer and is searchable',
    );
    return '$_temp0';
  }

  @override
  String get pdfNotesNotInNote => 'Not in the note';

  @override
  String get pdfNotesReadOnly =>
      'These highlights can\'t be changed here: the note\'s highlight data can\'t be read.';

  @override
  String get pdfNotesCommentHint => 'Add a note';

  @override
  String get pdfNotesSaveNote => 'Save note';

  @override
  String get pdfNotesAddToNote => 'Add quote to the note';

  @override
  String get pdfNotesDelete => 'Delete highlight';

  @override
  String pdfNotesColor(String color) {
    String _temp0 = intl.Intl.selectLogic(
      color,
      {
        'yellow': 'Yellow',
        'green': 'Green',
        'blue': 'Blue',
        'pink': 'Pink',
        'other': 'Colour',
      },
    );
    return '$_temp0';
  }

  @override
  String pdfNotesOpen(String summary) {
    return '$summary. Show the highlights.';
  }

  @override
  String get pdfReaderClose => 'Close';

  @override
  String pdfReaderPageOf(int page, int pages) {
    return 'Page $page of $pages';
  }

  @override
  String pdfReaderPageChip(int page, int pages) {
    return 'Page $page of $pages. Go to a page.';
  }

  @override
  String get pdfReaderLoading => 'Opening the PDF…';

  @override
  String get pdfReaderOpenFailed => 'The PDF couldn\'t be opened.';

  @override
  String get pdfReaderTryAgain => 'Try again';

  @override
  String get pdfReaderUnavailableTitle => 'This PDF can\'t be opened';

  @override
  String get pdfReaderLockedTitle => 'Locked';

  @override
  String get pdfReaderRefusedTitle => 'This PDF was refused';

  @override
  String get pdfReaderNotPdfTitle => 'This file isn\'t a PDF';

  @override
  String get pdfReaderNotPdfMessage =>
      'The attachment isn\'t a PDF, or its bytes can\'t be read as one.';

  @override
  String get pdfReaderPasswordTitle => 'This PDF is password-protected';

  @override
  String get pdfReaderPasswordMessage => 'Enter its password to read it.';

  @override
  String get pdfReaderPasswordHint => 'Password';

  @override
  String get pdfReaderWrongPassword => 'That password doesn\'t open this PDF.';

  @override
  String get pdfReaderUnlock => 'Open';

  @override
  String get pdfReaderSearch => 'Search';

  @override
  String get pdfReaderSearchHint => 'Search this PDF';

  @override
  String get pdfReaderSearchClear => 'Clear search';

  @override
  String get pdfReaderSearchSearching => 'Searching…';

  @override
  String get pdfReaderSearchNone => 'No matches';

  @override
  String pdfReaderSearchCount(int index, int total) {
    return '$index of $total';
  }

  @override
  String pdfReaderSearchNoText(int pages) {
    String _temp0 = intl.Intl.pluralLogic(
      pages,
      locale: localeName,
      other: '$pages pages have no text',
      one: '1 page has no text',
    );
    return '$_temp0';
  }

  @override
  String get pdfReaderPreviousMatch => 'Previous match';

  @override
  String get pdfReaderNextMatch => 'Next match';

  @override
  String get pdfReaderZoom => 'Zoom';

  @override
  String get pdfReaderOriginalPage => 'Show the page as printed';

  @override
  String get pdfReaderInvertPage => 'Invert for dark mode';

  @override
  String get pdfReaderZoomFitWidth => 'Fit width';

  @override
  String pdfReaderZoomPercent(int percent) {
    return '$percent%';
  }

  @override
  String get pdfReaderHighlights => 'Highlights';

  @override
  String pdfReaderHighlightPage(int page) {
    return 'Page $page';
  }

  @override
  String get pdfReaderNotEditable =>
      'This note\'s highlights can\'t be changed here.';

  @override
  String get pdfReaderAddQuote => 'Add quote to note';

  @override
  String get pdfReaderSelectionCopy => 'Copy';

  @override
  String get pdfReaderCopied => 'Copied';

  @override
  String get pdfReaderHighlighting => 'Highlighting…';

  @override
  String get pdfReaderHighlightAction => 'Highlight';

  @override
  String get pdfReaderPageFailed => 'This page couldn\'t be drawn.';

  @override
  String get pdfReaderJumpToPage => 'Go to page';

  @override
  String get pdfReaderPageNumber => 'Page number';

  @override
  String pdfReaderInvalidPage(int pages) {
    return 'Enter a page from 1 to $pages.';
  }

  @override
  String get pdfReaderGo => 'Go';

  @override
  String get previewEmptyDrawing => 'An empty drawing';

  @override
  String get previewCanvasLocked => 'Locked canvas';

  @override
  String get previewCanvasMissing => 'Canvas not found';

  @override
  String get previewCanvasNotCanvas => 'This note isn\'t a canvas';

  @override
  String get previewCanvasInvalid => 'Couldn\'t draw this canvas';

  @override
  String get previewCanvasWholeCanvas =>
      'This embeds the whole canvas. Name a group to embed less.';

  @override
  String get previewCanvasGroupMissing =>
      'That group is gone, so this shows the whole canvas.';

  @override
  String get previewCanvasOpen => 'Open canvas';

  @override
  String get previewQueryEmpty => 'No notes match.';

  @override
  String get previewQueryBadge => 'Query';

  @override
  String get previewLinkFailed => 'That link couldn\'t be opened.';

  @override
  String get previewLinkLocked => 'Unlock your notes to open that.';

  @override
  String get attachmentDone => 'Done';

  @override
  String get attachmentShare => 'Share';

  @override
  String attachmentDetails(String type, String size) {
    return '$type · $size';
  }

  @override
  String get settingsSyncSignIn => 'Sign in';

  @override
  String get settingsSyncOpenNotes => 'Open your notes on this device';

  @override
  String get settingsSyncNow => 'Sync now';

  @override
  String get settingsSyncAccount => 'Account';

  @override
  String get accountCancel => 'Cancel';

  @override
  String get accountDone => 'Done';

  @override
  String get accountRetry => 'Try again';

  @override
  String get accountCopy => 'Copy';

  @override
  String get accountCopied => 'Copied';

  @override
  String get accountSomethingWrong =>
      'Something went wrong. Try again in a moment.';

  @override
  String get accountSignedOutError =>
      'This device was signed out. Sign in again.';

  @override
  String get accountLoading => 'Loading';

  @override
  String get accountUsername => 'Username';

  @override
  String get accountPassword => 'Password';

  @override
  String get accountRepeatPassword => 'Repeat password';

  @override
  String get accountNewPassword => 'New password';

  @override
  String get accountCurrentPassword => 'Current password';

  @override
  String get formProblemUsernameInvalid =>
      'Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit.';

  @override
  String get formProblemUsernameReserved =>
      'That username is reserved. Try another.';

  @override
  String get formProblemPasswordTooShort =>
      'Use at least 12 characters. A few random words work well.';

  @override
  String get formProblemPasswordTooLong => 'Use at most 256 characters.';

  @override
  String get formProblemPasswordSameAsUsername =>
      'Choose a password that isn’t your username.';

  @override
  String get formProblemPasswordsDiffer => 'The passwords don’t match.';

  @override
  String get formProblemCredentialsDontMatch =>
      'That username and password don’t match.';

  @override
  String get formProblemRecoveryKeyMissing =>
      'A recovery key has 28 letters and digits, in groups of four.';

  @override
  String get accountServerLabel => 'Server';

  @override
  String get accountServerDefault => 'Kanso sync';

  @override
  String get accountServerChange => 'Use another server';

  @override
  String get accountServerAddress => 'Server address';

  @override
  String get accountServerAddressHint => 'https://sync.example.com';

  @override
  String get accountServerUse => 'Use this server';

  @override
  String get accountServerUseDefault => 'Use Kanso sync';

  @override
  String get accountServerChecking => 'Checking the server…';

  @override
  String get accountServerNoSignups => 'This server isn’t taking new accounts.';

  @override
  String get accountServerNoMethods =>
      'This server offers no way to sign in that this app can use.';

  @override
  String get accountSignInTitle => 'Sign in';

  @override
  String get accountSignInHeading => 'Sign in to Kanso';

  @override
  String get accountContinueWithApple => 'Continue with Apple';

  @override
  String get accountContinueWithGoogle => 'Continue with Google';

  @override
  String get accountContinueWithGithub => 'Continue with GitHub';

  @override
  String get accountSignInWithPasskey => 'Sign in with a passkey';

  @override
  String get accountOr => 'or';

  @override
  String get accountSignInButton => 'Sign in';

  @override
  String get accountCreateAccountLink => 'Create an account';

  @override
  String get accountForgotPassword =>
      'Forgot your password? Use your recovery key';

  @override
  String get accountSigningIn => 'Signing in…';

  @override
  String get accountCreateTitle => 'Create your account';

  @override
  String get accountCreateButton => 'Create account';

  @override
  String get accountCreatingButton => 'Creating…';

  @override
  String get accountCreateWithPasskey => 'Create with a passkey';

  @override
  String get accountCreateRecoveryNote =>
      'Kanso can’t reset your password; you’ll get a recovery key instead.';

  @override
  String get accountSignInInstead => 'I already have an account';

  @override
  String get accountGithubTitle => 'Continue with GitHub';

  @override
  String get accountGithubExplain => 'Type this code at GitHub to sign in.';

  @override
  String get accountGithubOpen => 'Open GitHub';

  @override
  String get accountGithubWaiting => 'Waiting for you to finish at GitHub…';

  @override
  String get accountProviderCancel => 'Cancel';

  @override
  String get accountProviderWaiting =>
      'Finish signing in in the page that opened. Kanso opens again when you’re done.';

  @override
  String get accountUsernameTitle => 'Choose a username';

  @override
  String get accountUsernameExplain =>
      'Your account is made with this name. You can change it before creating the account.';

  @override
  String get accountRecoveryKeyTitle => 'Your recovery key';

  @override
  String get accountRecoveryKeyText =>
      'If you forget your password, this key and your username are the only way back into your account. Kanso and the server’s owner can’t reset it. Anyone with it can take over your account and read your notes, so keep it somewhere safe, like a password manager or on paper. Each time it’s used, Kanso gives you a new one.';

  @override
  String get accountRecoveryKeyCopy => 'Copy recovery key';

  @override
  String get accountRecoveryKeyConfirm =>
      'I’ve saved my recovery key somewhere safe';

  @override
  String get accountRecoveryKeyNoneTitle => 'Nothing to save';

  @override
  String get accountRecoveryKeyNoneMessage =>
      'There’s no recovery key waiting to be saved.';

  @override
  String get accountRecoveryKeyFailedTitle => 'The recovery key didn’t load';

  @override
  String get accountRecoveryKeyFailedMessage =>
      'Something went wrong reading the recovery key. Try again.';

  @override
  String get bindingAdd => 'Add them to the account';

  @override
  String get bindingAddNote =>
      'They’re encrypted and uploaded. A note deleted in the account but still on this device comes back, and a note changed on both sides keeps both versions.';

  @override
  String get bindingReplace => 'Replace them with the account’s notes';

  @override
  String get bindingReplaceNote =>
      'Kanso saves a backup of this device’s library first, then downloads the account’s notes.';

  @override
  String get bindingCancel => 'Cancel';

  @override
  String get accountUnlockTitle => 'Open your notes on this device';

  @override
  String accountUnlockExplain(String username) {
    return 'You’re signed in as $username, but this device can’t read your notes yet.';
  }

  @override
  String get accountUnlockButton => 'Unlock';

  @override
  String get accountApproveFromDevice => 'Approve from another device';

  @override
  String get accountUseRecoveryKey => 'Use your recovery key';

  @override
  String get accountSignOut => 'Sign out';

  @override
  String get accountApprovalTitle => 'Approve this device';

  @override
  String get accountApprovalIntro =>
      'Open Kanso on a device that’s already signed in and approve this one.';

  @override
  String get accountApprovalRequesting => 'Asking another device…';

  @override
  String get accountApprovalWaiting => 'Waiting for another device…';

  @override
  String get accountApprovalUnlocking => 'Approved. Unlocking this device…';

  @override
  String get accountApprovalApproved => 'This device is unlocked.';

  @override
  String get accountApprovalNoDevices =>
      'No other device answered. Use your recovery key to open this device.';

  @override
  String get accountApprovalCancelled => 'That request was cancelled.';

  @override
  String get accountApprovalExpired => 'That request expired.';

  @override
  String get accountApprovalFailed =>
      'Can’t reach the server. Your notes are safe on this device.';

  @override
  String get accountApprovalCompare =>
      'On your other device, approve the request only if it shows this code.';

  @override
  String get accountApprovalRecoveryInstead => 'Use your recovery key instead';

  @override
  String accountApprovalCode(String code) {
    return 'Code $code';
  }

  @override
  String get accountRecoverTitle => 'Reset your password';

  @override
  String get accountRecoverExplain =>
      'Enter your recovery key and choose a new password. Your other devices sign out and ask for it, and you get a new recovery key to save.';

  @override
  String get accountRecoveryKeyField => 'Recovery key';

  @override
  String get accountRecoveryKeyHint => 'The key you saved, in groups of four.';

  @override
  String get accountRecoverButton => 'Reset password';

  @override
  String get accountRecoveringButton => 'Resetting…';

  @override
  String get accountTitle => 'Account';

  @override
  String accountSignedInAs(String username) {
    return 'Signed in as $username';
  }

  @override
  String get accountLoadFailedTitle => 'Your account didn’t load';

  @override
  String get accountLoadFailedMessage =>
      'Something went wrong reading your account. Try again.';

  @override
  String get accountRecoveryKeyUnsavedTitle =>
      'Your new recovery key isn’t saved yet';

  @override
  String get accountRecoveryKeyUnsavedMessage =>
      'Save it now: the one you had before no longer works.';

  @override
  String get accountRecoveryKeyShow => 'Show it';

  @override
  String get accountDevices => 'Devices';

  @override
  String get accountThisDevice => 'This device';

  @override
  String accountLastSeen(String date) {
    return 'Last seen $date';
  }

  @override
  String get accountDeviceSignOut => 'Sign out';

  @override
  String get accountWaysToSignIn => 'Ways to sign in';

  @override
  String get accountPasswordRow => 'Password';

  @override
  String get accountPasswordChange => 'Change';

  @override
  String get accountPasswordNotSet => 'Not set';

  @override
  String get accountPasskeys => 'Passkeys';

  @override
  String get accountPasskeyAdd => 'Add a passkey';

  @override
  String get accountPasskeyRemove => 'Remove';

  @override
  String accountPasskeyAdded(String date) {
    return 'Added $date';
  }

  @override
  String accountPasskeyUsed(String added, String used) {
    return 'Added $added · Last used $used';
  }

  @override
  String get accountPasskeyCanUnlock => 'Can unlock a new device';

  @override
  String get accountPasskeySignsInOnly => 'Signs in only';

  @override
  String get accountProviderApple => 'Apple';

  @override
  String get accountProviderGoogle => 'Google';

  @override
  String get accountProviderGithub => 'GitHub';

  @override
  String get accountProviderLinked => 'Linked';

  @override
  String get accountProviderNotLinked => 'Not linked';

  @override
  String get accountProviderLink => 'Link';

  @override
  String get accountProviderUnlink => 'Unlink';

  @override
  String get accountNoProofHere =>
      'Confirm it’s you with your password. This device can’t use another way for this account yet.';

  @override
  String get accountRecoveryKeySection => 'Recovery key';

  @override
  String get accountRecoveryKeyNew => 'Make a new one';

  @override
  String get accountRecoveryKeyFooter =>
      'Your recovery key still works if you lose your other ways in.';

  @override
  String get accountStorage => 'Storage';

  @override
  String get accountCost => 'What Kanso costs';

  @override
  String get accountCostLearnMore => 'Learn more';

  @override
  String get accountDeleteAccount => 'Delete account';

  @override
  String get accountWorking => 'Working…';

  @override
  String accountRevokeTitle(String name) {
    return 'Sign out $name?';
  }

  @override
  String get accountRevokeMessage =>
      'It stops syncing until someone signs in on it again. Its notes stay on it.';

  @override
  String accountSignOutTitle(String username) {
    return 'Sign out of $username?';
  }

  @override
  String get accountSignOutMessage =>
      'Notes stay on this device. Sync stops until you sign in again.';

  @override
  String get accountSignOutUnsavedKey =>
      'Your new recovery key hasn’t been saved yet; signing out deletes it.';

  @override
  String get accountChangePasswordTitle => 'Change password';

  @override
  String get accountChangePasswordMessage =>
      'Your other devices sign out and ask for the new password. Your recovery key changes too: the one you saved stops working, and Kanso shows the new one to save.';

  @override
  String get accountNewRecoveryKeyTitle => 'New recovery key';

  @override
  String get accountNewRecoveryKeyMessage =>
      'The key you saved stops working, and Kanso shows the new one to save instead.';

  @override
  String get accountNewRecoveryKeyButton => 'Make a new key';

  @override
  String get accountDeleteTitle => 'Delete account';

  @override
  String accountDeleteMessage(String username) {
    return 'This deletes $username’s encrypted notes, attachments and devices from the server. Notes on this device stay here, and other devices keep their copies. The server’s backups keep a copy for up to 30 days. This can’t be undone.';
  }

  @override
  String accountDeleteTypeUsername(String username) {
    return 'Type $username to confirm';
  }

  @override
  String get accountAddPasskeyTitle => 'Add a passkey';

  @override
  String get accountAddPasskeyMessage =>
      'Confirm it’s you with your password, then use your phone’s passkey sheet.';

  @override
  String accountRemovePasskeyTitle(String label) {
    return 'Remove $label?';
  }

  @override
  String get accountRemovePasskeyMessage =>
      'You won’t be able to sign in with it any more. Confirm it’s you with your password.';

  @override
  String get accountRemovePasskeyButton => 'Remove passkey';

  @override
  String accountLinkTitle(String provider) {
    return 'Link $provider';
  }

  @override
  String get accountLinkMessage =>
      'Confirm it’s you with your password, then finish in the page that opens.';

  @override
  String accountUnlinkTitle(String provider) {
    return 'Unlink $provider?';
  }

  @override
  String get accountUnlinkMessage =>
      'It no longer signs in to this account. Confirm it’s you with your password.';

  @override
  String get accountConfirmSignOut => 'Sign out';

  @override
  String accountNoticeDeviceSignedOut(String name) {
    return 'Signed out $name.';
  }

  @override
  String get accountNoticePasskeyAddedCanUnlock =>
      'Added the passkey. It can unlock: on a new device it signs in and opens your notes.';

  @override
  String get accountNoticePasskeyAddedSignsIn =>
      'Added the passkey. It signs in only: a new device will also need approval or your recovery key.';

  @override
  String get accountNoticePasskeyRemoved => 'Removed the passkey.';

  @override
  String get accountNoticeProviderLinked =>
      'Linked. You can sign in with it now.';

  @override
  String get accountNoticeProviderUnlinked =>
      'Unlinked. It no longer signs in to this account.';

  @override
  String get approveDeviceTitle => 'Approve a device';

  @override
  String approveDeviceMessage(String account, String device, String time) {
    return 'A device wants to join $account: $device, asked at $time.';
  }

  @override
  String get approveDeviceUnnamed => 'A device';

  @override
  String approveDeviceCompare(String code) {
    return 'Does it show $code? Only approve if it does.';
  }

  @override
  String get approveDeviceChanged =>
      'This request changed after it was made. Don’t approve.';

  @override
  String get approveDeviceApprove => 'Approve';

  @override
  String get approveDeviceDecline => 'Don’t approve';

  @override
  String approveDeviceMore(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count more requests are waiting.',
      one: '1 more request is waiting.',
    );
    return '$_temp0';
  }

  @override
  String get accountServerFailedTitle => 'This server didn’t answer';

  @override
  String get accountDeleteMismatch => 'That isn’t this account’s username.';

  @override
  String get shareTitle => 'Share';

  @override
  String get shareClose => 'Close';

  @override
  String get shareLoading => 'Loading sharing';

  @override
  String get shareLockedNote =>
      'Locked notes, and notes in locked notebooks, can’t be shared. Remove the lock first.';

  @override
  String get shareSignedOut =>
      'Sharing lets people you choose read, comment on or edit a note with you, end-to-end encrypted. It needs a Kanso account: sign in, or create one, then share.';

  @override
  String get shareSignIn => 'Sign in…';

  @override
  String get shareFailedTitle => 'Sharing didn’t load';

  @override
  String get shareFailedMessage =>
      'Something went wrong reading this note’s sharing. Try again.';

  @override
  String get shareIntro =>
      'Invite people by their Kanso username to read, comment on or edit this note with you. It stays end-to-end encrypted: your sync server passes it on without reading it.';

  @override
  String shareSharedWithYou(String owner, String access) {
    return '$owner shared this note with you. You $access.';
  }

  @override
  String get shareAccessView => 'can read it';

  @override
  String get shareAccessComment => 'can read it and comment on it';

  @override
  String get shareAccessEdit => 'can edit it';

  @override
  String get shareUsernameHint => 'Username';

  @override
  String get shareInvite => 'Invite';

  @override
  String get shareInviteRoleLabel => 'What the person you invite can do';

  @override
  String get shareRoleOwner => 'Owner';

  @override
  String get shareRoleEditor => 'Can edit';

  @override
  String get shareRoleCommenter => 'Can comment';

  @override
  String get shareRoleViewer => 'Can view';

  @override
  String shareInvited(String username) {
    return 'Invited $username. The note shows for them once they accept.';
  }

  @override
  String get shareStopped =>
      'Sharing stopped. Everyone keeps a private copy of the note as it is.';

  @override
  String get shareLeft => 'You left the note. Your copy of it is private now.';

  @override
  String get shareProblemNameMissing =>
      'Type the username of the person to invite.';

  @override
  String get shareProblemLinkMissing => 'Paste the invite link first.';

  @override
  String get shareProblemOffline =>
      'Couldn’t reach your sync server. Check your connection and try again.';

  @override
  String get shareProblemSignedOut =>
      'Your account signed this device out. Sign in again to share.';

  @override
  String get shareProblemUnexpected => 'Something went wrong. Try again.';

  @override
  String get sharePeople => 'People';

  @override
  String get shareMemberInvited => 'Invited, hasn’t accepted yet';

  @override
  String get shareMemberDeclined => 'Declined';

  @override
  String get shareMemberLeft => 'Left';

  @override
  String get shareMemberYou => 'You';

  @override
  String get shareMemberUnchecked => 'Can’t be checked';

  @override
  String get shareMemberVerified => 'Fingerprint verified';

  @override
  String get shareMemberAcceptKey => 'Accept the new key';

  @override
  String get shareMemberRemove => 'Remove from note';

  @override
  String shareMemberRoleLabel(String username) {
    return 'Change what $username can do';
  }

  @override
  String get shareRequests => 'Asked to join';

  @override
  String shareRequestAsked(String username) {
    return '$username asked to join';
  }

  @override
  String shareRequestDetail(String role, String fingerprint) {
    return '$role · Fingerprint $fingerprint';
  }

  @override
  String get shareApprove => 'Approve';

  @override
  String get shareDecline => 'Decline';

  @override
  String get shareLinks => 'Invite links';

  @override
  String shareLinkExpired(String role) {
    return '$role, expired';
  }

  @override
  String shareLinkDaysLeft(String role, int days) {
    String _temp0 = intl.Intl.pluralLogic(
      days,
      locale: localeName,
      other: '$role, for $days more days',
      one: '$role, for 1 more day',
    );
    return '$_temp0';
  }

  @override
  String get shareLinkCancel => 'Cancel link';

  @override
  String get shareLinkMake => 'Make link';

  @override
  String get shareLinkRoleLabel => 'What people who join with the link can do';

  @override
  String get shareLinkDaysLabel => 'How long the link works';

  @override
  String shareLinkDays(int days) {
    String _temp0 = intl.Intl.pluralLogic(
      days,
      locale: localeName,
      other: '$days days',
      one: '1 day',
    );
    return '$_temp0';
  }

  @override
  String get shareLinkAddress => 'Invite link';

  @override
  String get shareLinkCopy => 'Copy';

  @override
  String get shareLinkCopied => 'Copied';

  @override
  String get shareLinkSend => 'Send';

  @override
  String get shareLinkNote =>
      'Anyone with the link can ask to join; you approve each of them here. Copy it now: it can’t be shown again.';

  @override
  String get shareStop => 'Stop sharing';

  @override
  String get shareStopEveryone => 'Stop sharing for everyone';

  @override
  String get shareLeave => 'Leave';

  @override
  String get shareLeaveNote => 'Leave the note';

  @override
  String get shareStopWarning =>
      'Stopping sharing ends the note for everyone in it. Each of them keeps a private copy of it as it is now.';

  @override
  String get shareLeaveWarning =>
      'Leaving takes you out of the note. You keep a private copy of it as it is now.';

  @override
  String get shareKeepSharing => 'Keep sharing';

  @override
  String get shareBack => 'Back';

  @override
  String shareVerifyIntro(String username) {
    return 'You’re sharing with $username for the first time. Compare this fingerprint with the one they see in Settings › Account, in person or on a call. If it matches, nobody is standing between you.';
  }

  @override
  String get shareVerifyConfirm => 'Mark as verified and invite';

  @override
  String get shareInviteUnverified => 'Invite';

  @override
  String shareKeyChangedIntro(String username) {
    return '$username’s keys changed since you last shared with them. That happens when they set up their account again, and also when someone tries to read what you share. Ask them which fingerprint they see now before you trust it.';
  }

  @override
  String get shareKeyBefore => 'Before';

  @override
  String get shareKeyNow => 'Now';

  @override
  String get shareKeyTrust => 'Trust the new keys';

  @override
  String get shareKeyDontInvite => 'Don’t invite';

  @override
  String get shareKeyNotNow => 'Not now';

  @override
  String shareFingerprintLabel(String username) {
    return '$username’s fingerprint';
  }

  @override
  String get sharedWithYouTitle => 'Shared with you';

  @override
  String get sharedWithYouEmptyTitle => 'Nothing shared with you';

  @override
  String get sharedWithYouEmptyMessage =>
      'When someone shares a note with you, it waits here until you accept it.';

  @override
  String get sharedWithYouSignedOutTitle => 'Sign in to get shared notes';

  @override
  String get sharedWithYouSignedOutMessage =>
      'Notes other people share with you reach this phone once it is signed in to a Kanso account. Sign in from Settings.';

  @override
  String get sharedWithYouFailedTitle => 'Shared notes didn’t load';

  @override
  String get sharedWithYouFailedMessage =>
      'Something went wrong looking for notes shared with you. Try again.';

  @override
  String get sharedWithYouLoading => 'Loading shared notes';

  @override
  String sharedInvitationFrom(String from) {
    return 'A note from $from';
  }

  @override
  String get sharedAccept => 'Accept';

  @override
  String get sharedJoinWithLink => 'Join with a link';

  @override
  String get joinTitle => 'Join with a link';

  @override
  String get joinMessage =>
      'Paste the invite link you were sent. The note’s owner lets you in, and the note comes to your library once they do.';

  @override
  String get joinHint => 'Invite link';

  @override
  String get joinButton => 'Join';

  @override
  String joinAsked(String owner) {
    return 'Asked $owner to let you in. The note shows in your library once they approve.';
  }

  @override
  String get joinAnother => 'Join another note';

  @override
  String get galleryTitle => 'Gallery';

  @override
  String get gallerySearchHint => 'Search themes';

  @override
  String get gallerySearchClear => 'Clear search';

  @override
  String get galleryFilterLabel => 'Order of the themes';

  @override
  String get galleryFilterPopular => 'Popular';

  @override
  String get galleryFilterRecent => 'Recently updated';

  @override
  String get galleryBadgeOfficial => 'Official';

  @override
  String get galleryBadgeCommunity => 'Community';

  @override
  String get galleryBadgeIcons => 'Icons';

  @override
  String get galleryBadgeWithdrawn => 'Withdrawn';

  @override
  String galleryBy(String author) {
    return 'by $author';
  }

  @override
  String galleryVersion(String version) {
    return 'Version $version';
  }

  @override
  String galleryInstalls(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count installs',
      one: '1 install',
    );
    return '$_temp0';
  }

  @override
  String get galleryInstall => 'Install';

  @override
  String get galleryInstalled => 'Installed';

  @override
  String get galleryUpdate => 'Update';

  @override
  String get galleryRemove => 'Remove';

  @override
  String get galleryInstalling => 'Installing…';

  @override
  String get galleryUpdating => 'Updating…';

  @override
  String get galleryRemoving => 'Removing…';

  @override
  String get galleryNeedsNewer => 'Needs a newer Kanso';

  @override
  String get galleryLoading => 'Loading the gallery';

  @override
  String get galleryFailedTitle => 'The gallery didn’t load';

  @override
  String get galleryFailedMessage =>
      'Couldn’t reach the theme gallery. Check your connection and try again.';

  @override
  String get galleryStale =>
      'Showing the last list. Couldn’t reach the gallery just now.';

  @override
  String get galleryRefreshing => 'Checking for new themes…';

  @override
  String get galleryNoResultsTitle => 'No themes match';

  @override
  String galleryNoResultsMessage(String query) {
    return 'Nothing in the gallery matches “$query”. Try another word.';
  }

  @override
  String get galleryEmptyTitle => 'The gallery is empty';

  @override
  String get galleryEmptyMessage => 'There are no themes to show yet.';

  @override
  String get galleryProblemOffline =>
      'Couldn’t reach the gallery. Check your connection and try again.';

  @override
  String get galleryProblemUnexpected => 'Something went wrong. Try again.';

  @override
  String get sharedWithYouRefresh => 'Look for new shared notes';

  @override
  String get joinSentTitle => 'Request sent';

  @override
  String get timeJustNow => 'just now';

  @override
  String timeMinutesAgo(int minutes) {
    return '$minutes min ago';
  }

  @override
  String timeHoursAgo(int hours) {
    return '$hours h ago';
  }

  @override
  String get timeToday => 'today';

  @override
  String get timeYesterday => 'yesterday';

  @override
  String timeResumeAt(String time) {
    return 'at $time';
  }

  @override
  String timeResumeTomorrow(String time) {
    return 'tomorrow at $time';
  }

  @override
  String timeResumeOn(String day, String time) {
    return 'on $day at $time';
  }

  @override
  String get settingsSyncAutomatically => 'Sync automatically';

  @override
  String get settingsSyncAutomaticallyDetail =>
      'In the background, and soon after you edit.';

  @override
  String get settingsSyncEvery => 'Sync every';

  @override
  String get settingsSyncEveryTitle => 'Sync every';

  @override
  String get commentsTitle => 'Comments';

  @override
  String commentsButton(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Comments, $count open',
      one: 'Comments, 1 open',
      zero: 'Comments',
    );
    return '$_temp0';
  }

  @override
  String get commentsOrphaned => 'Orphaned';

  @override
  String get commentsNotShared => 'Comments are for shared notes';

  @override
  String get commentsNotSharedHint =>
      'Share this note to talk it over with others.';

  @override
  String get commentsEmpty => 'No comments yet';

  @override
  String get commentsEmptyHint => 'Select some words, then choose Comment.';

  @override
  String get commentsEmptyHintReader =>
      'Nobody has commented on this note yet.';

  @override
  String get commentEditedSuffix => ' · Edited';

  @override
  String get commentSendingSuffix => ' · Sending…';

  @override
  String get commentResolvedMark => 'Resolved';

  @override
  String get commentResolve => 'Resolve';

  @override
  String get commentReopen => 'Reopen';

  @override
  String get commentEdit => 'Edit';

  @override
  String get commentDelete => 'Delete';

  @override
  String get commentDeleteWarning =>
      'Delete this comment? It goes for everyone in the note.';

  @override
  String get commentEditTitle => 'Edit comment';

  @override
  String get commentSave => 'Save';

  @override
  String get commentNewTitle => 'Comment on the selection';

  @override
  String get commentNewAction => 'Comment';

  @override
  String get commentHint => 'Your comment';

  @override
  String get menuComment => 'Comment';

  @override
  String get sharedStatusShared => 'Shared';

  @override
  String get sharedStatusReadOnly => 'Read only';

  @override
  String get sharedStatusLive => 'Live';

  @override
  String get sharedStatusOffline => 'Offline';

  @override
  String get sharedStatusNearLimit => 'Near this shared note\'s size limit';

  @override
  String get sharedStatusPrivateCopy => 'Private copy · No longer shared';

  @override
  String sharedProblemsMore(String message, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count more',
      one: '1 more',
    );
    return '$message ($_temp0)';
  }

  @override
  String get sharedProblemsDismiss => 'Dismiss';

  @override
  String get dailyNoteLoading => 'Opening today’s note';

  @override
  String get dailyNoteFailedTitle => 'Today’s note didn’t open';

  @override
  String get dailyNoteFailedMessage =>
      'Something went wrong opening today’s note on this phone. Try again.';

  @override
  String get captureTakePhoto => 'Take photo';

  @override
  String get captureChoosePhotos => 'Choose photos';

  @override
  String get captureAttachPdf => 'Attach PDF';

  @override
  String get captureImageLink => 'Image link';

  @override
  String get captureAttachFailed => 'Couldn\'t attach that file.';

  @override
  String lockedMessage(int minutes) {
    return 'Unlock it to read and edit. It locks again after $minutes minutes or when you leave Kanso.';
  }

  @override
  String lockedUnlockWith(String method) {
    return 'Unlock with $method';
  }

  @override
  String get lockedUsePassword => 'Use password';

  @override
  String lockedBiometricFailed(String method) {
    return '$method didn\'t work. Use your lock password instead.';
  }

  @override
  String lockedInvalidated(String method) {
    return 'Your biometrics changed, so $method no longer opens locked notes. Use your lock password, then turn $method on again in Settings.';
  }

  @override
  String get biometricFaceId => 'Face ID';

  @override
  String get biometricTouchId => 'Touch ID';

  @override
  String get biometricOpticId => 'Optic ID';

  @override
  String get biometricFingerprint => 'fingerprint';

  @override
  String get biometricFace => 'face unlock';

  @override
  String get biometricIris => 'iris unlock';

  @override
  String get biometricGeneric => 'biometrics';

  @override
  String get biometricReasonUnlock => 'Unlock your locked notes';

  @override
  String get biometricReasonEnroll =>
      'Use it to unlock your locked notes on this device';

  @override
  String get lockFieldPassword => 'Lock password';

  @override
  String get lockFieldPasswordHint => 'At least 8 characters.';

  @override
  String get lockFieldAgain => 'Type it again';

  @override
  String get lockFieldCurrent => 'Current lock password';

  @override
  String get lockFieldNew => 'New lock password';

  @override
  String get lockCancel => 'Cancel';

  @override
  String get lockWrongPassword => 'That isn\'t the lock password.';

  @override
  String get lockMismatch => 'The passwords don\'t match.';

  @override
  String get lockNewMismatch => 'The new passwords don\'t match.';

  @override
  String get lockTooShort => 'The lock password needs at least 8 characters.';

  @override
  String get lockActionFailed => 'That didn\'t work. Try again.';

  @override
  String get lockUnlockNoteTitle => 'Unlock note';

  @override
  String get lockUnlockNotebookTitle => 'Unlock notebook';

  @override
  String lockUnlockNotebookMessage(String name) {
    return '“$name” is locked. Its notes open with your lock password.';
  }

  @override
  String get lockUnlockButton => 'Unlock';

  @override
  String get lockSetTitle => 'Set a lock password';

  @override
  String get lockSetMessage =>
      'Locked notes open with this password, on this device and any other that syncs with it. Kanso can\'t recover it: forget it, and the notes locked with it can\'t be read.';

  @override
  String get lockSetButton => 'Lock';

  @override
  String get lockNoteMenu => 'Lock note…';

  @override
  String get lockNoteAskTitle => 'Lock note';

  @override
  String get lockNoteConfirmTitle => 'Lock this note?';

  @override
  String get lockNoteConfirmMessage =>
      'Its text is sealed, and its earlier versions are deleted. It stays open until the locked notes lock again.';

  @override
  String get lockNotebookAskTitle => 'Lock notebook';

  @override
  String get lockNotebookConfirmTitle => 'Lock this notebook?';

  @override
  String get lockNotebookConfirmMessage =>
      'Every note in it is sealed, and new notes added to it are sealed too.';

  @override
  String get lockNotebookMenu => 'Lock Notebook…';

  @override
  String get lockRemoveNoteMenu => 'Remove lock…';

  @override
  String get lockRemoveNotebookMenu => 'Remove Notebook Lock…';

  @override
  String get lockUnlockNotebookMenu => 'Unlock notebook…';

  @override
  String get lockNowMenu => 'Lock notes now';

  @override
  String get lockRemoveNoteTitle => 'Remove the lock?';

  @override
  String get lockRemoveNoteMessage =>
      'The note\'s text is kept as plain text again: search, backlinks, tasks and the local API can read it. Its pictures and files are decrypted again too; one that another locked note also shows is copied, so that note keeps its sealed copy.';

  @override
  String get lockRemoveNotebookTitle => 'Remove the notebook\'s lock?';

  @override
  String lockRemoveNotebookMessage(String name) {
    return 'Notes put in “$name” from now on aren\'t locked. The notes in it now stay locked: remove a note\'s lock to keep its text as plain text again.';
  }

  @override
  String get lockRemoveButton => 'Remove Lock';

  @override
  String get lockRemoveClosed => 'Unlock the notes first, then try again.';

  @override
  String get lockMoreMenu => 'More';

  @override
  String get lockMoreTitle => 'Note actions';

  @override
  String get lockNotebookMenuTitle => 'Notebook actions';

  @override
  String get noteRowLocked => 'Locked';

  @override
  String get noteRowUnlocked => 'Unlocked';

  @override
  String get noteListUnlocked => 'Unlocked';

  @override
  String get settingsLockedNotes => 'Locked notes';

  @override
  String settingsUnlockWith(String method) {
    return 'Unlock with $method';
  }

  @override
  String get settingsLockAfter => 'Lock after';

  @override
  String settingsTenMinutes(int minutes) {
    return '$minutes minutes';
  }

  @override
  String get settingsChangeLockPassword => 'Change lock password…';

  @override
  String get settingsLockNow => 'Lock notes now';

  @override
  String settingsBiometricsClosed(String method) {
    return 'Unlock a locked note with your lock password first, then turn on $method.';
  }

  @override
  String settingsBiometricsOn(String method) {
    return '$method now unlocks your locked notes on this device.';
  }

  @override
  String get settingsBiometricsOff =>
      'Locked notes open with the lock password again on this device.';

  @override
  String settingsBiometricsFailed(String method) {
    return '$method could not be turned on.';
  }

  @override
  String get lockChangeTitle => 'Change lock password';

  @override
  String get lockChangeMessage =>
      'Every locked note is sealed again with the new password. Kanso can\'t recover it.';

  @override
  String get lockChangeButton => 'Change';

  @override
  String lockChangeKept(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count locked notes or notebooks didn\'t open with the current password and kept it.',
      one:
          '1 locked note or notebook didn\'t open with the current password and kept it.',
    );
    return '$_temp0';
  }

  @override
  String lockOfferTitle(String method) {
    return 'Use $method?';
  }

  @override
  String lockOfferMessage(String method) {
    return 'Use $method to unlock your locked notes on this device? The password still works, and other devices need it.';
  }

  @override
  String lockOfferAccept(String method) {
    return 'Use $method';
  }

  @override
  String get lockOfferDecline => 'Not now';

  @override
  String get lockDone => 'Done';

  @override
  String get importTitle => 'Import notes';

  @override
  String get importEmptyButton => 'Import notes';

  @override
  String get importIntro =>
      'Bring notes in from another app. Kanso only reads what you choose, and importing again updates the notes it made without undoing your edits.';

  @override
  String get importAppleNotesRow => 'Apple Notes';

  @override
  String get importAppleNotesTitle => 'Import Apple Notes';

  @override
  String get importAppleNotesIntro =>
      'Apple Notes doesn’t let other apps read its library on iPhone and iPad. These are the ways across.';

  @override
  String get importAppleNotesMacTitle =>
      'Import on your Mac and sync (recommended)';

  @override
  String get importAppleNotesMacBody =>
      'On your Mac, open Kanso and choose File › Import… › Apple Notes. Sync brings everything here: folders, checklists, tables, scans, drawings and attachments. Locked notes can’t be read; Kanso lists them.';

  @override
  String get importAppleNotesShareTitle => 'Share notes one at a time';

  @override
  String get importAppleNotesShareBody =>
      'In Notes, open a note, tap Share, then choose Kanso. It arrives in your Inbox as a Markdown note with its checklists and images.';

  @override
  String get importAppleNotesFilesTitle => 'Choose exported files';

  @override
  String get importAppleNotesFilesBody =>
      'Pick a folder or .zip of notes exported as PDF, RTF, HTML, text or Markdown. They go in an Apple Notes notebook, with the folders you had inside it.';

  @override
  String get importAppleNotesFilesAction => 'Choose exported files';

  @override
  String get importAppleNotesHelp => 'Read more in Help';

  @override
  String get importSourcesLabel => 'Import from';

  @override
  String get importPickFile => 'Choose a file';

  @override
  String get importPickFolder => 'Choose a folder';

  @override
  String importPickTitle(Object source) {
    return 'Import from $source';
  }

  @override
  String get importPicking => 'Waiting for your choice…';

  @override
  String get importReadingTitle => 'Reading your notes';

  @override
  String get importReadingMessage => 'Nothing is written yet.';

  @override
  String get importWritingTitle => 'Importing';

  @override
  String get importWritingMessage => 'Keep Kanso open until it finishes.';

  @override
  String get importSummaryTitle => 'Ready to import';

  @override
  String importSummaryFrom(Object name) {
    return 'From $name';
  }

  @override
  String get importSummaryEmptyTitle => 'Nothing to import';

  @override
  String importSummaryEmptyMessage(Object name) {
    return 'Kanso found no notes in $name.';
  }

  @override
  String get importButton => 'Import';

  @override
  String get importChooseAnother => 'Choose another';

  @override
  String get importDoneTitle => 'Import finished';

  @override
  String get importOpenFirst => 'Open the first note';

  @override
  String get importDoneButton => 'Done';

  @override
  String get importSkippedTitle => 'Skipped';

  @override
  String get importRejectedTitle => 'Not imported';

  @override
  String get importFailedTitle => 'The import didn’t finish';

  @override
  String get importTryAgain => 'Try again';

  @override
  String get importProblemPickFailed =>
      'Kanso couldn’t read what you chose. Try choosing it again.';

  @override
  String get importProblemNotAnExport =>
      'That file isn’t an export Kanso can read.';

  @override
  String get importProblemUnexpected => 'Something went wrong. Try again.';

  @override
  String get openedDocumentsFailedTitle => 'The file wasn’t opened';

  @override
  String get openedDocumentsFailedMessage =>
      'Kanso couldn’t add it to your notes. Try opening it again.';

  @override
  String get openedDocumentsProblemsTitle => 'Some files weren’t opened';

  @override
  String get openedDocumentsOk => 'OK';

  @override
  String get backupsTitle => 'Backups';

  @override
  String get backupsSchedule => 'Once a day · last 14';

  @override
  String backupsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count backups',
      one: '1 backup',
      zero: 'None yet',
    );
    return '$_temp0';
  }

  @override
  String get backupsSummary =>
      'Kanso backs up its database on this device once a day and keeps the last 14. A backup made with Back up now is kept until you delete it here.';

  @override
  String get backupsBackUpNow => 'Back up now';

  @override
  String get backupsBackingUp => 'Backing up…';

  @override
  String get backupsListHeader => 'Backups';

  @override
  String get backupsKindDaily => 'Daily backup';

  @override
  String get backupsKindManual => 'Made with Back up now';

  @override
  String backupsKindManualKept(String kind, String size) {
    return '$kind · $size';
  }

  @override
  String get backupsDelete => 'Delete backup';

  @override
  String get backupsEmptyTitle => 'No backups yet';

  @override
  String get backupsEmptyMessage => 'Back up now makes one right away.';

  @override
  String get backupsLoading => 'Reading the backups';

  @override
  String get backupsFailedTitle => 'The backups couldn’t be read';

  @override
  String get backupsFailedMessage => 'Try again in a moment.';

  @override
  String get backupsNoticeMadeNow => 'The backup was made.';

  @override
  String get backupsNoticeDeleted => 'The backup was deleted.';

  @override
  String get backupsProblemDaily => 'Today’s backup couldn’t be made.';

  @override
  String get backupsProblemBackUp => 'The backup couldn’t be made.';

  @override
  String get backupsProblemDelete => 'The backup couldn’t be deleted.';

  @override
  String get backupsProblemList => 'The list of backups couldn’t be updated.';

  @override
  String get backupsDeleteTitle => 'Delete this backup?';

  @override
  String backupsDeleteMessage(String when) {
    return 'The backup made $when is deleted from this device. This can’t be undone.';
  }

  @override
  String get backupsDeleteConfirm => 'Delete';

  @override
  String get backupsDeleteCancel => 'Cancel';

  @override
  String fileSizeBytes(String size) {
    return '$size B';
  }

  @override
  String fileSizeKilobytes(String size) {
    return '$size KB';
  }

  @override
  String fileSizeMegabytes(String size) {
    return '$size MB';
  }

  @override
  String fileSizeGigabytes(String size) {
    return '$size GB';
  }

  @override
  String get noteInfoHistory => 'Show history';

  @override
  String get historyTitle => 'Note history';

  @override
  String get historyClose => 'Close';

  @override
  String get historyLoading => 'Reading the history';

  @override
  String get historyEmptyTitle => 'No earlier versions yet';

  @override
  String get historyEmptyMessage =>
      'Kanso keeps one every few minutes while you edit, and one before every change made elsewhere.';

  @override
  String get historyFailedTitle => 'The history couldn’t be read';

  @override
  String get historyFailedMessage => 'Try again in a moment.';

  @override
  String get historyLockedTitle => 'This note is locked';

  @override
  String get historyLockedMessage =>
      'Its earlier versions stay sealed until you unlock it.';

  @override
  String get historyNotFoundTitle => 'This note isn’t here any more';

  @override
  String get historyNotFoundMessage =>
      'It may have been deleted on another device.';

  @override
  String get historyVersionLabel => 'Version';

  @override
  String get historyVersionsTitle => 'Versions';

  @override
  String get historyNow => 'Now';

  @override
  String historyOrigin(String origin) {
    String _temp0 = intl.Intl.selectLogic(
      origin,
      {
        'ui': 'Edited in Kanso',
        'api': 'Replaced through the API',
        'sync': 'Replaced by sync',
        'import': 'Replaced by an import',
        'collab': 'Edited in a shared note',
        'agent': 'Suggested by an agent, applied by you',
        'plugin': 'Changed by a plugin',
        'other': 'Saved version',
      },
    );
    return '$_temp0';
  }

  @override
  String get historyPaneChanges => 'Changes';

  @override
  String get historyPaneVersion => 'This version';

  @override
  String get historyPaneLabel => 'What to show of the version';

  @override
  String get historyNoChanges =>
      'This version’s text is the same as the note’s now.';

  @override
  String historyHunkLine(int line) {
    return 'Line $line';
  }

  @override
  String historyHunkLines(int first, int last) {
    return 'Lines $first–$last';
  }

  @override
  String get historyHunkNew => 'New lines';

  @override
  String historyAdded(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count lines added',
      one: '1 line added',
    );
    return '$_temp0';
  }

  @override
  String historyRemoved(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count lines removed',
      one: '1 line removed',
    );
    return '$_temp0';
  }

  @override
  String get historyLineAdded => 'Added';

  @override
  String get historyLineRemoved => 'Removed';

  @override
  String get historyCopySection => 'Copy section';

  @override
  String get historyCopied => 'Copied';

  @override
  String get historyRestore => 'Restore this version';

  @override
  String get historyRestoreDone => 'This version was restored.';

  @override
  String get historyRestoring => 'Restoring…';

  @override
  String get historyDiffLoading => 'Working out what changed';

  @override
  String get historyProblemChanged =>
      'The note changed while its history was open. The history is shown again; choose the version once more.';

  @override
  String get historyProblemGone => 'That version isn’t kept any more.';

  @override
  String get historyProblemRestore => 'The version couldn’t be restored.';

  @override
  String get historyProblemDiff => 'What changed couldn’t be worked out.';

  @override
  String historyAuthors(String authors) {
    return '$authors';
  }

  @override
  String get libraryReplace => 'Replace in all notes';

  @override
  String get replaceTitle => 'Replace in all notes';

  @override
  String get replaceClose => 'Close';

  @override
  String get replaceFind => 'Find';

  @override
  String get replaceWith => 'Replace with';

  @override
  String get replaceMatchCase => 'Match case';

  @override
  String get replaceWholeWord => 'Whole word';

  @override
  String get replaceRegex =>
      'Regular expression (\$1 in the replacement for a group)';

  @override
  String get replaceSkipCode => 'Skip code blocks';

  @override
  String get replaceIncludeTrash => 'Include notes in the Trash';

  @override
  String get replaceOptionsLabel => 'Search options';

  @override
  String get replaceTypeToFind => 'Type what to find.';

  @override
  String get replaceNoMatches => 'No matches.';

  @override
  String get replaceSearching => 'Searching the notes';

  @override
  String replaceFound(int matches, int notes) {
    String _temp0 = intl.Intl.pluralLogic(
      matches,
      locale: localeName,
      other: '$matches matches',
      one: '1 match',
    );
    String _temp1 = intl.Intl.pluralLogic(
      notes,
      locale: localeName,
      other: '$notes notes',
      one: '1 note',
    );
    return '$_temp0 in $_temp1';
  }

  @override
  String get replacePreviewHeader => 'Preview';

  @override
  String get replaceInTrash => 'In Trash';

  @override
  String get replaceUntitled => 'Untitled';

  @override
  String replaceMore(int count) {
    return 'and $count more';
  }

  @override
  String replaceLine(int line) {
    return 'Line $line';
  }

  @override
  String replaceIncludeNote(String title) {
    return 'Include $title in the replace';
  }

  @override
  String replaceApply(int matches, int notes) {
    String _temp0 = intl.Intl.pluralLogic(
      matches,
      locale: localeName,
      other: 'Replace $matches',
      one: 'Replace 1',
    );
    String _temp1 = intl.Intl.pluralLogic(
      notes,
      locale: localeName,
      other: '$notes notes',
      one: '1 note',
    );
    return '$_temp0 in $_temp1';
  }

  @override
  String get replaceApplyIdle => 'Replace';

  @override
  String get replaceApplying => 'Replacing…';

  @override
  String replaceDone(int matches, int notes) {
    String _temp0 = intl.Intl.pluralLogic(
      matches,
      locale: localeName,
      other: '$matches matches',
      one: '1 match',
    );
    String _temp1 = intl.Intl.pluralLogic(
      notes,
      locale: localeName,
      other: '$notes notes',
      one: '1 note',
    );
    return 'Replaced $_temp0 in $_temp1.';
  }

  @override
  String replaceSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count notes changed elsewhere since the preview and were left alone.',
      one: '1 note changed elsewhere since the preview and was left alone.',
    );
    return '$_temp0';
  }

  @override
  String get replaceUndo => 'Undo replace';

  @override
  String get replaceUndoing => 'Putting back…';

  @override
  String replaceUndone(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return 'Put back $_temp0.';
  }

  @override
  String replaceUndoSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes were edited since and kept their changes.',
      one: '1 note was edited since and kept its changes.',
    );
    return '$_temp0';
  }

  @override
  String get replaceDismiss => 'Dismiss';

  @override
  String get replaceProblemBadPattern =>
      'The text to find can’t be used as a regular expression.';

  @override
  String get replaceProblemApply => 'The notes couldn’t be changed.';

  @override
  String get replaceProblemUndo => 'The notes couldn’t be put back.';

  @override
  String get replaceProblemSearch => 'The notes couldn’t be searched.';

  @override
  String get journalPreviousDay => 'Previous day';

  @override
  String get journalNextDay => 'Next day';

  @override
  String get journalCalendar => 'Calendar';

  @override
  String get journalTodaysNote => 'Today’s note';

  @override
  String get journalCalendarTitle => 'Journal';

  @override
  String get journalFailed => 'Couldn’t make that note. Try again.';

  @override
  String get calendarPreviousMonth => 'Previous month';

  @override
  String get calendarNextMonth => 'Next month';

  @override
  String get calendarToday => 'Today';

  @override
  String get calendarHint =>
      'A dot marks a day with a note. Tap a day to open it, or to start it from the daily template.';

  @override
  String get calendarFailed => 'The calendar couldn’t be read.';

  @override
  String calendarDayLabel(String date) {
    return '$date';
  }

  @override
  String calendarDayWithNote(String date) {
    return '$date, has a note';
  }

  @override
  String calendarDayToday(String date) {
    return '$date, today';
  }

  @override
  String calendarDayTodayWithNote(String date) {
    return '$date, today, has a note';
  }

  @override
  String get newMenuTitle => 'New';

  @override
  String get newMenuLabel => 'New';

  @override
  String get newFromTemplate => 'New from template…';

  @override
  String get newTemplate => 'New template';

  @override
  String get templatePickerTitle => 'New from template';

  @override
  String get templatesEmptyTitle => 'No templates yet';

  @override
  String get templatesEmptyMessage =>
      'Notes in the Templates notebook show here. New template makes one.';

  @override
  String get templatesFailed => 'The templates couldn’t be read.';

  @override
  String get settingsDailyNotes => 'Daily notes';

  @override
  String get settingsDailyTemplate => 'Daily template';

  @override
  String get dailyTemplateNone => 'None';

  @override
  String dailyTemplateDetail(String date, String time, String cursor) {
    return 'New daily notes start from this template, with $date, $time and $cursor filled in.';
  }

  @override
  String get dailyTemplateDetailEmpty =>
      'New daily notes start empty. Notes in the Templates notebook can be chosen here; New template makes one.';

  @override
  String get dailyTemplateFailed => 'Couldn’t set the daily template.';

  @override
  String get settingsReminders => 'Reminders';

  @override
  String get settingsRemindMe => 'Remind me of tasks';

  @override
  String get settingsRemindersDetail =>
      'A notification when a task with a time comes due, like “- [ ] Call Sam @2026-10-25T09:00”.';

  @override
  String get settingsRemindersDenied =>
      'Notifications are turned off for Kanso. Allow them in your device’s settings to get reminders.';

  @override
  String get settingsRemindersFailed => 'Couldn’t save that setting.';

  @override
  String get listOptionsBoard => 'Board';

  @override
  String get boardTitle => 'Board';

  @override
  String get boardNotes => 'Notes';

  @override
  String get boardTasks => 'Tasks';

  @override
  String get boardKindLabel => 'Board';

  @override
  String get boardClose => 'Close board';

  @override
  String get boardLaneNoStatus => 'No status';

  @override
  String get boardLaneTodo => 'To do';

  @override
  String get boardLaneDone => 'Done';

  @override
  String get boardEmptyNotesTitle => 'No notes here';

  @override
  String get boardEmptyNotesMessage =>
      'Notes in this list show here, a column for each status.';

  @override
  String get boardEmptyTasksTitle => 'No tasks in these notes';

  @override
  String get boardEmptyTasksMessage =>
      'Write “- [ ] Call Sam” in a note, and it shows here.';

  @override
  String get boardHintNotes =>
      'Touch and hold a card to move it to another status.';

  @override
  String get boardHintTasks =>
      'Tick a task to do it, or touch and hold it to move it.';

  @override
  String get boardHintNotesDrag => 'Drag a card to change its status.';

  @override
  String get boardHintTasksDrag => 'Drag a task to Done to tick it off.';

  @override
  String get boardMoveTo => 'Move to…';

  @override
  String get boardMoveSheetTitle => 'Move to';

  @override
  String get boardMoveFailed => 'Couldn’t move the card.';

  @override
  String get boardChangedInNote =>
      'That task changed in its note. The board is up to date now.';

  @override
  String get boardFailedTitle => 'The board didn’t load';

  @override
  String get boardFailedMessage =>
      'Something went wrong reading the notes on this device. Try again.';

  @override
  String boardTaskCount(int done, int total) {
    return '$done/$total tasks';
  }

  @override
  String boardColumnLabel(String lane, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count cards',
      one: '1 card',
    );
    return '$lane, $_temp0';
  }

  @override
  String get boardTickTask => 'Mark done';

  @override
  String get boardUntickTask => 'Mark not done';

  @override
  String boardTaskDue(String when) {
    return 'Due $when';
  }

  @override
  String get boardNoticeDismiss => 'OK';

  @override
  String get libraryStatuses => 'Status';

  @override
  String emptyStatusTitle(String status) {
    return 'No notes marked $status';
  }

  @override
  String get emptyStatusMessage =>
      'Notes with this status will appear in this list.';

  @override
  String get noteExportPdf => 'Export as PDF';

  @override
  String get noteExportPreparing => 'Preparing the PDF';

  @override
  String get noteExportHtml => 'Export as web page';

  @override
  String get notePresent => 'Present';

  @override
  String get presentationClose => 'End presentation';

  @override
  String get presentationEmpty => 'Nothing to present yet';

  @override
  String presentationPosition(int position, int count) {
    return '$position / $count';
  }

  @override
  String get noteExportPreparingPage => 'Preparing the web page';

  @override
  String get noteExportFailed => 'The note could not be exported.';

  @override
  String get libraryNewSubNotebook => 'New Sub-notebook';

  @override
  String get libraryRenameNotebook => 'Rename…';

  @override
  String get libraryChangeNotebookIcon => 'Change Icon…';

  @override
  String get libraryChangeNotebookColor => 'Change Color…';

  @override
  String get libraryDeleteNotebook => 'Delete Notebook…';

  @override
  String libraryDeleteNotebookTitle(Object name) {
    return 'Delete “$name”?';
  }

  @override
  String get libraryDeleteNotebookMessage =>
      'Its notes and the notes in its sub-notebooks move to the trash.';

  @override
  String get libraryRenameNotebookTitle => 'Rename Notebook';

  @override
  String get libraryNotebookNameHint => 'Name';

  @override
  String get librarySaveAction => 'Save';

  @override
  String get libraryDeleteAction => 'Delete';

  @override
  String get libraryChangeIconTitle => 'Change Icon';

  @override
  String get libraryIconSearchHint => 'Search icons';

  @override
  String get libraryIconDefault => 'Default';

  @override
  String get libraryChangeColorTitle => 'Change Color';

  @override
  String get libraryColorHexHint => 'Hex, such as #2185d0';

  @override
  String get libraryColorDefault => 'No color';

  @override
  String get libraryColorInvalid => 'That isn’t a color.';

  @override
  String get libraryNewTag => 'New Tag';

  @override
  String get libraryCreateTag => 'Create';

  @override
  String get libraryTagNameHint => 'Name';

  @override
  String get libraryRenameTagTitle => 'Rename Tag';

  @override
  String get libraryRenameTag => 'Rename…';

  @override
  String get libraryTagColor => 'Color';

  @override
  String get libraryDeleteTag => 'Delete Tag…';

  @override
  String libraryDeleteTagTitle(Object name) {
    return 'Delete the tag “$name”?';
  }

  @override
  String get libraryDeleteTagMessage =>
      'Notes keep their content; the tag is removed from them.';

  @override
  String get libraryEmptyTrash => 'Empty Trash…';

  @override
  String libraryEmptyTrashTitle(num count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return 'Delete $_temp0 in the trash permanently?';
  }

  @override
  String libraryEmptyTrashLockedTitle(int count, int locked) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    String _temp1 = intl.Intl.pluralLogic(
      locked,
      locale: localeName,
      other: '$locked locked notes and their sealed attachments',
      one: '1 locked note and its sealed attachments',
    );
    return 'Delete $_temp0 permanently, including $_temp1?';
  }

  @override
  String get libraryEmptyTrashMessage => 'They can’t be restored afterwards.';

  @override
  String get libraryEmptyTrashConfirm => 'Empty Trash';

  @override
  String get libraryActionFailed => 'That didn’t work. Try again.';

  @override
  String get noteActionPin => 'Pin Note';

  @override
  String get noteActionUnpin => 'Unpin Note';

  @override
  String get noteActionDuplicate => 'Duplicate';

  @override
  String get noteActionCopyLink => 'Copy Note Link';

  @override
  String get noteActionCopyKansoLink => 'Copy Kanso Link';

  @override
  String get noteActionStatus => 'Status';

  @override
  String get noteActionMoveToTrash => 'Move to Trash';

  @override
  String get noteActionRestore => 'Restore';

  @override
  String get noteActionDeleteForever => 'Delete Forever';

  @override
  String get noteActionDeleteForeverTitle => 'Delete this note forever?';

  @override
  String get noteActionDeleteForeverMessage =>
      'It can’t be restored afterwards.';

  @override
  String get noteActionMore => 'More';

  @override
  String get noteActionLinkCopied => 'Link copied.';

  @override
  String get libraryTagsMore => 'More';

  @override
  String get libraryTagsFewer => 'Fewer';

  @override
  String get tabsBarLabel => 'Tabs';

  @override
  String get tabClose => 'Close';

  @override
  String get tabCloseOthers => 'Close Others';

  @override
  String get tabCloseRight => 'Close Tabs to the Right';

  @override
  String get tabOpenInNewWindow => 'Open in New Window';

  @override
  String get tabOpenInNewTab => 'Open in New Tab';

  @override
  String get tabCloseTooltip => 'Close tab';

  @override
  String get tabsAll => 'All tabs';

  @override
  String tabLocked(String title) {
    return '$title, locked';
  }

  @override
  String get publishNoteMenu => 'Publish…';

  @override
  String get publishNotebookMenu => 'Publish notebook…';

  @override
  String get publishNoteTitle => 'Publish note';

  @override
  String get publishNotebookTitle => 'Publish notebook as a site';

  @override
  String publishNoteContentSummary(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count words',
      one: '$count word',
      zero: 'No words',
    );
    return '$_temp0';
  }

  @override
  String get publishExpiryHeading => 'Link expiry';

  @override
  String get publishWorking => 'Publishing…';

  @override
  String get publishPublish => 'Publish';

  @override
  String get publishUpdate => 'Update published note';

  @override
  String get publishPublishSite => 'Publish site';

  @override
  String get publishUpdateSite => 'Update site';

  @override
  String get publishCopyLink => 'Copy link';

  @override
  String get publishUnpublish => 'Unpublish';

  @override
  String get publishCopied => 'Publish link copied.';

  @override
  String get publishNeverExpires => 'This link does not expire.';

  @override
  String publishExpiresOn(String date) {
    return 'Expires $date';
  }

  @override
  String get publishExpired =>
      'This link has expired. Update to create a new link.';

  @override
  String get paletteHint => 'Search or run a command';

  @override
  String get paletteEmptyTitle => 'Nothing matches';

  @override
  String get paletteEmptyMessage => 'Try a shorter word, or another name.';

  @override
  String get paletteAddTag => 'Add tag';

  @override
  String get noteOutline => 'Outline';

  @override
  String get noteOutlineEmptyTitle => 'No headings';

  @override
  String get noteOutlineEmptyMessage => 'Headings in the note show up here.';

  @override
  String get noteBacklinks => 'Backlinks';

  @override
  String get noteBacklinksLinked => 'Linked from';

  @override
  String get noteBacklinksUnlinked => 'Unlinked mentions';

  @override
  String get noteBacklinksEmptyTitle => 'Nothing links here';

  @override
  String get noteBacklinksEmptyMessage =>
      'A note that links to this one shows up here.';

  @override
  String get paletteCommands => 'Commands';

  @override
  String get focusModeExit => 'Leave focus mode';

  @override
  String get editorWritingSection => 'Writing';

  @override
  String get editorDefaultWordGoal => 'Default word goal';

  @override
  String get editorDefaultWordGoalDetail =>
      'Notes without their own goal use this. A note’s progress appears in its editor footer.';

  @override
  String get editorFocusLivePreview => 'Live Preview in focus mode';

  @override
  String get editorFocusLivePreviewDetail =>
      'Focus mode shows the note in Live, and leaving it goes back to the mode you were in.';

  @override
  String get editorWordGoalNone => 'None';

  @override
  String get editorWordGoalLoading => 'Loading…';

  @override
  String get editorWordGoalUnavailable => 'Unavailable';

  @override
  String editorWordGoalWords(String words) {
    return '$words words';
  }

  @override
  String editorWordGoalAim(String words) {
    return 'Aim for $words';
  }

  @override
  String editorWordGoalProgress(String current, String goal) {
    return '$current of $goal words';
  }

  @override
  String get editorWordGoalSet => 'Set word goal';

  @override
  String get editorWordGoalForNote => 'Word goal for this note';

  @override
  String editorWordGoalDefault(String words) {
    return 'Default ($words words)';
  }

  @override
  String get editorWordGoalNoGoalForNote => 'No goal for this note';

  @override
  String get editorWordGoalCustom => 'Custom…';

  @override
  String get editorWordGoalCustomTitle => 'Custom word goal';

  @override
  String get editorWordGoalCustomHint => 'Words, such as 1,500 or 1.5k';

  @override
  String get editorWordGoalInvalid => 'Enter a word goal from 1 to 1,000,000.';

  @override
  String get editorWordGoalCancel => 'Cancel';

  @override
  String get editorWordGoalSave => 'Set goal';

  @override
  String get settingsAi => 'AI';

  @override
  String get settingsAiOn => 'On';

  @override
  String get settingsAiOff => 'Off';

  @override
  String get aiTitle => 'AI';

  @override
  String get aiEnabled => 'Use AI';

  @override
  String get aiEnabledFooter =>
      'Ask your notes, rewrite a selection and summarize a note. AI is off until you turn it on.';

  @override
  String get aiProviderSection => 'Answers come from';

  @override
  String get aiProvider => 'Provider';

  @override
  String get aiProviderSheetTitle => 'Choose a provider';

  @override
  String get aiAddress => 'Address';

  @override
  String get aiModel => 'Model';

  @override
  String get aiRegion => 'Region';

  @override
  String get aiApiKey => 'API key';

  @override
  String get aiApiKeyOptional => 'API key (optional)';

  @override
  String get aiKeyHint => 'Paste the key';

  @override
  String get aiKeySave => 'Save key';

  @override
  String get aiKeySaved => 'Saved';

  @override
  String get aiKeyRemove => 'Remove';

  @override
  String get aiKeyFooter =>
      'Kept in secure storage on this device and never shown again.';

  @override
  String get aiTest => 'Test';

  @override
  String get aiTestHint =>
      'Sends one short question to check the address, model and key. No notes are sent.';

  @override
  String get aiTestRunning => 'Asking the model…';

  @override
  String aiTestOk(String said) {
    return 'Ready. The model said “$said”.';
  }

  @override
  String get aiLoadFailedTitle => 'Couldn’t open AI settings';

  @override
  String get aiLoadFailedMessage =>
      'Something went wrong reading them. Try again.';

  @override
  String get aiSaveFailed => 'Couldn’t save that. Try again.';

  @override
  String get askTitle => 'Ask your notes';

  @override
  String get askClear => 'Clear conversation';

  @override
  String get askHint => 'Ask about your notes…';

  @override
  String get askSend => 'Ask';

  @override
  String get askStop => 'Stop';

  @override
  String get askEmptyMessage =>
      'A question is answered from your own notes, with links to the notes it used. The answer isn’t saved and no note is changed.';

  @override
  String get askSearching => 'Searching your notes…';

  @override
  String get askThinking => 'Thinking…';

  @override
  String get askNothingMatches =>
      'Nothing in your notes matches this question.';

  @override
  String get askCutOff => 'The answer was cut off before it finished.';

  @override
  String get askStopped => 'Stopped.';

  @override
  String get askFailedTitle => 'Couldn’t answer';

  @override
  String get askFailedMessage => 'Something went wrong. Try again.';

  @override
  String askUsedNotes(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Used $count notes',
      one: 'Used 1 note',
    );
    return '$_temp0';
  }

  @override
  String get askUntitled => 'Untitled';

  @override
  String get askOffTitle => 'AI is off';

  @override
  String get askOffMessage =>
      'Turn on AI in Settings to ask questions of your notes.';

  @override
  String get askOpenSettings => 'Open AI settings';

  @override
  String get assistTitle => 'Ask AI';

  @override
  String get assistSelectionAction => 'Ask AI…';

  @override
  String get assistAccessoryLabel => 'AI assistant';

  @override
  String get assistSummary => 'Summary';

  @override
  String get assistSummarizeNote => 'Summarize this note';

  @override
  String get assistThinking => 'Thinking…';

  @override
  String get assistStop => 'Stop';

  @override
  String get assistAccept => 'Accept';

  @override
  String get assistNothing => 'Nothing came back.';

  @override
  String get assistDiscard => 'Discard';

  @override
  String get assistCopy => 'Copy';

  @override
  String get assistCopied => 'Copied';

  @override
  String get assistClose => 'Close';

  @override
  String get assistKeptOut => 'This note is kept out of AI.';

  @override
  String get assistSelectText => 'Select some text first.';

  @override
  String get assistContinueAfterText =>
      'Put the cursor after some text to continue it.';

  @override
  String get assistReadOnly => 'This note can’t be edited.';

  @override
  String get assistStopped => 'Stopped. A partial rewrite can’t be applied.';

  @override
  String get assistFailed => 'The suggestion couldn’t be made. Try again.';

  @override
  String get assistChanged =>
      'The text changed, so the suggestion wasn’t applied.';

  @override
  String get libraryAsk => 'Ask your notes';

  @override
  String get dictationKey => 'Dictate';

  @override
  String get dictationKeyStop => 'Stop dictating';

  @override
  String get dictationListening => 'Listening';

  @override
  String get dictationStop => 'Stop';

  @override
  String get dictationVoiceNoteTitle => 'Voice note';

  @override
  String get dictationVoiceNoteHint => 'Tap the mic and start talking.';

  @override
  String get dictationTranscriptEmpty => 'What you say will appear here.';

  @override
  String get dictationMicStart => 'Start listening';

  @override
  String get dictationMicStop => 'Stop listening';

  @override
  String get dictationDone => 'Done';

  @override
  String get dictationCancel => 'Cancel';

  @override
  String get dictationSaving => 'Saving the note';

  @override
  String get dictationSaveFailedTitle => 'The note wasn\'t saved';

  @override
  String get dictationSaveFailedMessage =>
      'Your words are still here. Try again.';

  @override
  String get dictationTryAgain => 'Try again';

  @override
  String get dictationUnavailableTitle => 'Dictation isn\'t available';

  @override
  String get dictationUnavailableNotOnDevice =>
      'This phone can\'t turn speech into text by itself in this language, and Kanso never sends your voice anywhere else.';

  @override
  String get dictationUnavailableNoRecognizer =>
      'This phone has no speech recognizer. Install or turn on one in the phone\'s settings.';

  @override
  String get dictationUnavailableLanguage =>
      'Speech recognition doesn\'t support this language.';

  @override
  String get dictationDeniedTitle => 'Kanso can\'t hear you';

  @override
  String get dictationDeniedMessage =>
      'Allow the microphone and speech recognition for Kanso in Settings to dictate.';

  @override
  String get dictationRestrictedMessage =>
      'This phone\'s rules keep Kanso from using the microphone or speech recognition.';

  @override
  String get sheetDismiss => 'Dismiss sheet';

  @override
  String get dictationOpenSettings => 'Open Settings';

  @override
  String get dictationErrorTitle => 'Dictation stopped';

  @override
  String get dictationErrorMessage =>
      'Something interrupted the microphone. Try again.';

  @override
  String get dictationNoSpeech =>
      'Nothing was heard. Tap the mic and try again.';

  @override
  String get dictationDismiss => 'OK';

  @override
  String get dictationSettingsTitle => 'Voice';

  @override
  String get dictationSettingsInputSection => 'Voice input';

  @override
  String get dictationSettingsInput => 'Voice input';

  @override
  String get dictationSettingsInputDetail =>
      'Tap the mic above the keyboard, speak, and tap it again: what you said goes in at the cursor. A voice note makes a new note from what you say.';

  @override
  String get dictationSettingsCommands => 'Spoken punctuation and commands';

  @override
  String get dictationSettingsCommandsDetail =>
      'Say \"comma\", \"full stop\", \"new line\", \"new paragraph\", \"bullet point\" or \"new to-do\" on their own, with a short pause either side, and they\'re written. \"Delete that\" takes out the last thing you said, and \"undo that\" the last step.';

  @override
  String get dictationSettingsRecognitionSection => 'Recognition';

  @override
  String dictationRecognizerReady(String language) {
    return 'On-device recognition is available for $language';
  }

  @override
  String dictationRecognizerSystem(String language) {
    return '$language is recognized by the phone\'s speech service, which may use the network.';
  }

  @override
  String dictationRecognizerUnavailable(String language) {
    return 'On-device recognition isn\'t available for $language';
  }

  @override
  String get dictationSettingsMicDenied => 'Kanso can\'t use the microphone';

  @override
  String triageBarCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes to file',
      one: '1 note to file',
    );
    return '$_temp0';
  }

  @override
  String get triageBarOpen => 'Triage';

  @override
  String get triageTitle => 'Triage';

  @override
  String triageSubtitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count to file',
      one: '1 to file',
    );
    return '$_temp0';
  }

  @override
  String get triageEmpty => 'Your Inbox is clear';

  @override
  String get triageClose => 'Close';

  @override
  String get triageMenu => 'Triage options';

  @override
  String get triageAcceptAllNotebooks => 'Accept all suggested notebooks';

  @override
  String get triageAcceptAllTags => 'Accept all suggested tags';

  @override
  String get triageFieldNotebook => 'Notebook';

  @override
  String get triageFieldTags => 'Tags';

  @override
  String get triageFieldTitle => 'Title';

  @override
  String get triageFieldLinks => 'Links';

  @override
  String get triageAcceptAll => 'Accept all';

  @override
  String triageFileAllAccepted(int count) {
    return 'File all accepted ($count)';
  }

  @override
  String get triageFile => 'File';

  @override
  String get triageSkip => 'Skip';

  @override
  String get triageOpen => 'Open';

  @override
  String triageAcceptChip(String label) {
    return 'Accept $label';
  }

  @override
  String triageRejectChip(String label) {
    return 'Reject $label';
  }

  @override
  String triageEditChip(String label) {
    return 'Edit $label';
  }

  @override
  String triageBecause(String titles) {
    return 'because: near $titles';
  }

  @override
  String get triageNewTag => 'new';

  @override
  String get triageNoSuggestions => 'Nothing to suggest for this note.';

  @override
  String get triageNotRead =>
      'Notebook and tag suggestions come once this note is read.';

  @override
  String get triageNoAi =>
      'Suggestions come from your notes. Set up AI in Settings for better titles.';

  @override
  String get triageRelatedOff =>
      'Turn on Related notes to get notebook and tag suggestions';

  @override
  String get triageTurnOn => 'Turn on';

  @override
  String triageAiUnavailable(String reason) {
    return '$reason Showing suggestions from your notes.';
  }

  @override
  String get triageChanged => 'Changed on another device';

  @override
  String triageConfirmTitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'File $count notes?',
      one: 'File 1 note?',
    );
    return '$_temp0';
  }

  @override
  String get triageEditHint => 'Type a value';

  @override
  String get triageEditUse => 'Use';

  @override
  String get triageSettingsTitle => 'Inbox';

  @override
  String get triageSettingsShowBar => 'Show Triage bar';

  @override
  String get triageSettingsShowBarHelp =>
      'Above the Inbox\'s notes: how many wait to be filed, and a button to go through them.';

  @override
  String get triageSettingsTitles => 'Suggest titles';

  @override
  String get triageSettingsTitlesHelp =>
      'Offers the first sentence as the title when a note\'s own is empty, a date, “Untitled” or cut short.';

  @override
  String get triageSettingsLinks => 'Suggest links';

  @override
  String get triageSettingsLinksHelp =>
      'Offers links to notes the text names, and to near notes, as a Related line at the end of the note.';

  @override
  String get triageSettingsMaxTags => 'Maximum tags per note';

  @override
  String get triageSettingsUseAi => 'Use AI for Inbox suggestions';

  @override
  String get triageSettingsUseAiHelp =>
      'Sends the note and the names of likely notebooks and tags to your AI provider.';

  @override
  String get triageSettingsOpen => 'Inbox';

  @override
  String get digestLoopsTitle => 'Open loops';

  @override
  String digestLoopsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count open loops',
      one: '1 open loop',
    );
    return '$_temp0';
  }

  @override
  String get digestLoopsEmptyTitle => 'No open loops';

  @override
  String get digestLoopsEmptyMessage =>
      'When you write things like “I\'ll send the deck”, “waiting on Rudo” or “need to check the lease”, Kanso notices and lists them here, so nothing slips.';

  @override
  String get digestLoopsOffTitle => 'Open loops are off';

  @override
  String get digestLoopsOffMessage =>
      'Turn them on in Settings, under Brief and review.';

  @override
  String get digestKindMine => 'I will';

  @override
  String get digestKindWaiting => 'Waiting on';

  @override
  String get digestKindQuestion => 'To find out';

  @override
  String get digestLoopDone => 'Done';

  @override
  String get digestLoopNotALoop => 'Not a loop';

  @override
  String get digestLoopMakeTask => 'Make task';

  @override
  String get digestLoopOpenNote => 'Open note';

  @override
  String get digestLoopFromAi => 'Found by AI';

  @override
  String get digestLoopGone => 'That sentence is not in the note any more.';

  @override
  String get digestLoopsRefresh => 'Refresh';

  @override
  String get digestBriefTitle => 'Morning brief';

  @override
  String get digestBriefEmptyTitle => 'You\'re all clear';

  @override
  String get digestBriefEmptyMessage =>
      'Each morning this page gathers what\'s due, the open loops in your notes, today\'s reminders and notes worth another look. Nothing needs you right now.';

  @override
  String get digestBriefDue => 'Due';

  @override
  String get digestBriefLater => 'Later today';

  @override
  String get digestBriefLoops => 'Open loops';

  @override
  String digestBriefMoreLoops(int count) {
    return 'See all $count open loops';
  }

  @override
  String get digestBriefRelevant => 'From your notes';

  @override
  String digestBriefYesterday(int wrote, int finished) {
    String _temp0 = intl.Intl.pluralLogic(
      wrote,
      locale: localeName,
      other: '$wrote notes written',
      one: '1 note written',
    );
    return 'Yesterday: $_temp0, $finished finished';
  }

  @override
  String get digestReviewTitle => 'Weekly review';

  @override
  String digestReviewWeek(String week) {
    return 'Week $week';
  }

  @override
  String get digestReviewPrevious => 'Previous week';

  @override
  String get digestReviewNext => 'Next week';

  @override
  String get digestReviewWritten => 'Written';

  @override
  String get digestReviewFinished => 'Finished';

  @override
  String get digestReviewStillOpen => 'Still open';

  @override
  String get digestReviewOverdue => 'Overdue';

  @override
  String get digestReviewThemes => 'Themes';

  @override
  String digestReviewThemeNotes(String term, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$term · $_temp0';
  }

  @override
  String get digestReviewNextWeek => 'Next up';

  @override
  String get digestReviewNothing => 'Nothing to show for this week.';

  @override
  String get digestReviewSave => 'Save as note';

  @override
  String get digestReviewSaved => 'Saved to the Journal';

  @override
  String get digestReviewOpenSaved => 'Open saved note';

  @override
  String get digestReviewReplaceTitle => 'Replace the saved review?';

  @override
  String get digestReviewReplaceMessage =>
      'This week\'s review is already saved in the Journal. Replacing it writes the current review over that note.';

  @override
  String get digestReviewReplace => 'Replace';

  @override
  String get digestReviewError => 'Couldn\'t read the review. Try again.';

  @override
  String get digestError => 'Couldn\'t read this. Try again.';

  @override
  String get digestRetry => 'Try again';

  @override
  String get settingsBrief => 'Brief and review';

  @override
  String get settingsBriefLoops => 'Find open loops';

  @override
  String get settingsBriefMorning => 'Morning brief';

  @override
  String get settingsBriefMorningWeekdays => 'Weekdays only';

  @override
  String get settingsBriefReview => 'Weekly review';

  @override
  String get settingsBriefDay => 'Review day';

  @override
  String get settingsBriefTime => 'Time';

  @override
  String get settingsBriefMarkDone => 'Mark done loops in the note';

  @override
  String get settingsBriefDetail =>
      'Finds things your notes say you will do, wait for or check, like “I\'ll send the deck”. Nothing leaves this phone. The notifications say only how much is due and open.';

  @override
  String get settingsBriefMarkDoneDetail =>
      'Adds a tick after a sentence when you mark its loop done. Off, Kanso remembers it without touching your text.';

  @override
  String get settingsBriefEarlier => '15 minutes earlier';

  @override
  String get settingsBriefLater => '15 minutes later';

  @override
  String get briefNotificationOpen => 'Open';

  @override
  String get weekdayMonday => 'Monday';

  @override
  String get weekdayTuesday => 'Tuesday';

  @override
  String get weekdayWednesday => 'Wednesday';

  @override
  String get weekdayThursday => 'Thursday';

  @override
  String get weekdayFriday => 'Friday';

  @override
  String get weekdaySaturday => 'Saturday';

  @override
  String get weekdaySunday => 'Sunday';

  @override
  String get digestFindLoops => 'Find more open loops…';

  @override
  String get digestFindTitle => 'Find more open loops?';

  @override
  String digestFindMessage(int days, String host) {
    return 'Read your notes from the last $days days with the model at $host to find promises the rules missed? Locked notes and notes tagged #ai-exclude are left out.';
  }

  @override
  String get digestFindConfirm => 'Find loops';

  @override
  String get digestFinding => 'Reading your notes…';

  @override
  String get digestFoundTitle => 'Open loops found';

  @override
  String get digestFoundNone => 'No more open loops found.';

  @override
  String get digestFoundKeep => 'Keep selected';

  @override
  String digestFoundKept(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Kept $count open loops.',
      one: 'Kept 1 open loop.',
    );
    return '$_temp0';
  }

  @override
  String get digestSummarise => 'Summarise';

  @override
  String get digestSummarising => 'Writing…';

  @override
  String get digestSummaryLabel => 'Summary by AI';

  @override
  String get digestStop => 'Stop';

  @override
  String get digestMorningBrief => 'Your morning brief';

  @override
  String digestMorningBriefCounts(int due, int loops) {
    String _temp0 = intl.Intl.pluralLogic(
      due,
      locale: localeName,
      other: '$due due',
      one: '1 due',
      zero: 'Nothing due',
    );
    String _temp1 = intl.Intl.pluralLogic(
      loops,
      locale: localeName,
      other: '$loops open loops',
      one: '1 open loop',
      zero: 'no open loops',
    );
    return '$_temp0, $_temp1';
  }

  @override
  String get triageEmptyMessage =>
      'Triage files the notes that land in your Inbox. For each one it suggests a notebook, tags, a title and links, and nothing changes until you accept. New captures will wait here.';

  @override
  String get digestReviewEmptyTitle => 'A quiet week';

  @override
  String get digestReviewEmptyMessage =>
      'The weekly review gathers the notes you wrote, the tasks you finished, the loops still open and what\'s due next. Nothing happened this week yet: write something or tick a task off and it shows up here.';

  @override
  String get aiTroubleKey =>
      'Your model turned the key down. The API key was refused. Check it in Settings › AI, then try again.';

  @override
  String get aiTroubleBusy =>
      'Your model is busy. It\'s limiting requests right now. Wait a moment, then try again.';

  @override
  String get aiTroubleSlow =>
      'Your model took too long. It didn\'t answer in time. Try again, or pick a smaller model in Settings › AI.';

  @override
  String get aiTroubleMissing =>
      'Your model wasn\'t found. The model named in Settings › AI isn\'t available. Choose another, then try again.';

  @override
  String aiTroubleReachLocal(String app) {
    return 'Couldn\'t reach your model. Is $app running? Start it, then try again.';
  }

  @override
  String get aiTroubleReach =>
      'Couldn\'t reach your model. Check your connection and the address in Settings › AI, then try again.';

  @override
  String get aiTroubleSetup =>
      'AI isn\'t set up yet. Choose a model in Settings › AI to use this.';

  @override
  String get aiTroubleOther =>
      'Your model ran into a problem. Try again. If it keeps happening, check Settings › AI.';

  @override
  String get aiTroubleMissingKey =>
      'No API key is saved. Add one in Settings › AI: choose your provider, then paste the key into API key.';

  @override
  String get aiTroubleMissingModel =>
      'No model is chosen. Pick a model in Settings › AI, then try again.';

  @override
  String get aiTroubleMissingUrl =>
      'No server address is set. Enter the server\'s address in Settings › AI, then try again.';

  @override
  String get aiTroubleNoAgent =>
      'No agent is set up. Choose an agent in Settings › AI, or type the command of your own. Then try again.';

  @override
  String get aiTroubleAgentNotInstalled =>
      'Your agent isn\'t installed. Install it, or choose another agent in Settings › AI and use Find to check again.';

  @override
  String get aiTroubleAgentNeedsNode =>
      'Your agent needs a newer Node. Install the Node version it asks for, then use Find again in Settings › AI.';

  @override
  String get aiTroubleNoTools =>
      'This model can\'t use tools. Pick a model that supports tools in Settings › AI.';

  @override
  String get aiTroubleServerDown =>
      'Your model\'s server had a problem. That\'s on its side. Wait a moment, then try again.';

  @override
  String get aiTroubleBlocked =>
      'Something blocked the request. A proxy, firewall or region limit turned it away before it reached your model, so it isn\'t your API key. Check your network and the address in Settings › AI, then try again.';

  @override
  String get aiTroubleReachLocalModel =>
      'Couldn\'t reach your model. Is Ollama or LM Studio running? Start it, then try again.';

  @override
  String get aiTroubleRetry => 'Try again';

  @override
  String get aiTroubleSettings => 'AI settings';

  @override
  String get agentTasksTitle => 'Agent tasks';

  @override
  String get agentTasksEntry => 'Agent task';

  @override
  String get noteListAskAgent => 'Ask an agent';

  @override
  String get agentTasksGoalLabel => 'What should the AI do?';

  @override
  String get agentTasksGoalHint => 'Merge my Lisbon trip notes into one plan';

  @override
  String get agentTasksStart => 'Start';

  @override
  String agentTasksChosenNotes(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count chosen notes',
      one: '1 chosen note',
    );
    return 'Working on $_temp0';
  }

  @override
  String get agentTasksEmptyTitle => 'No agent tasks yet';

  @override
  String get agentTasksEmptyMessage =>
      'Give the AI a goal. It reads your notes and proposes changes. Nothing is written until you review and apply them.';

  @override
  String get agentTaskWorking => 'Working…';

  @override
  String get agentTaskWaiting => 'Waiting its turn';

  @override
  String get agentTaskNothing => 'The AI found nothing to change.';

  @override
  String agentTaskReady(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count changes to review',
      one: '1 change to review',
    );
    return '$_temp0';
  }

  @override
  String agentTaskApplied(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Applied $count changes',
      one: 'Applied 1 change',
    );
    return '$_temp0';
  }

  @override
  String get agentTaskUndone => 'Undone';

  @override
  String agentTaskNotesRead(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes read',
      one: '1 note read',
    );
    return '$_temp0';
  }

  @override
  String get agentTaskStop => 'Stop';

  @override
  String get agentTaskReview => 'Review';

  @override
  String get agentTaskAnswer => 'Answer';

  @override
  String get agentTaskSend => 'Send';

  @override
  String get agentTaskDismiss => 'Dismiss';

  @override
  String get agentTroubleTitle => 'The AI couldn\'t do that';

  @override
  String get agentTroubleOff =>
      'Agent tasks are turned off. You can turn them on in Settings.';

  @override
  String get agentTroubleAiOff =>
      'AI is turned off. You can turn it on in Settings.';

  @override
  String get agentTroubleGeneric =>
      'Something went wrong reaching the AI. Check your connection and the AI settings, then try again.';

  @override
  String get agentTroubleTryAgain => 'Try again';

  @override
  String get agentTroubleSettings => 'AI settings';

  @override
  String get agentReviewTitle => 'Review changes';

  @override
  String agentReviewApply(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Apply $count changes',
      one: 'Apply 1 change',
      zero: 'Apply',
    );
    return '$_temp0';
  }

  @override
  String agentReviewPosition(int number, int total) {
    return '$number of $total';
  }

  @override
  String get agentReviewKeep => 'Keep the original notes';

  @override
  String get agentReviewTrashed => 'The original notes move to the Trash.';

  @override
  String get agentReviewKept => 'The original notes stay where they are.';

  @override
  String get agentReviewChangedSince =>
      'This note changed since the AI read it. Ticking it applies the change over yours.';

  @override
  String get agentReviewOutside => 'Not one of the notes you chose';

  @override
  String get agentReviewPublished => 'This note is published';

  @override
  String get agentReviewKindCreate => 'New note';

  @override
  String get agentReviewKindEdit => 'Edit';

  @override
  String get agentReviewKindMove => 'Move';

  @override
  String get agentReviewKindMerge => 'Merge';

  @override
  String get agentReviewFromAgent => 'From the AI';

  @override
  String get agentReviewNothing => 'The AI didn\'t propose any changes.';

  @override
  String get agentReviewUndo => 'Undo';

  @override
  String get agentSettingsTitle => 'Agent tasks';

  @override
  String get agentSettingsEnabled => 'Allow agent tasks';

  @override
  String get agentSettingsMayPropose => 'The AI may propose';

  @override
  String get agentSettingsNewNotes => 'New notes';

  @override
  String get agentSettingsEdits => 'Edits to notes';

  @override
  String get agentSettingsMoves => 'Moves to another notebook';

  @override
  String get agentSettingsMerges => 'Merging notes';

  @override
  String get agentSettingsTimeLimit => 'Time limit';

  @override
  String agentSettingsMinutes(int minutes) {
    return '$minutes min';
  }

  @override
  String get agentSettingsMaxNotes => 'Notes it may read';

  @override
  String get agentSettingsAskAll => 'Ask before reading all notebooks';

  @override
  String get agentTasksNewNotesIn => 'New notes go in';

  @override
  String get agentTasksInbox => 'Inbox';

  @override
  String get agentTasksReadAllTitle => 'Let the AI read all your notebooks?';

  @override
  String get agentTasksReadAllBody =>
      'No notes were chosen, so this task may read every notebook that is not locked. Locked notes and notes tagged #ai-exclude stay out of reach.';

  @override
  String get agentTasksReadAllConfirm => 'Read all notebooks';

  @override
  String get agentTasksReadAllCancel => 'Cancel';

  @override
  String get agentSettingsFooter =>
      'An agent reads only notes that are not locked, not in the Trash and not tagged #ai-exclude. It never writes; you approve every change.';

  @override
  String get voiceTidy => 'Tidy';

  @override
  String get voiceTidyOffer =>
      'Tidy this voice note: a title, short paragraphs and tasks. The transcript stays.';

  @override
  String get voiceTidyWorking => 'Tidying…';

  @override
  String voiceTidyFailed(String reason) {
    return 'Couldn\'t tidy this note: $reason.';
  }

  @override
  String get recordingSummaryMenu => 'Summarise recording';

  @override
  String get recordingSummaryTitle => 'Summarise recording';

  @override
  String get recordingSummaryClose => 'Close';

  @override
  String get recordingSummaryUnavailable =>
      'To summarise a recording, turn on AI in Settings and open a note that holds the transcript of a recording. Locked notes, notes in the trash and notes tagged #ai-exclude are never sent.';

  @override
  String get recordingSummaryNothingYet =>
      'Nothing is sent until you choose Send. You will see the summary and the action items before anything is written.';

  @override
  String get recordingSummaryReplacesNote =>
      'This note already has a summary. A new one replaces it.';

  @override
  String get recordingSummaryWorking => 'Summarising the transcript…';

  @override
  String get recordingSummaryCancelHint =>
      'Cancel stops the request. The note is not changed.';

  @override
  String get recordingSummaryUnchanged => 'The note is not changed.';

  @override
  String get recordingSummarySend => 'Send';

  @override
  String get recordingSummaryCancel => 'Cancel';

  @override
  String get recordingSummaryDiscard => 'Discard';

  @override
  String get recordingSummaryApply => 'Apply';

  @override
  String get recordingSummaryTryAgain => 'Try again';

  @override
  String get recordingSummarySectionSummary => 'Summary';

  @override
  String get recordingSummarySectionDecisions => 'Decisions';

  @override
  String get recordingSummarySectionActions => 'Action items';

  @override
  String get recordingSummaryActionsHint =>
      'Only the ticked ones are written into the note.';

  @override
  String get recordingSummaryCut =>
      'The transcript was long, so only its first part was summarised.';

  @override
  String get recordingSummaryAdded => 'Summary added above the transcript.';

  @override
  String get recordingSummaryReplaced => 'Summary replaced.';

  @override
  String recordingSummaryFailed(String reason) {
    return 'Couldn\'t summarise this recording. $reason';
  }

  @override
  String recordingSummaryApplyFailed(String reason) {
    return 'Nothing was written. $reason';
  }

  @override
  String recordingSummaryDropped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count items were left out for want of a quote from the recording.',
      one: 'One item was left out for want of a quote from the recording.',
    );
    return '$_temp0';
  }

  @override
  String get voiceTidyTryAgain => 'Try again';

  @override
  String get voiceSettingsCaptureSection => 'Voice notes';

  @override
  String get voiceSettingsTidy => 'Tidy voice notes';

  @override
  String get voiceSettingsTidyOff => 'Off';

  @override
  String get voiceSettingsTidyAsk => 'Ask each time';

  @override
  String get voiceSettingsTidyAlways => 'Always';

  @override
  String get voiceSettingsTidyHelp =>
      'Your own AI gives a voice note a title, short paragraphs and tasks for what was said you would do. Only the words are sent. The transcript stays in the note, folded under it.';

  @override
  String get voiceSettingsTidyNeedsAi =>
      'Set up AI in Settings › AI to tidy notes.';

  @override
  String get voiceSettingsMeetings => 'Meetings';

  @override
  String get voiceSettingsMeetingsHelp =>
      'Treat recordings longer than 3 minutes as meetings. Tidy gives them a Summary, Decisions, Action items and Notes, and their audio is kept.';

  @override
  String get voiceSettingsKeepAudio => 'Keep audio';

  @override
  String get voiceSettingsKeepAudioHelp =>
      'Attach the recording to the voice note. Audio over 10 MB isn\'t kept; the transcript always is.';

  @override
  String get voiceSettingsServer => 'Use the speech server when needed';

  @override
  String get voiceSettingsServerHelp =>
      'When this phone can\'t turn a recording into text by itself, it can send the audio to the system\'s speech service to be transcribed. Off, such a recording waits here, untouched.';

  @override
  String voiceSettingsWaiting(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count recordings are waiting to be transcribed',
      one: '1 recording is waiting to be transcribed',
    );
    return '$_temp0';
  }

  @override
  String get voiceTidyDone => 'Tidied this voice note';

  @override
  String get flashcardsReviewTitle => 'Review';

  @override
  String get flashcardsShowAnswer => 'Show Answer';

  @override
  String get flashcardsAgain => 'Again';

  @override
  String get flashcardsHard => 'Hard';

  @override
  String get flashcardsGood => 'Good';

  @override
  String get flashcardsEasy => 'Easy';

  @override
  String get flashcardsSwipeHint => 'Swipe left for Again, right for Good';

  @override
  String get flashcardsDoneForToday => 'Done for today.';

  @override
  String get flashcardsDoneBody =>
      'Come back when more cards are due. Cards are lines written as question :: answer in your notes.';

  @override
  String get flashcardsReviewMore => 'Review More';

  @override
  String get flashcardsNoCardsTitle => 'No cards yet';

  @override
  String get flashcardsNoCardsBody =>
      'Write a line like  What is FSRS? :: A scheduler  in any note. Each such line is a flashcard.';

  @override
  String get flashcardsStats => 'Stats';

  @override
  String get flashcardsSettings => 'Flashcards';

  @override
  String get flashcardsSuspend => 'Suspend Card';

  @override
  String get flashcardsSuspended => 'Card suspended.';

  @override
  String get flashcardsOpenNote => 'Open Note';

  @override
  String get flashcardsStatsTotal => 'Cards';

  @override
  String get flashcardsStatsDue => 'Due today';

  @override
  String get flashcardsStatsNew => 'New';

  @override
  String get flashcardsStatsSuspended => 'Suspended';

  @override
  String get flashcardsStatsPerDay => 'Reviews, last 30 days';

  @override
  String get flashcardsStatsRetention => 'Retention, last 30 days';

  @override
  String get flashcardsStatsForecast => 'Due, next 7 days';

  @override
  String get flashcardsStatsHardest => 'Hardest cards';

  @override
  String get flashcardsStatsNone => 'Nothing reviewed yet.';

  @override
  String get flashcardsNewPerDay => 'New cards a day';

  @override
  String get flashcardsReviewsPerDay => 'Reviews a day';

  @override
  String get flashcardsTargetRecall => 'Target recall';

  @override
  String get flashcardsDailyReminder => 'Daily reminder';

  @override
  String get flashcardsReminderOff => 'Off';

  @override
  String get flashcardsShowIcons => 'Show card icons in the editor';

  @override
  String get flashcardsResetHistory => 'Reset Review History…';

  @override
  String get flashcardsResetTitle => 'Reset review history?';

  @override
  String get flashcardsResetBody =>
      'Every card starts again as new, on all your devices. The cards in your notes stay.';

  @override
  String get flashcardsReset => 'Reset';

  @override
  String get flashcardsMakeCards => 'Make Flashcards…';

  @override
  String get flashcardsMakeBlockedExcluded => 'This note is excluded from AI.';

  @override
  String get flashcardsMakeReading => 'Reading the note…';

  @override
  String get flashcardsSourceNotFound => 'Source not found';

  @override
  String get flashcardsNoSuggestions => 'No cards came back. Try again.';

  @override
  String get flashcardsQuizMe => 'Quiz Me';

  @override
  String get flashcardsQuizHint => 'Type your answer';

  @override
  String get flashcardsQuizCheck => 'Check';

  @override
  String get flashcardsQuizNext => 'Next';

  @override
  String get flashcardsQuizSaveCard => 'Save as Card';

  @override
  String get flashcardsQuizDone => 'That was the round.';

  @override
  String get flashcardsQuizAgain => 'Another Round';

  @override
  String get flashcardsDraftFrom => 'Draft from This Note…';

  @override
  String get flashcardsDraftTitle => 'Draft';

  @override
  String get flashcardsDraftSources => 'Sources';

  @override
  String get flashcardsDraftWhat => 'What should it say, and for whom?';

  @override
  String get flashcardsDraftWrite => 'Write Draft';

  @override
  String get flashcardsDraftStop => 'Stop';

  @override
  String get flashcardsDraftTryAgain => 'Try Again';

  @override
  String get flashcardsDraftCopy => 'Copy';

  @override
  String get flashcardsDraftSave => 'Save as Note';

  @override
  String get flashcardsDraftDiscard => 'Discard';

  @override
  String get flashcardsDraftDiscardTitle => 'Discard this draft?';

  @override
  String get flashcardsDraftDiscardBody =>
      'It has not been saved. Drafts are not kept.';

  @override
  String get flashcardsDraftNoSource => 'No source in your notes';

  @override
  String get flashcardsDraftNone =>
      'None of these notes can be sent to AI. Locked notes and notes tagged #ai-exclude are never sent.';

  @override
  String get flashcardsDraftNoAi =>
      'Drafting needs an AI provider. Choose one in Settings › AI.';

  @override
  String get flashcardsDraftSaved => 'Saved the draft as a note.';

  @override
  String get flashcardsDraftReviewTitle => 'Review';

  @override
  String get flashcardsDraftNotebook => 'Notebook';

  @override
  String flashcardsProgress(int at, int total, int fresh) {
    String _temp0 = intl.Intl.pluralLogic(
      fresh,
      locale: localeName,
      other: ' · $fresh new',
      zero: '',
    );
    return '$at of $total$_temp0';
  }

  @override
  String flashcardsInterval(String name, String span) {
    return '$name · $span';
  }

  @override
  String flashcardsAddCards(int n) {
    String _temp0 = intl.Intl.pluralLogic(
      n,
      locale: localeName,
      other: 'Add $n Cards',
      one: 'Add 1 Card',
    );
    return '$_temp0';
  }

  @override
  String flashcardsAdded(int n) {
    String _temp0 = intl.Intl.pluralLogic(
      n,
      locale: localeName,
      other: 'Added $n cards.',
      one: 'Added 1 card.',
    );
    return '$_temp0';
  }

  @override
  String flashcardsQuizProgress(int at, int total) {
    return 'Question $at of $total';
  }

  @override
  String flashcardsDraftSent(int n) {
    String _temp0 = intl.Intl.pluralLogic(
      n,
      locale: localeName,
      other: 'Only these $n notes are sent to your AI.',
      one: 'Only this note is sent to your AI.',
    );
    return '$_temp0';
  }

  @override
  String get flashcardsShapeReport => 'Report';

  @override
  String get flashcardsShapeEssay => 'Essay';

  @override
  String get flashcardsShapeUpdate => 'Update';

  @override
  String get flashcardsShapeOutline => 'Outline';

  @override
  String get flashcardsShapeCustom => 'Custom';

  @override
  String get flashcardsLengthShort => 'Short';

  @override
  String get flashcardsLengthMedium => 'Medium';

  @override
  String get flashcardsLengthLong => 'Long';

  @override
  String get flashcardsDraftFromNotebook => 'Draft from This Notebook…';

  @override
  String get flashcardsDraftFromTag => 'Draft from This Tag…';

  @override
  String flashcardsDraftFromNotes(int n) {
    String _temp0 = intl.Intl.pluralLogic(
      n,
      locale: localeName,
      other: 'Draft from These $n Notes…',
      one: 'Draft from This Note…',
    );
    return '$_temp0';
  }

  @override
  String get flashcardsDraftNoNotes =>
      'There are no notes here that a draft can read. Locked notes and notes tagged #ai-exclude are never sent.';

  @override
  String get flashcardsDraftNoSourceNote =>
      'Underlined: no source in your notes. These are left out when you save.';

  @override
  String get flashcardsDraftKeepUnsourced => 'Keep sentences without a source';

  @override
  String get askNeverWrote => 'You never wrote that down.';

  @override
  String get askClosestNotes => 'Closest notes';

  @override
  String get askSources => 'Sources';

  @override
  String get askAlsoRead => 'Also read';

  @override
  String get askNotFound => 'Not found in your notes';

  @override
  String get askNotFoundHint =>
      'The answer said this, but the lines it pointed to aren\'t in the notes Kanso sent. Treat it as unverified.';

  @override
  String get askBannerMostly =>
      'Most of this answer couldn\'t be checked against your notes.';

  @override
  String get askBannerNoSources =>
      'This answer has no sources. Your model may not follow citation instructions; try a larger model.';

  @override
  String get askSearchAll => 'Search all notes';

  @override
  String get askSaveAsNote => 'Save as note';

  @override
  String get askSavedAsNote => 'Saved as a note.';

  @override
  String get askSaveFailed => 'Couldn\'t save the note.';

  @override
  String get askOpenNote => 'Open note';

  @override
  String get askScopeTitle => 'Search in';

  @override
  String get askAllNotes => 'All notes';

  @override
  String get askDropTime => 'Search without this time range';

  @override
  String get askReading => 'Reading your notes…';

  @override
  String get askWriting => 'Writing the answer…';

  @override
  String get askChecking => 'Checking the sources…';

  @override
  String get aiAskSection => 'Ask';

  @override
  String get aiAskBudget => 'Most each question may send';

  @override
  String get aiAskBudgetSmall => 'Small';

  @override
  String get aiAskBudgetStandard => 'Standard';

  @override
  String get aiAskBudgetLarge => 'Large';

  @override
  String get aiAskCheckSources => 'Check sources';

  @override
  String get aiAskCheckSourcesOff =>
      'Without checking, an answer may cite lines that were never sent.';

  @override
  String get aiAskCheckSourcesOn =>
      'Each cited line is checked against what was sent. A sentence that fails is marked as not found.';

  @override
  String get aiAskUseLinks => 'Look up related notes through links';

  @override
  String get aiAskUseLinksFooter =>
      'Notes that link to the best matches, and notes they link to, are looked up too. Locked notes and notes tagged #ai-exclude are never followed.';

  @override
  String get askLinesChanged =>
      'These lines have changed since the answer was written.';

  @override
  String askNoSource(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count sentences cite no lines of your notes.',
      one: '1 sentence cites no lines of your notes.',
    );
    return '$_temp0';
  }

  @override
  String get peopleTitle => 'People & Projects';

  @override
  String peopleSubtitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count pages',
      one: '1 page',
      zero: 'No pages yet',
    );
    return '$_temp0';
  }

  @override
  String peopleNewBadge(int count) {
    return '$count new';
  }

  @override
  String get peopleNewPerson => 'New person…';

  @override
  String get peopleNewProject => 'New project…';

  @override
  String get peopleNewPersonTitle => 'New person';

  @override
  String get peopleNewProjectTitle => 'New project';

  @override
  String get peopleNameHint => 'Name';

  @override
  String get peopleAlsoKnownAsHint => 'Also known as (optional)';

  @override
  String get peopleCreate => 'Create';

  @override
  String get peopleSuggestionsTitle => 'Suggestions';

  @override
  String get peoplePagesTitle => 'Pages';

  @override
  String get peopleCreatePage => 'Create page';

  @override
  String get peopleNotPerson => 'Not a person';

  @override
  String get peopleNotProject => 'Not a project';

  @override
  String get peopleNotNow => 'Not now';

  @override
  String peopleAliasQuestion(String name, String page) {
    return 'Is “$name” the same as $page?';
  }

  @override
  String get peopleAliasYes => 'Yes, add alias';

  @override
  String get peopleAliasNo => 'No, different';

  @override
  String get peopleEmptyTitle => 'No people or projects yet';

  @override
  String get peopleEmptyMessage =>
      'Kanso suggests a page when a name keeps coming up in your notes. Or make one yourself.';

  @override
  String get peopleSuggestionsOff =>
      'Suggestions are off. Turn them on in Settings › People & Projects.';

  @override
  String get peopleKindPerson => 'Person';

  @override
  String get peopleKindProject => 'Project';

  @override
  String get peopleRefresh => 'Refresh';

  @override
  String get peopleRefreshAll => 'Refresh all';

  @override
  String get peopleKeep => 'Keep this version';

  @override
  String get peopleUnkeep => 'Stop keeping this version';

  @override
  String get peopleCopyInto => 'Copy into the note';

  @override
  String get peopleCopiedInto => 'Copied into the note';

  @override
  String get peopleHide => 'Hide section';

  @override
  String get peopleShowHidden => 'Show hidden sections';

  @override
  String get peopleNotRight => 'Not right';

  @override
  String peopleShowAll(int total) {
    return 'Show all $total';
  }

  @override
  String get peopleMergeWith => 'Merge with…';

  @override
  String get peopleMergeTitle => 'Merge with';

  @override
  String get peopleMergeNone => 'There is no other page to merge with.';

  @override
  String peopleMergeConfirmTitle(String name) {
    return 'Merge into $name?';
  }

  @override
  String peopleMergeConfirmMessage(String name) {
    return 'This page\'s names and text move to $name, and this page goes to the Trash.';
  }

  @override
  String get peopleMergeConfirm => 'Merge';

  @override
  String get peopleAddAlias => 'Add alias…';

  @override
  String get peopleAddAliasTitle => 'Another name';

  @override
  String get peopleSettingsTitle => 'People & Projects';

  @override
  String get peopleSettingsSuggest => 'Suggest people and projects';

  @override
  String get peopleSettingsSuggestHelp =>
      'Kanso looks for names that keep coming up in your notes, on this device only. Nothing becomes a page until you say so.';

  @override
  String get peopleSettingsWriteAi => 'Write summaries with AI';

  @override
  String get peopleSettingsWriteAiHelp =>
      'Off keeps the computed sections only.';

  @override
  String get peopleSettingsRefresh => 'Refresh AI sections';

  @override
  String get peopleRefreshAuto => 'What suits your model';

  @override
  String get peopleRefreshOpen => 'When I open an out-of-date page';

  @override
  String get peopleRefreshPress => 'Only when I press Refresh';

  @override
  String get peopleSettingsDetectAi => 'Use AI to find people and projects';

  @override
  String get peopleSettingsDetectAiHelp =>
      'Once a day at most, your model labels the suggestions as people, projects or neither. It never makes a page.';

  @override
  String get peopleSettingsDismissed => 'Dismissed names';

  @override
  String get peopleSuggestAgain => 'Suggest again';

  @override
  String get peopleDismissedNone => 'Nothing dismissed.';

  @override
  String get peopleFailed => 'Couldn\'t read People & Projects.';

  @override
  String get peopleYouPromised => 'You promised';

  @override
  String get peopleTheyPromised => 'They promised';

  @override
  String get peopleSharedNote => 'shared note';

  @override
  String get peopleSettingsKeepIn => 'Keep pages in';

  @override
  String get peopleSettingsKeepInDefault => 'People & Projects';

  @override
  String get peopleSettingsKeepInHelp =>
      'The notebook new person and project pages go in. The default is made when the first page is.';

  @override
  String get notebookHomeChooseHome => 'Choose Home note…';

  @override
  String get notebookHomeNewHome => 'New Home note';

  @override
  String get notebookHomeCustomise => 'Customise home…';

  @override
  String get notebookHomeCopyInto => 'Copy into Home note';

  @override
  String get notebookHomeCopiedInto => 'Copied into the Home note';

  @override
  String get notebookHomeEdit => 'Edit';

  @override
  String get notebookHomeChooseTitle => 'Home note';

  @override
  String get notebookHomeChooseNone =>
      'This notebook has no notes to choose from yet.';

  @override
  String get notebookHomeCustomiseTitle => 'Customise home';

  @override
  String get notebookHomeResetLayout => 'Reset layout';

  @override
  String get notebookHomeDone => 'Done';

  @override
  String get notebookHomeFailed => 'Couldn\'t read this homepage.';

  @override
  String notebookHomeMapLabel(String name) {
    return 'Map of the notes in $name';
  }

  @override
  String get notebookHomeSettingsTitle => 'Notebook homepages';

  @override
  String get notebookHomeSettingsShow => 'Show notebook homepages';

  @override
  String get notebookHomeSettingsShowHelp =>
      'A card above a notebook\'s notes opens its homepage. Off hides the card and the page.';

  @override
  String get notebookHomeSettingsAi => 'Write notebook summaries with AI';

  @override
  String get notebookHomeSettingsAiHelp =>
      'Off keeps the computed sections only. How often they refresh is the setting under Write summaries with AI, above.';

  @override
  String get recordingTitle => 'Record';

  @override
  String get recordingStart => 'Start recording';

  @override
  String get recordingStop => 'Stop';

  @override
  String get recordingPause => 'Pause';

  @override
  String get recordingResume => 'Resume';

  @override
  String get recordingDiscard => 'Discard';

  @override
  String get recordingCancel => 'Cancel';

  @override
  String get recordingBack => 'Back';

  @override
  String get recordingSaving => 'Saving…';

  @override
  String get recordingClockLabel => 'Time recorded';

  @override
  String get recordingDiscardTitle => 'Discard this recording?';

  @override
  String get recordingDiscardBody =>
      'The audio recorded so far is deleted and can\'t be brought back.';

  @override
  String get recordingRecover => 'Recover';

  @override
  String get recordingIntoNewNote => 'Into a new note';

  @override
  String get recordingIntoThisNote => 'Into this note';

  @override
  String get recordingReminder =>
      'Tell everyone nearby you are recording. The rules differ by place. The audio stays on this device until you sync.';

  @override
  String get recordingProblemBusy =>
      'A recording is already running. Stop it before starting another.';

  @override
  String get recordingProblemCannotStart =>
      'Kanso couldn\'t use the microphone. Allow it in Settings, then try again.';

  @override
  String get recordingProblemLocked =>
      'That note is locked. Unlock it first, then record into it.';

  @override
  String get recordingProblemNoteGone => 'That note is gone.';

  @override
  String get recordingProblemSaveFailed =>
      'The recording couldn\'t be saved. Kanso keeps the audio and offers to recover it.';

  @override
  String get recordingMenuRecordHere => 'Record into this note';

  @override
  String get libraryRecord => 'Record';

  @override
  String get feedbackSection => 'Feedback';

  @override
  String get feedbackSend => 'Send feedback';

  @override
  String get feedbackMine => 'My feedback';

  @override
  String feedbackFixedIn(String release) {
    return 'Fixed in $release';
  }

  @override
  String get feedbackSectionFooter =>
      'Your words and screenshot stay private with Kanso. No notes are ever sent.';

  @override
  String get feedbackKindLabel => 'What kind of feedback';

  @override
  String get feedbackKindBug => 'Something’s wrong';

  @override
  String get feedbackKindIdea => 'Idea';

  @override
  String get feedbackKindQuestion => 'Question';

  @override
  String get feedbackHintProblem => 'What happened, and what did you expect?';

  @override
  String get feedbackHintIdea => 'What would you like Kanso to do?';

  @override
  String feedbackCounter(int count, int max) {
    return '$count / $max';
  }

  @override
  String get feedbackAddScreenshot => 'Add screenshot';

  @override
  String get feedbackRemoveScreenshot => 'Remove';

  @override
  String get feedbackScreenshotWarning =>
      'Screenshots can show your notes. Check before sending.';

  @override
  String feedbackScreenshotAttached(String name) {
    return 'Screenshot: $name';
  }

  @override
  String get feedbackIncludeAppInfo => 'Include app info';

  @override
  String get feedbackIncludeAppInfoDetail =>
      'Version, system and recent errors. Never your notes.';

  @override
  String get feedbackShowSent => 'Show what’s sent';

  @override
  String get feedbackHideSent => 'Hide what’s sent';

  @override
  String get feedbackSentEmpty =>
      'The report appears here once it has at least 10 characters.';

  @override
  String get feedbackCredit => 'Credit me by name';

  @override
  String get feedbackCreditDetail =>
      'If this is fixed, your name is in the release notes. Never a link to you.';

  @override
  String get feedbackCreditName => 'Your name';

  @override
  String get feedbackFooter =>
      'Your words and screenshot stay private with Kanso. A short summary may become a public GitHub issue. No notes are ever sent.';

  @override
  String get feedbackSendButton => 'Send';

  @override
  String get feedbackSending => 'Sending…';

  @override
  String get feedbackSentToast => 'Thanks — feedback sent.';

  @override
  String get feedbackQueuedToast =>
      'Saved — it will send when you’re back online.';

  @override
  String get feedbackPausedToast =>
      'Feedback is paused for now. Your report is saved and will send later.';

  @override
  String get feedbackRateLimitedToast =>
      'You’ve sent a lot of feedback today. Try again tomorrow.';

  @override
  String get feedbackErrorTooShort =>
      'Say a little more (at least 10 characters).';

  @override
  String get feedbackErrorTooLong => 'Keep it under 4,000 characters.';

  @override
  String get feedbackErrorCharacter =>
      'Take out the control characters and try again.';

  @override
  String get feedbackErrorCredit =>
      'Use 1 to 39 letters, numbers, spaces, dots, dashes or underscores for your name.';

  @override
  String get feedbackErrorScreenshot =>
      'That image is over 5 MB. Choose a smaller one.';

  @override
  String get feedbackErrorOther =>
      'Kanso couldn’t build this report. Turn off app info and try again.';

  @override
  String feedbackErrorRejected(String field) {
    return 'Kanso couldn’t send this report ($field). Edit it and try again.';
  }

  @override
  String get feedbackErrorRejectedPlain =>
      'Kanso couldn’t send this report. Edit it and try again.';

  @override
  String get feedbackMineIntro =>
      'The feedback you sent and where each report stands. Pull down to check.';

  @override
  String get feedbackMineLoading => 'Reading your feedback';

  @override
  String get feedbackMineFailed => 'Kanso couldn’t read your feedback.';

  @override
  String get feedbackMineEmptyTitle => 'No feedback yet';

  @override
  String get feedbackMineEmptyMessage =>
      'Reports you send from Settings › Send feedback show up here.';

  @override
  String get feedbackChipWaiting => 'Waiting to send';

  @override
  String get feedbackChipReceived => 'Received';

  @override
  String feedbackChipFiled(int number) {
    return 'Filed as #$number';
  }

  @override
  String get feedbackChipBuilding => 'Building';

  @override
  String get feedbackChipFixProposed => 'Fix proposed';

  @override
  String get feedbackChipMerged => 'Merged';

  @override
  String feedbackChipReleased(String release) {
    return 'Released in $release';
  }

  @override
  String get feedbackChipNotFiled => 'Not filed';

  @override
  String feedbackChipSame(int number) {
    return 'Same as #$number';
  }

  @override
  String get feedbackChipClosed => 'Closed';

  @override
  String get feedbackChipDeleting => 'Deleting…';

  @override
  String get feedbackDetailNotFiled =>
      'Not filed — it didn’t look like a bug or an idea. Still kept privately for the owner to read.';

  @override
  String feedbackDetailSame(int number) {
    return 'Your report was added to issue #$number.';
  }

  @override
  String get feedbackOpenOnGithub => 'Open on GitHub';

  @override
  String get feedbackDelete => 'Delete';

  @override
  String get feedbackDeleteTitle => 'Delete this report?';

  @override
  String get feedbackDeleteMessage =>
      'Kanso removes your text, screenshot and app info from its server. A GitHub issue made from it stays, but holds no words of yours.';

  @override
  String get feedbackTimeline => 'Where it stands';

  @override
  String get feedbackStepFiled => 'Filed';

  @override
  String get libraryCanvases => 'Canvases';

  @override
  String get emptyCanvasesTitle => 'No canvases yet';

  @override
  String get emptyCanvasesMessage =>
      'Draw, write and pin notes on an endless page. Tap New to start one.';

  @override
  String get canvasNew => 'New canvas';

  @override
  String get canvasUntitled => 'Untitled canvas';

  @override
  String get canvasToolPen => 'Pen';

  @override
  String get canvasToolPencil => 'Pencil';

  @override
  String get canvasToolMarker => 'Marker';

  @override
  String get canvasToolHighlighter => 'Highlighter';

  @override
  String get canvasToolEraser => 'Eraser';

  @override
  String get canvasToolPartialEraser => 'Partial eraser';

  @override
  String get canvasToolLasso => 'Select';

  @override
  String get canvasToolText => 'Text card';

  @override
  String get canvasToolArrow => 'Arrow';

  @override
  String get canvasToolHand => 'Move canvas';

  @override
  String get canvasUndo => 'Undo';

  @override
  String get canvasRedo => 'Redo';

  @override
  String get canvasFit => 'Fit to content';

  @override
  String get canvasColor => 'Ink color';

  @override
  String get canvasWidth => 'Pen width';

  @override
  String get canvasFingerDraws => 'Finger draws';

  @override
  String get canvasFingerDrawsHelp =>
      'Off, a finger only moves and zooms the canvas. It turns off by itself when a pen is used.';

  @override
  String get canvasMore => 'More';

  @override
  String get canvasSurface => 'Canvas';

  @override
  String get canvasBudgetWarnTitle => 'This canvas is getting large';

  @override
  String get canvasBudgetWarnMessage =>
      'Select some ink and move it to a new canvas before this one is full.';

  @override
  String get canvasBudgetFullTitle => 'This canvas is full';

  @override
  String get canvasBudgetFullMessage =>
      'New ink is refused until you move some to a new canvas.';

  @override
  String get canvasMoveToNew => 'Move to a new canvas';

  @override
  String get canvasMovedTitle => 'Moved ink';

  @override
  String get canvasSelectionDelete => 'Delete selection';

  @override
  String get canvasSelectionClear => 'Clear selection';

  @override
  String get canvasCardTitle => 'Text card';

  @override
  String get canvasCardHint => 'Write on the card';

  @override
  String get canvasCardSave => 'Done';

  @override
  String get canvasCardDelete => 'Delete card';

  @override
  String get canvasCardOpenNote => 'Open note';

  @override
  String get canvasCardMissing => 'This note is gone.';

  @override
  String get canvasNotSaved =>
      'Couldn\'t save. Your drawing is kept here and will be saved again.';

  @override
  String get canvasFailed => 'Couldn\'t open this canvas.';

  @override
  String get canvasLockedTitle => 'This canvas is locked';

  @override
  String get canvasLockedMessage =>
      'Unlock it from the notes list to draw on it.';

  @override
  String get canvasExportSvg => 'Export as image';

  @override
  String get canvasExportExcalidraw => 'Export to Excalidraw';

  @override
  String get canvasExported => 'Copied to the clipboard.';

  @override
  String get canvasExportFailed => 'Couldn\'t export this canvas.';

  @override
  String get canvasBack => 'Back';

  @override
  String get canvasSharedLabel => 'Shared';

  @override
  String get canvasViewOnlyLabel => 'View only';

  @override
  String get canvasViewOnlyMessage =>
      'You can look around and zoom, but you can\'t draw on this canvas.';

  @override
  String canvasOthersHere(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count others here',
      one: '1 other here',
    );
    return '$_temp0';
  }

  @override
  String canvasPointerLabel(String name) {
    return '$name\'s pointer';
  }

  @override
  String get canvasShareAction => 'Share this canvas';

  @override
  String get overviewTitle => 'Overview';

  @override
  String get overviewToday => 'Today';

  @override
  String get overviewThisWeek => 'This week';

  @override
  String get overviewThatWeek => 'That week';

  @override
  String get overviewNotes => 'Notes';

  @override
  String get overviewSummarise => 'Summarise';

  @override
  String get overviewMenu => 'More';

  @override
  String get overviewOpenTodayNote => 'Open today\'s note';

  @override
  String get overviewSaveWeek => 'Save week as note';

  @override
  String get overviewSettings => 'Overview settings';

  @override
  String get overviewCancel => 'Cancel';

  @override
  String get overviewRevisitTitle => 'Revisit in…';

  @override
  String get overviewRevisit => 'Revisit in…';

  @override
  String overviewRevisitSet(String day) {
    return 'You\'ll see this again on $day.';
  }

  @override
  String overviewTick(String title) {
    return 'Tick off $title';
  }

  @override
  String get overviewTickOff => 'Tick off';

  @override
  String get overviewOpenNote => 'Open note';

  @override
  String get overviewMeetingNote => 'Meeting note';

  @override
  String get overviewSeriesNote => 'Series note';

  @override
  String overviewLastTime(String day) {
    return 'Last time: $day';
  }

  @override
  String get overviewNewMeetingNote => 'New meeting note';

  @override
  String get overviewMeetingNoteError => 'Couldn\'t create the meeting note.';

  @override
  String get overviewEventDetails => 'Event details';

  @override
  String get overviewOpenInCalendar => 'Open in Calendar';

  @override
  String get overviewLinkToNote => 'Link to a note…';

  @override
  String get overviewCopyMeetingLink => 'Copy meeting link';

  @override
  String get overviewHideCalendar => 'Hide this calendar';

  @override
  String get overviewChooseNote => 'Choose a note';

  @override
  String get overviewNotNow => 'Not now';

  @override
  String get overviewDone => 'Done';

  @override
  String get overviewNotYet => 'Not yet';

  @override
  String overviewQuietDays(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count days quiet',
      one: '1 day quiet',
    );
    return '$_temp0';
  }

  @override
  String get overviewReview => 'Review';

  @override
  String overviewInbox(int count) {
    return 'Inbox $count';
  }

  @override
  String overviewPromisesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count things you said you\'d do',
      one: '1 thing you said you\'d do',
    );
    return '$_temp0';
  }

  @override
  String overviewWaitingCount(int count) {
    return '$count waiting';
  }

  @override
  String overviewFlashcards(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count flashcards',
      one: '1 flashcard',
    );
    return '$_temp0';
  }

  @override
  String overviewFromThePast(String why) {
    return 'A note from the past · $why';
  }

  @override
  String get overviewOpen => 'Open';

  @override
  String get overviewNotUseful => 'Not useful';

  @override
  String get overviewMore => 'More';

  @override
  String overviewWrittenBy(String time) {
    return 'Written by your model at $time';
  }

  @override
  String get overviewRewrite => 'Rewrite';

  @override
  String get overviewHide => 'Hide';

  @override
  String overviewTicked(String title) {
    return 'Done: $title';
  }

  @override
  String get overviewNewLede =>
      'This is your Overview. As you write, it gathers today\'s meetings and tasks, the notes you\'re in the middle of, and the people you\'re waiting on.';

  @override
  String get overviewWriteToday => 'Write today\'s note';

  @override
  String overviewAgendaMore(int count, int total) {
    return '$count more today · All $total in Tasks';
  }

  @override
  String overviewEarlier(int count) {
    return 'Earlier today ($count)';
  }

  @override
  String get overviewAlso => 'Also';

  @override
  String get overviewPickUp => 'Pick up where you left off';

  @override
  String get overviewWaiting => 'Waiting on others';

  @override
  String get overviewActivity => 'Agent and shared activity';

  @override
  String overviewMoreRows(int count) {
    return '$count more';
  }

  @override
  String get overviewTriage => 'Triage';

  @override
  String get overviewSeeAll => 'See all';

  @override
  String get overviewPreviousWeek => 'Previous week';

  @override
  String get overviewNextWeek => 'Next week';

  @override
  String get overviewProjects => 'Active projects';

  @override
  String overviewNotesChanged(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes changed',
      one: '1 note changed',
    );
    return '$_temp0';
  }

  @override
  String overviewLast(String title) {
    return 'last: “$title”';
  }

  @override
  String overviewNext(String task) {
    return 'Next: $task';
  }

  @override
  String get overviewFinished => 'Finished';

  @override
  String get overviewStillOpen => 'Still open';

  @override
  String overviewAllOpenTasks(int count) {
    return 'All $count open tasks';
  }

  @override
  String get overviewAhead => 'Ahead';

  @override
  String overviewSavedAs(String title) {
    return 'Saved as “$title”';
  }

  @override
  String get settingsOverviewOpens => 'Open to Overview';

  @override
  String get settingsOverviewQuiet => 'Waiting on others after';

  @override
  String settingsOverviewDays(int count) {
    return '$count days';
  }

  @override
  String get settingsOverviewFewer => 'One day fewer';

  @override
  String get settingsOverviewMore => 'One day more';

  @override
  String get playerCardLabel => 'Recording player';

  @override
  String get playerPlay => 'Play';

  @override
  String get playerPause => 'Pause';

  @override
  String get playerPosition => 'Position in the recording';

  @override
  String playerPositionValue(String elapsed, String total) {
    return '$elapsed of $total';
  }

  @override
  String get playerBack => 'Back 5 seconds';

  @override
  String get playerForward => 'Forward 5 seconds';

  @override
  String get playerSpeed => 'Playback speed';

  @override
  String get playerVolume => 'Volume';

  @override
  String get playerMute => 'Mute';

  @override
  String get playerUnmute => 'Unmute';

  @override
  String get playerNotHere => 'Audio not on this device yet';

  @override
  String get playerSyncNow => 'Sync Now';

  @override
  String get playerMissing => 'This recording\'s audio is missing';

  @override
  String get playerFailed => 'Couldn\'t play the recording';

  @override
  String get playerTryAgain => 'Try Again';

  @override
  String get sketchInsertKey => 'Insert sketch';

  @override
  String get sketchSizeTitle => 'Insert sketch';

  @override
  String get sketchSizeSmall => 'Small, 320 by 240';

  @override
  String get sketchSizeMedium => 'Medium, 480 by 320';

  @override
  String get sketchSizeLarge => 'Large, 640 by 480';

  @override
  String get sketchLabel => 'Sketch';

  @override
  String get sketchOpenAsCanvas => 'Open as canvas';

  @override
  String get sketchCopyAsCanvas => 'Copy as canvas';

  @override
  String get sketchShared =>
      'Shared note: copy it to a canvas to draw together.';

  @override
  String get sketchFull =>
      'This sketch is full. Open it as a canvas to keep drawing.';

  @override
  String get sketchFingerDraws => 'Draw with a finger';

  @override
  String get sketchFailed => 'This sketch can\'t be drawn on.';

  @override
  String get sketchCanvasMade => 'Opened as a canvas';

  @override
  String get sketchCanvasCopied => 'Copied to a new canvas';

  @override
  String sketchTitle(String title) {
    return '$title sketch';
  }

  @override
  String get sketchUntitledNote => 'Untitled';

  @override
  String get sketchNoteFull =>
      'This note\'s sketches are full. Open one as a canvas.';

  @override
  String get sketchNearlyFull =>
      'This sketch is nearly full. Open it as a canvas to keep drawing without a limit.';

  @override
  String get sketchSharedInsert =>
      'Sketches can\'t be added to a shared note. Add one to a private note, or draw on a canvas.';

  @override
  String get noticeNeedsAttention => 'Needs your attention';

  @override
  String get noticeHowToFix => 'How to fix';

  @override
  String get noticeDismiss => 'Dismiss';

  @override
  String get noticeMarkFixed => 'Mark as fixed';

  @override
  String get noticeSheetTitle => 'Update notice';

  @override
  String get noticeSheetDone => 'Done';

  @override
  String noticeCount(int total) {
    return '1 of $total';
  }

  @override
  String get noticeOpenSettings => 'Open Settings';

  @override
  String get noticeLearnMore => 'Learn More';

  @override
  String noticeStep(int number, String step) {
    return 'Step $number: $step';
  }

  @override
  String lockFilesSealed(int count, String size) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Its pictures and files ($size) are',
      one: 'Its picture or file ($size) is',
    );
    return '$_temp0 sealed too.';
  }

  @override
  String lockFilesShared(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count attachments are',
      one: '1 attachment is',
    );
    String _temp1 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'copies',
      one: 'copy',
    );
    return '$_temp0 also used by other notes: the locked note keeps its own sealed $_temp1, and the other notes keep theirs.';
  }

  @override
  String get lockProgressTitleNote => 'Locking note';

  @override
  String get lockProgressTitleNotebook => 'Locking notebook';

  @override
  String get lockProgressTitleRemove => 'Removing lock';

  @override
  String lockProgressLine(int count, String size) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Sealing $count attachments, $size',
      one: 'Sealing 1 attachment, $size',
    );
    return '$_temp0';
  }

  @override
  String lockProgressUnsealingLine(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Unsealing $count attachments',
      one: 'Unsealing 1 attachment',
    );
    return '$_temp0';
  }

  @override
  String lockProgressCount(int done, int total) {
    return '$done of $total';
  }

  @override
  String get lockSizeUnderOneKb => 'under 1 KB';

  @override
  String get lockProgressPreparing => 'Getting ready';

  @override
  String get lockProgressLocking => 'Locking the note';

  @override
  String get lockProgressCancel => 'Cancel';

  @override
  String get lockProgressStopping => 'Stopping after this file';

  @override
  String lockCopiedMessage(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          'Locked. $count attachments are also used by other notes, so this note has its own sealed copies.',
      one:
          'Locked. 1 attachment is also used by other notes, so this note has its own sealed copy.',
    );
    return '$_temp0';
  }

  @override
  String lockUnsealedMessage(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          'Lock removed. $count attachments were copied for this note, because another locked note still shows the sealed ones.',
      one:
          'Lock removed. 1 attachment was copied for this note, because another locked note still shows the sealed one.',
    );
    return '$_temp0';
  }

  @override
  String get lockRemoveStoppedMessage =>
      'Removing the lock stopped. The note is still locked, and its pictures and files are still sealed.';

  @override
  String get lockStoppedMessage =>
      'Locking stopped. The note isn\'t locked yet: finish locking it or undo it from the bar above the note.';

  @override
  String get lockStoppedNotebookMessage =>
      'Locking stopped. The notebook isn\'t locked yet: lock it again to finish.';

  @override
  String lockInterruptedMessage(int sealed, int files) {
    return 'Locking this note didn\'t finish: $sealed of its $files pictures and files are sealed, and the note isn\'t locked yet. Finish locking, or undo it to keep everything as it was.';
  }

  @override
  String get lockInterruptedFinish => 'Finish locking';

  @override
  String get lockInterruptedUndo => 'Undo locking';

  @override
  String get lockInterruptedWorking => 'Working…';

  @override
  String get lockInterruptedPasswordTitle => 'Enter the lock password';

  @override
  String get lockInterruptedPasswordMessage =>
      'The notes are closed. The lock password this note was being locked with finishes or undoes it.';

  @override
  String get lockInterruptedContinue => 'Continue';

  @override
  String get lockInterruptedFailed => 'That didn\'t work. Try again.';

  @override
  String get lockFilesNoticeDismiss => 'Dismiss';

  @override
  String get lockMigrationTitle => 'Seal attachments of locked notes';

  @override
  String get lockMigrationPrompt =>
      'Attachments of your locked notes aren\'t sealed yet. Enter your lock password to seal them now.';

  @override
  String lockMigrationNotes(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count locked notes still have pictures and files stored in plain form.',
      one: '1 locked note still has pictures and files stored in plain form.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationCounted(int count, String size) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count attachments ($size) will be sealed.',
      one: '1 attachment ($size) will be sealed.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationShared(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count attachments are also used by other notes: each locked note gets its own sealed copy, and the other notes keep theirs.',
      one:
          '1 attachment is also used by another note: the locked note gets its own sealed copy, and the other note keeps the original.',
    );
    return '$_temp0';
  }

  @override
  String get lockMigrationUncounted =>
      'Kanso counts them, and their size, once the password opens the notes.';

  @override
  String get lockMigrationSealNow => 'Seal now';

  @override
  String get lockMigrationLater => 'Later';

  @override
  String get lockMigrationPasswordMessage =>
      'Type your lock password. Kanso seals the attachments of every locked note it opens.';

  @override
  String get lockMigrationSealing => 'Sealing attachments';

  @override
  String get lockMigrationStoppedTitle => 'Sealing didn\'t finish';

  @override
  String lockMigrationStoppedMessage(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count locked notes were left half done.',
      one: '1 locked note was left half done.',
    );
    return '$_temp0 Some attachments are sealed, and none is lost. Finish sealing, or undo it to keep everything as it was.';
  }

  @override
  String get lockMigrationFinish => 'Finish sealing';

  @override
  String get lockMigrationUndo => 'Undo';

  @override
  String lockMigrationDone(int files, int notes) {
    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: 'Sealed $files attachments',
      one: 'Sealed 1 attachment',
    );
    String _temp1 = intl.Intl.pluralLogic(
      notes,
      locale: localeName,
      other: '$notes locked notes',
      one: '1 locked note',
    );
    return '$_temp0 in $_temp1.';
  }

  @override
  String lockMigrationNothing(int notes) {
    String _temp0 = intl.Intl.pluralLogic(
      notes,
      locale: localeName,
      other: 'Checked $notes locked notes: no attachments needed sealing.',
      one: 'Checked 1 locked note: no attachments needed sealing.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationCopies(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count attachments also used by other notes have their own sealed copies.',
      one: '1 attachment also used by another note has its own sealed copy.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationUnopened(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count locked notes use another lock password and were left as they are.',
      one: '1 locked note uses another lock password and was left as it is.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationFailedNotes(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count locked notes couldn\'t be sealed and stay as they were.',
      one: '1 locked note couldn\'t be sealed and stays as it was.',
    );
    return '$_temp0';
  }

  @override
  String get lockMigrationStopped =>
      'Sealing stopped. Open Settings, then Locked notes, to finish it.';

  @override
  String lockMigrationUndone(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Undone. $count attachments are readable again, as before.',
      one: 'Undone. 1 attachment is readable again, as before.',
    );
    return '$_temp0';
  }

  @override
  String lockMigrationReminder(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Attachments of $count locked notes aren\'t sealed yet.',
      one: 'Attachments of 1 locked note aren\'t sealed yet.',
    );
    return '$_temp0';
  }

  @override
  String get settingsSealAttachments => 'Seal attachments of locked notes…';

  @override
  String settingsSealAttachmentsValue(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$_temp0';
  }

  @override
  String get lockMigrationClose => 'Done';

  @override
  String get lockFilesNotice =>
      'Locked notes now seal their pictures and files too. A note locked before is sealed the first time you open it. Update Kanso on every device before you change the lock password: an older version can\'t re-seal the files, and they would stop opening.';

  @override
  String get pdfNotesInkTitle => 'Pencil marks';

  @override
  String pdfNotesInkStrokes(int strokes) {
    String _temp0 = intl.Intl.pluralLogic(
      strokes,
      locale: localeName,
      other: '$strokes strokes',
      one: '1 stroke',
    );
    return '$_temp0';
  }

  @override
  String get pdfNotesInkClear => 'Clear page';

  @override
  String get aiTroubleAgentSignIn =>
      'Your agent needs you to sign in. Sign in to your agent, then try again.';

  @override
  String get aiTroubleBilling =>
      'Your provider needs a billing change. Check your billing or plan with your AI provider, then try again.';

  @override
  String get aiTroubleWrongAddress =>
      'Your model server address looks wrong. Check the address in Settings › AI, then try again.';

  @override
  String get recordingSpeakersMenu => 'Rename speakers';

  @override
  String get recordingSpeakersTitle => 'Rename speakers';

  @override
  String get recordingSpeakersClose => 'Close';

  @override
  String get recordingSpeakersCancel => 'Cancel';

  @override
  String get recordingSpeakersRename => 'Rename';

  @override
  String get recordingSpeakersHint =>
      'Type a name for each voice. Two voices given the same name become one. The names are text in the note, and Undo brings the old ones back.';

  @override
  String get recordingSpeakersNobody =>
      'This note has no transcript that says who spoke. Transcribe the recording on a computer with Tell speakers apart turned on, or start a paragraph with a name and a colon. Locked notes can\'t be renamed here.';

  @override
  String get recordingSpeakersRenamed => 'Speakers renamed.';

  @override
  String recordingSpeakersField(String name, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count paragraphs',
      one: '1 paragraph',
    );
    return '$name · $_temp0';
  }

  @override
  String recordingSpeakersFailed(String reason) {
    return 'Couldn\'t rename the speakers. $reason';
  }

  @override
  String get whatsNewSheetTitle => 'What\'s new';

  @override
  String whatsNewTitle(String version) {
    return 'Kanso $version';
  }

  @override
  String whatsNewEarlier(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Also since your last update: $count releases',
      one: 'Also since your last update: 1 release',
    );
    return '$_temp0';
  }

  @override
  String whatsNewEarlierVersion(String version) {
    return 'Kanso $version';
  }

  @override
  String get agentSessionAsking => 'Has a question';

  @override
  String get agentSessionDone => 'Done';

  @override
  String get agentSessionFailed => 'Failed';

  @override
  String get agentSessionDiscarded => 'Discarded';

  @override
  String agentSessionLine(String agent, String status, String when) {
    return '$agent · $status · $when';
  }

  @override
  String get agentSessionWhenNow => 'now';

  @override
  String agentSessionWhenMinutes(int count) {
    return '${count}m ago';
  }

  @override
  String agentSessionWhenHours(int count) {
    return '${count}h ago';
  }

  @override
  String get agentSessionWhenYesterday => 'yesterday';

  @override
  String agentSessionWhenDays(int count) {
    return '${count}d ago';
  }

  @override
  String get agentSessionUnread => 'New activity';

  @override
  String get agentSessionEnd => 'End session';

  @override
  String get agentSessionRename => 'Rename';

  @override
  String get agentSessionRenameTitle => 'Rename session';

  @override
  String get agentSessionRenameHint => 'Session name';

  @override
  String get agentSessionRenameSave => 'Save';

  @override
  String get agentSessionDelete => 'Delete session';

  @override
  String get agentSessionDeleteTitle => 'Delete this session?';

  @override
  String get agentSessionDeleteBody =>
      'Its transcript is removed from this phone. Changes already applied stay.';

  @override
  String get agentSessionMore => 'Session actions';

  @override
  String get agentSessionEndTitle => 'End this session?';

  @override
  String get agentSessionEndBody =>
      'The AI stops and the transcript stays. Changes it proposed stay for you to review.';

  @override
  String get agentSessionComposerHint => 'Ask for more…';

  @override
  String get agentSessionComposerWorking =>
      'The AI is working. You can write when it asks.';

  @override
  String get agentSessionComposerEnded => 'This session has ended.';

  @override
  String get agentSessionComposerSend => 'Send message';

  @override
  String agentSessionThought(int seconds) {
    return 'Thought for ${seconds}s';
  }

  @override
  String get agentSessionThoughtShort => 'Thought';

  @override
  String agentSessionTools(int count) {
    return 'Used $count tools';
  }

  @override
  String agentSessionAllowedOnce(String title) {
    return 'Allowed once: $title';
  }

  @override
  String agentSessionAllowedSession(String title) {
    return 'Allowed for this session: $title';
  }

  @override
  String agentSessionAllowedAlways(String title) {
    return 'Always allowed: $title';
  }

  @override
  String agentSessionRejected(String title) {
    return 'Rejected: $title';
  }

  @override
  String agentSessionRound(int round, String counts) {
    return 'Round $round · $counts';
  }

  @override
  String agentSessionRoundApplied(String counts) {
    return 'Applied $counts';
  }

  @override
  String agentSessionRoundDiscarded(String counts) {
    return 'Discarded $counts';
  }

  @override
  String agentSessionRoundUndone(String counts) {
    return 'Undone $counts';
  }

  @override
  String agentSessionRoundLost(String counts) {
    return '$counts are no longer available';
  }

  @override
  String get agentSessionUndo => 'Undo';

  @override
  String get agentSessionGone => 'This session is no longer here.';

  @override
  String get agentSessionToolDone => 'Done';

  @override
  String get agentSessionToolFailed => 'Failed';

  @override
  String get agentSessionToolRunning => 'Running';

  @override
  String get agentSessionYou => 'You';

  @override
  String agentSessionAgentSaid(String agent) {
    return '$agent said';
  }

  @override
  String get boardSettingsLabel => 'Board settings';

  @override
  String get boardSettingsTitle => 'Board settings';

  @override
  String get boardSettingsDone => 'Done';

  @override
  String get boardSettingsGroupBy => 'Group by';

  @override
  String get boardSettingsStatus => 'Status';

  @override
  String get boardSettingsGroupHint =>
      'Pick a select property to lay the notes out in a lane for each of its values.';

  @override
  String get boardSettingsLanes => 'Lanes';

  @override
  String get boardSettingsLaneName => 'Lane name';

  @override
  String boardSettingsMoveLane(String lane) {
    return 'Move $lane';
  }

  @override
  String boardSettingsRemoveLane(String lane) {
    return 'Remove $lane';
  }

  @override
  String get boardSettingsAddLane => 'Add a lane';

  @override
  String get boardSettingsAddLaneButton => 'Add lane';

  @override
  String boardSettingsShowUnset(String key) {
    return 'Show cards without a $key';
  }

  @override
  String boardSettingsNoValue(String key) {
    return 'No $key';
  }

  @override
  String get boardSettingsRemove => 'Remove';

  @override
  String boardHintGroup(String key) {
    return 'Touch and hold a card to change its $key.';
  }

  @override
  String boardHintGroupDrag(String key) {
    return 'Drag a card to change its $key.';
  }
}
