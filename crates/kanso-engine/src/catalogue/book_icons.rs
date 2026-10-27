//! The icons a notebook can show: a set to choose from, kept in the notebook
//! by name ([`kanso_core::model::BOOK_ICON_FIELD`]) so it syncs. An icon
//! theme can add more of its own, kept as `<theme>/<icon>`. Each app draws
//! the choices with its own icons; the desktop's are Lucide's, and a
//! choice's [`name`](Choice::name) is the Lucide icon's own name.

use kanso_core::Book;
use kanso_core::model::BOOK_ICON_FIELD;
use kanso_core::roles::{ROLE_FIELD, Role};

/// One icon a notebook can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    /// What the notebook keeps: the Lucide name.
    pub name: &'static str,
    /// What the picker's tooltip says.
    pub label: &'static str,
    /// More words the search finds it by, spaced.
    pub keywords: &'static str,
}

const fn choice(name: &'static str, label: &'static str, keywords: &'static str) -> Choice {
    Choice {
        name,
        label,
        keywords,
    }
}

/// The name of the first choice in the picker, which stands for the
/// default: a notebook keeps no icon for it.
pub const DEFAULT_NAME: &str = "notebook";

/// The icons to choose from, in the picker's order: notebooks and files,
/// work and code, plans, study and making things, then life.
pub const CHOICES: &[Choice] = &[
    choice("notebook", "Notebook", "book journal default"),
    choice("notebook-pen", "Journal", "diary writing log"),
    choice("notebook-text", "Notes", "text writing"),
    choice("book-open", "Book", "reading library"),
    choice("book-marked", "Bookmarked", "reading saved"),
    choice("library", "Library", "books shelf reading"),
    choice("folder", "Folder", "files directory"),
    choice("folder-open", "Open folder", "files directory"),
    choice("folder-heart", "Favourite folder", "files love"),
    choice("folder-lock", "Locked folder", "private secret files"),
    choice("archive", "Archive", "old storage box"),
    choice("inbox", "Inbox", "mail incoming"),
    choice("mail", "Mail", "email letter"),
    choice("file-text", "Document", "page doc paper"),
    choice("files", "Files", "documents pages"),
    choice("scroll-text", "Scroll", "history story text"),
    choice("sticky-note", "Sticky note", "memo post-it"),
    choice("bookmark", "Bookmark", "saved later"),
    choice("briefcase", "Work", "job office business"),
    choice("building-2", "Company", "office business org"),
    choice("landmark", "Institution", "bank government"),
    choice("users", "People", "team group"),
    choice("user", "Person", "me profile"),
    choice("contact", "Contacts", "people address"),
    choice("handshake", "Deals", "partners clients agreement"),
    choice("bot", "Agents", "ai robot assistant"),
    choice("brain", "Brain", "mind thinking ai"),
    choice("cpu", "Hardware", "chip computer"),
    choice("code", "Code", "programming dev"),
    choice("terminal", "Terminal", "shell command cli"),
    choice("git-branch", "Git", "branch version source"),
    choice("database", "Database", "data sql storage"),
    choice("server", "Server", "backend hosting"),
    choice("cloud", "Cloud", "hosting sky"),
    choice("bug", "Bugs", "issues debug"),
    choice("wrench", "Tools", "fix settings repair"),
    choice("hammer", "Build", "construct make"),
    choice("rocket", "Project", "launch startup"),
    choice("lightbulb", "Ideas", "idea thought"),
    choice("target", "Goals", "aim objective"),
    choice("flag", "Milestones", "goal marker"),
    choice("trophy", "Wins", "award achievement"),
    choice("chart-line", "Metrics", "stats analytics graph"),
    choice("chart-pie", "Reports", "stats analytics"),
    choice("presentation", "Presentations", "slides talk"),
    choice("clipboard-list", "Tasks", "todo checklist"),
    choice("list-checks", "Checklists", "todo done"),
    choice("calendar", "Calendar", "date schedule"),
    choice("calendar-days", "Plans", "schedule week"),
    choice("clock", "Time", "hours schedule"),
    choice("alarm-clock", "Reminders", "alarm wake"),
    choice("hourglass", "Waiting", "later time"),
    choice("graduation-cap", "Study", "school learning education"),
    choice("school", "School", "education class"),
    choice("flask-conical", "Research", "science lab experiment"),
    choice("microscope", "Science", "research lab"),
    choice("atom", "Physics", "science atom"),
    choice("calculator", "Maths", "numbers calculate"),
    choice("languages", "Languages", "translate words"),
    choice("palette", "Design", "art colour"),
    choice("pen-tool", "Drawing", "vector design"),
    choice("brush", "Painting", "art paint"),
    choice("pencil", "Writing", "draft edit"),
    choice("feather", "Poetry", "writing quill"),
    choice("image", "Images", "pictures photos"),
    choice("camera", "Photos", "photography pictures"),
    choice("film", "Film", "movies video"),
    choice("clapperboard", "Production", "video movie"),
    choice("mic", "Podcast", "audio recording voice"),
    choice("music", "Music", "songs audio"),
    choice("headphones", "Listening", "audio music"),
    choice("guitar", "Guitar", "music instrument"),
    choice("gamepad-2", "Games", "gaming play"),
    choice("dices", "Board games", "dice games"),
    choice("puzzle", "Puzzles", "games pieces"),
    choice("heart", "Favourites", "love liked"),
    choice("star", "Starred", "favourite important"),
    choice("sparkles", "Inspiration", "magic ideas"),
    choice("gift", "Gifts", "presents birthday"),
    choice("party-popper", "Celebrations", "party events"),
    choice("house", "Home", "house family"),
    choice("bed", "Sleep", "rest bedroom"),
    choice("sofa", "Living", "home furniture"),
    choice("key", "Keys", "access passwords"),
    choice("lock", "Locked", "private secret"),
    choice("shield", "Private", "security safe"),
    choice("plane", "Travel", "trips flight"),
    choice("map", "Maps", "places directions"),
    choice("map-pin", "Places", "location address"),
    choice("compass", "Explore", "navigation direction"),
    choice("mountain", "Outdoors", "hiking nature"),
    choice("tent", "Camping", "outdoors trip"),
    choice("car", "Car", "driving vehicle"),
    choice("bike", "Cycling", "bicycle ride"),
    choice("truck", "Logistics", "delivery shipping fleet"),
    choice("bus", "Transport", "commute bus"),
    choice("train-front", "Trains", "rail commute"),
    choice("ship", "Shipping", "boat sea"),
    choice("globe", "World", "international web"),
    choice("wallet", "Money", "finance budget"),
    choice("piggy-bank", "Savings", "money budget"),
    choice("coins", "Coins", "money cash"),
    choice("credit-card", "Payments", "card money"),
    choice("receipt", "Receipts", "expenses bills"),
    choice("banknote", "Cash", "money bills"),
    choice("shopping-cart", "Shopping", "groceries buy"),
    choice("shopping-bag", "Purchases", "shopping buy"),
    choice("store", "Store", "shop retail"),
    choice("tag", "Labels", "tag price"),
    choice("utensils", "Food", "recipes meals cooking"),
    choice("chef-hat", "Cooking", "recipes kitchen"),
    choice("coffee", "Coffee", "cafe drink"),
    choice("cake", "Baking", "dessert birthday"),
    choice("apple", "Health food", "fruit nutrition"),
    choice("dumbbell", "Fitness", "gym workout"),
    choice("heart-pulse", "Health", "medical wellbeing"),
    choice("stethoscope", "Medical", "doctor health"),
    choice("pill", "Medicine", "health pharmacy"),
    choice("baby", "Baby", "kids family"),
    choice("dog", "Pets", "dog animals"),
    choice("cat", "Cat", "pets animals"),
    choice("paw-print", "Animals", "pets"),
    choice("leaf", "Nature", "plants green"),
    choice("flower", "Garden", "flowers plants"),
    choice("trees", "Forest", "trees nature"),
    choice("sun", "Summer", "sunny weather"),
    choice("moon", "Night", "dreams evening"),
    choice("cloud-rain", "Weather", "rain forecast"),
    choice("snowflake", "Winter", "snow cold"),
    choice("flame", "Hot", "fire trending"),
    choice("zap", "Energy", "power fast"),
    choice("droplet", "Water", "drop liquid"),
    choice("church", "Faith", "church religion"),
    choice("hand-heart", "Charity", "giving care volunteer"),
    choice("newspaper", "News", "articles press"),
    choice("rss", "Feeds", "blogs subscriptions"),
    choice("link", "Links", "bookmarks web"),
    choice("message-circle", "Chats", "messages talk"),
    choice("phone", "Calls", "phone contacts"),
    choice("video", "Meetings", "video calls"),
    choice("megaphone", "Marketing", "announcements promo"),
    choice("lock-keyhole", "Vault", "secure private"),
    choice("eye", "Watching", "watchlist view"),
    choice("infinity", "Ongoing", "forever loop"),
];

