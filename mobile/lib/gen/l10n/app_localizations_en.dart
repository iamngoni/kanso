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
  String get graphEmpty => 'No linked notes to show.';

  @override
  String get graphFailed => 'Couldn\'t load the graph.';

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
  String get fontsTextSizeSection => 'Text size';

  @override
  String get fontsTextSizeFooter =>
      'Changes the text of notes only. The rest of Kanso follows your device’s text size.';

  @override
  String get fontsTextSizeSample =>
      'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.';

  @override
  String get textSizeSmall => 'Small';

  @override
  String get textSizeDefault => 'Default';

  @override
  String get textSizeLarge => 'Large';

  @override
  String get textSizeLarger => 'Larger';

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
      'Its text and the pictures and files it links to are sealed, and its earlier versions are deleted. A file another note still shows gets a sealed copy. It stays open until the locked notes lock again.';

  @override
  String get lockNotebookAskTitle => 'Lock notebook';

  @override
  String get lockNotebookConfirmTitle => 'Lock this notebook?';

  @override
  String get lockNotebookConfirmMessage =>
      'Every note in it, and the pictures and files they link to, are sealed, and new notes added to it are sealed too. A file another note still shows gets a sealed copy.';

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
      'The note\'s text is kept as plain text again: search, backlinks, tasks and the local API can read it.';

  @override
  String get lockRemoveNotebookTitle => 'Remove the notebook\'s lock?';

  @override
  String get lockRemoveNotebookMessage =>
      'The notes in it are kept as plain text again: search, backlinks, tasks and the local API can read them.';

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
}
