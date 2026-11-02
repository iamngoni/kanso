//! The icons the app draws. GPUI Kit's default bundle covers the components;
//! the app's own views use a few more from the Lucide catalog, embedded here
//! so the binary carries only what it shows. Icon themes draw over them
//! under paths of their own (`icon_themes`), which load from here too.

use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(
    pub ExtraIcons,
    [
        AlarmClock,
        Apple,
        Archive,
        ArrowUpDown,
        AtSign,
        Atom,
        Baby,
        Banknote,
        Bed,
        Bike,
        Bookmark,
        BookMarked,
        Brackets,
        Brain,
        Briefcase,
        Brush,
        Bug,
        Bus,
        Cake,
        Calculator,
        CalendarClock,
        CalendarDays,
        Camera,
        Car,
        Cat,
        ChartLine,
        ChefHat,
        Church,
        Circle,
        CircleArrowDown,
        CircleDot,
        CirclePause,
        Clapperboard,
        ClipboardList,
        ClipboardPaste,
        Clock,
        Cloud,
        CloudCheck,
        CloudOff,
        CloudRain,
        CloudUpload,
        Code,
        CodeXml,
        Coffee,
        Coins,
        Columns2,
        Command,
        Compass,
        Contact,
        Copy,
        CornerDownLeft,
        CreditCard,
        Database,
        DatabaseBackup,
        Dices,
        Dog,
        Download,
        Droplet,
        Dumbbell,
        Feather,
        FileKey,
        FilePlus,
        Files,
        Film,
        FingerprintPattern,
        Flag,
        Flame,
        FlaskConical,
        Flower,
        Focus,
        FolderHeart,
        FolderLock,
        Gamepad2,
        Gift,
        GitBranch,
        Goal,
        GraduationCap,
        Guitar,
        Hammer,
        HandHeart,
        Handshake,
        Hash,
        Heading,
        Heading1,
        Heading2,
        Heading3,
        Headphones,
        HeartPulse,
        Hourglass,
        House,
        Image,
        Infinity,
        Key,
        Keyboard,
        KeyRound,
        Landmark,
        Languages,
        Laptop,
        LayoutTemplate,
        Leaf,
        Library,
        Lightbulb,
        Link,
        Link2,
        List,
        ListChecks,
        ListFilter,
        ListOrdered,
        ListTodo,
        Lock,
        LockKeyhole,
        LockOpen,
        LogIn,
        LogOut,
        Mail,
        MapPin,
        Megaphone,
        MessageCircle,
        MessageSquare,
        MessageSquarePlus,
        Microscope,
        MonitorSmartphone,
        Mountain,
        Music,
        Newspaper,
        Notebook,
        NotebookPen,
        NotebookText,
        Package,
        PanelRight,
        PartyPopper,
        PawPrint,
        Pencil,
        PenTool,
        Phone,
        PiggyBank,
        Pill,
        Pin,
        PinOff,
        Plane,
        Plug,
        PlugZap,
        Power,
        Presentation,
        Puzzle,
        Receipt,
        RefreshCcw,
        Regex,
        Rocket,
        Rss,
        Ruler,
        Save,
        Scan,
        ScanText,
        School,
        ScrollText,
        Share2,
        ShieldAlert,
        ShieldCheck,
        Server,
        Shapes,
        Shield,
        Ship,
        ShoppingBag,
        ShoppingCart,
        Snowflake,
        Sofa,
        Sparkles,
        Split,
        SquareCheck,
        SquareKanban,
        SquarePen,
        Stethoscope,
        StickyNote,
        Store,
        SunMoon,
        Table,
        TableOfContents,
        Tag,
        Target,
        Tent,
        Terminal,
        TextQuote,
        TrainFront,
        Trash,
        Trees,
        Trophy,
        Truck,
        Type,
        Unplug,
        Upload,
        UserPlus,
        Users,
        Utensils,
        Video,
        Wallet,
        Waypoints,
        WholeWord,
        Wrench,
        Zap,
        ZoomIn,
        ZoomOut,
    ]
);

/// Whether the app draws a UI icon named `name` (`chevron-right`): one an
/// icon theme can draw in its place.
pub fn is_app_icon(name: &str) -> bool {
    let path = format!("icons/{name}.svg");
    matches!(ExtraIcons.load(&path), Ok(Some(_))) || matches!(Assets.load(&path), Ok(Some(_)))
}

/// The names of every UI icon the app draws, sorted.
pub fn app_icon_names() -> Vec<String> {
    KansoAssets
        .list("icons/")
        .unwrap_or_default()
        .iter()
        .filter_map(|path| {
            let name = path.strip_prefix("icons/")?.strip_suffix(".svg")?;
            (!name.contains('/')).then(|| name.to_owned())
        })
        .collect()
}

/// The default component icons plus [`ExtraIcons`], and the icons of every
/// icon theme read (`icons/@<key>/…`, see `icon_themes::themed_path`), and
/// the library's slots as drawn without a theme (`icons/library/…`).
#[derive(Clone, Copy, Debug, Default)]
pub struct KansoAssets;

impl AssetSource for KansoAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.starts_with(crate::icon_themes::THEMED_PREFIX) {
            return match crate::icon_themes::themed_svg(path) {
                Some(svg) => Ok(Some(Cow::Owned(svg.to_vec()))),
                None => Err(anyhow::anyhow!("no icon theme draws {path}")),
            };
        }
        if let Some(name) = path.strip_prefix(crate::icon_themes::BUILTIN_PREFIX) {
            return self.load(&format!("icons/{name}"));
        }
        if let Some(fallback) = crate::icon_themes::unthemed_slot_path(path) {
            return self.load(&fallback);
        }
        // The extra set answers `None` for paths it lacks; the default bundle
        // reports them as errors, so it goes last.
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::assets::IconName;
    use gpui_kit::component::IconNamed as _;

    /// Every extra icon and every default component icon loads as an SVG.
    #[test]
    fn extra_and_default_icons_load() {
        let extra = ExtraIcons.list("icons/").unwrap();
        assert!(extra.len() >= 30, "{extra:?}");
        for path in extra.iter().chain([
            &IconName::Search.path(),
            &gpui_kit::component::IconName::Inbox.path(),
        ]) {
            let bytes = KansoAssets.load(path).unwrap().expect("icon bytes");
            assert!(
                std::str::from_utf8(&bytes).unwrap().contains("<svg"),
                "{path}"
            );
        }
        assert!(KansoAssets.list("icons/").unwrap().len() > extra.len());
        // A path neither set has stays an error, as with the default bundle.
        assert!(KansoAssets.load("icons/not-an-icon.svg").is_err());
    }
}