/// The choice named `name`, if it is one.
pub fn find(name: &str) -> Option<&'static Choice> {
    CHOICES.iter().find(|choice| choice.name == name)
}

/// Whether `name` can name a theme's folder or one of its icons: lowercase
/// ASCII letters, digits, dashes and underscores, starting with a letter or
/// digit, at most 64 long. Pure.
pub fn is_valid_name(name: &str) -> bool {
    name.len() <= 64
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// `<theme>/<icon>` as its two names, if it has that shape. Pure.
pub fn split_key(key: &str) -> Option<(&str, &str)> {
    let (theme, icon) = key.split_once('/')?;
    (is_valid_name(theme) && is_valid_name(icon)).then_some((theme, icon))
}

/// The icon a notebook keeps, given what its icon field holds: a built-in
/// choice's name, or `<theme>/<icon>` for an icon theme's, whether or not
/// that theme is in use (an icon pack's from before icon themes is kept the
/// same way). Anything else, or no value, is no icon. Pure.
pub fn chosen_name(kept: Option<&str>) -> Option<String> {
    let name = kept?;
    (find(name).is_some() || split_key(name).is_some()).then(|| name.to_owned())
}

/// The icon `book` keeps, as kept: see [`chosen_name`].
pub fn chosen(book: &Book) -> Option<String> {
    chosen_name(book.extra.get(BOOK_ICON_FIELD).and_then(|v| v.as_str()))
}

/// The job `book` has, if any.
pub fn role(book: &Book) -> Option<Role> {
    Role::of_book(
        &book.id,
        book.extra.get(ROLE_FIELD).and_then(|value| value.as_str()),
    )
}

/// The words of `query`, lowercased, split at spaces and dashes.
pub fn words(query: &str) -> Vec<String> {
    query
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether every word of `query` starts a word of `label`, `name` or
/// `keywords`, ignoring case. No words match everything. Pure.
pub fn matches(query: &[String], label: &str, name: &str, keywords: &str) -> bool {
    let text = format!("{label} {name} {keywords}");
    let theirs = words(&text);
    query
        .iter()
        .all(|word| theirs.iter().any(|their| their.starts_with(word.as_str())))
}

/// The built-in choices a search for `query` finds, in the picker's order:
/// see [`matches`]. Pure.
pub fn matching(query: &str) -> Vec<&'static Choice> {
    let query = words(query);
    CHOICES
        .iter()
        .filter(|choice| matches(&query, choice.label, choice.name, choice.keywords))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    use super::*;

    fn book(icon: serde_json::Value) -> Book {
        serde_json::from_value(json!({"_id": "book:a", "name": "A", "kansoIcon": icon})).unwrap()
    }

    fn names(query: &str) -> Vec<&'static str> {
        matching(query)
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }

    #[test]
    fn every_choice_is_named_once_and_the_default_comes_first() {
        let mut names = HashSet::new();
        let mut labels = HashSet::new();
        for choice in CHOICES {
            assert!(names.insert(choice.name), "{} twice", choice.name);
            assert!(labels.insert(choice.label), "label {} twice", choice.label);
            assert!(is_valid_name(choice.name), "{}", choice.name);
        }
        assert!(CHOICES.len() >= 120, "{}", CHOICES.len());
        assert_eq!(CHOICES[0].name, DEFAULT_NAME);
    }

    #[test]
    fn a_notebook_keeps_a_choice_a_theme_icon_or_nothing() {
        assert_eq!(chosen(&book(json!("rocket"))).as_deref(), Some("rocket"));
        for value in [json!("not-an-icon"), json!(3), json!(null), json!("../x/y")] {
            assert_eq!(chosen(&book(value)), None);
        }
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(chosen(&plain), None);
        // A theme's icon is kept whether or not the theme is in use, and so
        // is an old icon pack's.
        assert_eq!(
            chosen(&book(json!("phosphor/planet"))).as_deref(),
            Some("phosphor/planet")
        );
        assert_eq!(
            chosen_name(Some("sketch/wave")).as_deref(),
            Some("sketch/wave")
        );
        assert_eq!(chosen_name(Some("a/b/c")), None);
        assert_eq!(chosen_name(None), None);
    }

    #[test]
    fn names_are_lowercase_words_of_a_limited_length() {
        assert!(is_valid_name("phosphor"));
        assert!(is_valid_name("my-icons_2"));
        assert!(is_valid_name(&"a".repeat(64)));
        for bad in ["", "-a", "_a", "A", "a b", "a/b", "a.b", &"a".repeat(65)] {
            assert!(!is_valid_name(bad), "{bad}");
        }
        assert_eq!(split_key("phosphor/planet"), Some(("phosphor", "planet")));
        assert_eq!(split_key("planet"), None);
        assert_eq!(split_key("a/Planet"), None);
    }

    #[test]
    fn a_notebook_with_a_job_has_its_role() {
        let inbox: Book =
            serde_json::from_value(json!({"_id": "book:inbox", "name": "Inbox"})).unwrap();
        assert_eq!(role(&inbox), Some(Role::Inbox));
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(role(&plain), None);
    }

    #[test]
    fn search_finds_icons_by_label_name_and_keyword() {
        assert_eq!(matching("").len(), CHOICES.len());
        assert_eq!(names("journal").first().copied(), Some("notebook"));
        assert!(names("rocket").contains(&"rocket"));
        assert!(names("zzzzzz").is_empty());
        assert_eq!(words("Book-open_x  Y"), vec!["book", "open", "x", "y"]);
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: whatever is typed, the icons found keep the picker's
        /// order, and typing more words never finds more.
        #[test]
        fn prop_more_words_find_fewer_icons(first in "[a-z]{0,4}", second in "[a-z]{0,4}") {
            let one = names(&first);
            let both = names(&format!("{first} {second}"));
            prop_assert!(both.iter().all(|name| one.contains(name)));
            let order: Vec<usize> = one
                .iter()
                .map(|name| CHOICES.iter().position(|c| c.name == *name).unwrap())
                .collect();
            prop_assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
        }

        /// A name is kept exactly when it is a choice or a valid
        /// `<theme>/<icon>`, and it is kept as it was.
        #[test]
        fn prop_a_kept_name_is_a_choice_or_a_theme_key(text in "[a-z0-9/_-]{0,12}") {
            match chosen_name(Some(&text)) {
                Some(kept) => {
                    prop_assert_eq!(&kept, &text);
                    prop_assert!(find(&text).is_some() || split_key(&text).is_some());
                }
                None => prop_assert!(find(&text).is_none() && split_key(&text).is_none()),
            }
        }
    }
}
