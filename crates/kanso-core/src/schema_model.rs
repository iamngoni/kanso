//! What a notebook's property list is made of. The rules that read, check
//! and merge it are in [`crate::schema`].

/// The notebook field that holds the property list.
pub const FIELD: &str = "kansoProperties";

/// The notebook field that holds the property its board is grouped by.
pub const BOARD_FIELD: &str = "kansoBoard";

/// Most lanes a notebook's board can name.
pub const MAX_LANES: usize = 200;

/// Most properties a notebook can define.
pub const MAX_PROPERTIES: usize = 100;
/// Most options one select property can have.
pub const MAX_OPTIONS: usize = 200;

/// A property's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyType {
    Text,
    Number,
    /// Yes/No.
    Bool,
    Date,
    /// A local wall-clock date and time.
    DateTime,
    /// A date and time with an IANA zone (an embargo).
    DateTimeZone,
    /// One of the property's options.
    Select,
    List,
    /// A shared notebook's member, by username.
    Person,
}

impl PropertyType {
    pub const ALL: [PropertyType; 9] = [
        PropertyType::Text,
        PropertyType::Number,
        PropertyType::Bool,
        PropertyType::Date,
        PropertyType::DateTime,
        PropertyType::DateTimeZone,
        PropertyType::Select,
        PropertyType::List,
        PropertyType::Person,
    ];

    /// The `type` a property list stores.
    pub fn as_str(self) -> &'static str {
        match self {
            PropertyType::Text => "text",
            PropertyType::Number => "number",
            PropertyType::Bool => "bool",
            PropertyType::Date => "date",
            PropertyType::DateTime => "datetime",
            PropertyType::DateTimeZone => "datetimeZone",
            PropertyType::Select => "select",
            PropertyType::List => "list",
            PropertyType::Person => "person",
        }
    }

    pub fn parse(text: &str) -> Option<PropertyType> {
        PropertyType::ALL.into_iter().find(|t| t.as_str() == text)
    }
}

/// One property a notebook defines: `{"key": "stage", "type": "select",
/// "options": ["Idea", "Scripting"]}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyDef {
    /// The front-matter key.
    pub key: String,
    pub kind: PropertyType,
    /// The choices of a select property; empty for every other type.
    pub options: Vec<String>,
}

/// Why a property's value does not fit its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mismatch {
    NotText,
    NotANumber,
    NotYesNo,
    NotADate,
    NotADateTime,
    NotADateTimeZone,
    /// Not one of the select's options.
    NotAnOption {
        key: String,
    },
    NotAList,
}

impl Mismatch {
    /// The warning shown beside the value.
    pub fn message(&self) -> String {
        match self {
            Mismatch::NotText => "Not text".into(),
            Mismatch::NotANumber => "Not a number".into(),
            Mismatch::NotYesNo => "Not yes or no".into(),
            Mismatch::NotADate => "Not a date".into(),
            Mismatch::NotADateTime => "Not a date and time".into(),
            Mismatch::NotADateTimeZone => "Not a date and time with a zone".into(),
            Mismatch::NotAnOption { key } => format!("Not one of this notebook's {key}s"),
            Mismatch::NotAList => "Not a list".into(),
        }
    }
}

/// Why a property list is refused when it is saved.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    #[error("A property needs a name.")]
    EmptyKey,
    #[error("The property '{0}' is defined twice.")]
    DuplicateKey(String),
    #[error("A notebook can define at most {MAX_PROPERTIES} properties.")]
    TooManyProperties,
    #[error("The property '{0}' has too many options.")]
    TooManyOptions(String),
    #[error("The property '{0}' lists the option '{1}' twice.")]
    DuplicateOption(String, String),
    #[error("The property '{0}' has an empty option.")]
    EmptyOption(String),
    #[error("Only a select property has options ('{0}').")]
    OptionsOnNonSelect(String),
    #[error("The notebook's properties are not a list of definitions.")]
    NotAList,
}

/// How a notebook's board is laid out (`kansoBoard`): the property its
/// notes are grouped by, and the lanes it names. Saved with the notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardSpec {
    /// The property (a front-matter key) the lanes are values of.
    pub by: String,
    /// The lanes, left to right. Empty means the property's select options
    /// in their order.
    pub lanes: Vec<String>,
    /// Whether the lane of notes with no value shows even when it is empty.
    pub show_unset: bool,
}
