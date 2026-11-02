//! Holding a plugin's panel tree to its caps before Kanso draws it.
//!
//! The tree comes from a plugin, so it is untrusted: capped in bytes, nodes,
//! depth and string length; every ID that comes back in a press is
//! well-formed and unique; links are plain HTTPS to a named host. Text is
//! never interpreted: there is no markup to inject.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::panel_model::{MAX_DEPTH, MAX_NODES, MAX_OPTIONS, MAX_STRING, Node};

/// The largest panel tree, serialised.
pub const MAX_TREE_BYTES: usize = 256 * 1024;
/// The widest gap a row or column may ask for.
const MAX_GAP: f32 = 32.0;
/// How deep a raw tree may nest before it is even read; well past
/// [`MAX_DEPTH`], so a hostile tree can't overflow the stack.
const SCAN_DEPTH: usize = 64;

/// Why a panel tree was refused.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PanelError {
    #[error("the panel is larger than {} KiB", MAX_TREE_BYTES / 1024)]
    TooLarge,
    #[error("the panel can't be read: {0}")]
    Malformed(String),
    #[error("the panel has more than {MAX_NODES} parts")]
    TooManyNodes,
    #[error("the panel nests more than {MAX_DEPTH} levels deep")]
    TooDeep,
    #[error("the panel has text longer than {} KiB", MAX_STRING / 1024)]
    StringTooLong,
    #[error("{0:?} isn't a valid ID for a button or field")]
    BadId(String),
    #[error("the ID {0:?} is used twice")]
    DuplicateId(String),
    #[error("{0:?} isn't an https link to a named site")]
    BadLink(String),
    #[error("a list may hold only text and rows")]
    BadList,
    #[error("a choice has too many options, or a value that isn't one of them")]
    TooManyOptions,
    #[error("a number in the panel isn't a usable number")]
    BadNumber,
}

/// Reads `tree` as a panel and holds it to the caps. The returned tree has
/// progress values clamped into 0 to 1 and gaps into 0 to 32. Pure.
pub fn check(tree: &Value) -> Result<Node, PanelError> {
    if raw_depth(tree) > SCAN_DEPTH {
        return Err(PanelError::TooDeep);
    }
    let size = serde_json::to_vec(tree)
        .map_err(|e| PanelError::Malformed(e.to_string()))?
        .len();
    if size > MAX_TREE_BYTES {
        return Err(PanelError::TooLarge);
    }
    let mut node: Node =
        serde_json::from_value(tree.clone()).map_err(|e| PanelError::Malformed(e.to_string()))?;
    let mut state = State {
        nodes: 0,
        ids: BTreeSet::new(),
    };
    walk(&mut node, 1, &mut state)?;
    Ok(node)
}

/// The deepest nesting of arrays and objects in `value`, without recursion.
fn raw_depth(value: &Value) -> usize {
    let mut deepest = 0;
    let mut stack = vec![(value, 1usize)];
    while let Some((v, depth)) = stack.pop() {
        match v {
            Value::Array(items) => {
                deepest = deepest.max(depth);
                stack.extend(items.iter().map(|i| (i, depth + 1)));
            }
            Value::Object(map) => {
                deepest = deepest.max(depth);
                stack.extend(map.values().map(|i| (i, depth + 1)));
            }
            _ => {}
        }
    }
    deepest
}

struct State {
    nodes: usize,
    ids: BTreeSet<String>,
}

fn short(s: &str) -> Result<(), PanelError> {
    if s.len() > MAX_STRING {
        Err(PanelError::StringTooLong)
    } else {
        Ok(())
    }
}

fn claim_id(id: &str, state: &mut State) -> Result<(), PanelError> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if !ok {
        return Err(PanelError::BadId(id.chars().take(64).collect()));
    }
    if !state.ids.insert(id.to_owned()) {
        return Err(PanelError::DuplicateId(id.to_owned()));
    }
    Ok(())
}

fn gap(gap: &mut Option<f32>) -> Result<(), PanelError> {
    if let Some(g) = gap {
        if !g.is_finite() {
            return Err(PanelError::BadNumber);
        }
        *g = g.clamp(0.0, MAX_GAP);
    }
    Ok(())
}

fn check_link(label: &str, url: &str) -> Result<(), PanelError> {
    short(label)?;
    short(url)?;
    let bad = || PanelError::BadLink(url.chars().take(200).collect());
    let parsed = url::Url::parse(url).map_err(|_| bad())?;
    let named = matches!(parsed.host(), Some(url::Host::Domain(_)));
    if parsed.scheme() != "https"
        || !named
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(bad());
    }
    Ok(())
}

fn walk(node: &mut Node, depth: usize, state: &mut State) -> Result<(), PanelError> {
    state.nodes += 1;
    if state.nodes > MAX_NODES {
        return Err(PanelError::TooManyNodes);
    }
    if depth > MAX_DEPTH {
        return Err(PanelError::TooDeep);
    }
    match node {
        Node::Text { value, .. } | Node::Heading { value } | Node::Badge { value } => short(value),
        Node::Row { children, gap: g } | Node::Column { children, gap: g } => {
            gap(g)?;
            children
                .iter_mut()
                .try_for_each(|c| walk(c, depth + 1, state))
        }
        Node::List { items } => {
            if !items
                .iter()
                .all(|i| matches!(i, Node::Text { .. } | Node::Row { .. }))
            {
                return Err(PanelError::BadList);
            }
            items.iter_mut().try_for_each(|c| walk(c, depth + 1, state))
        }
        Node::Button { id, label, .. } => {
            claim_id(id, state)?;
            short(label)
        }
        Node::Input {
            id,
            label,
            value,
            placeholder,
        } => {
            claim_id(id, state)?;
            short(label)?;
            short(value)?;
            short(placeholder)
        }
        Node::Toggle { id, label, .. } => {
            claim_id(id, state)?;
            short(label)
        }
        Node::Select {
            id,
            label,
            options,
            value,
        } => {
            claim_id(id, state)?;
            short(label)?;
            if options.len() > MAX_OPTIONS {
                return Err(PanelError::TooManyOptions);
            }
            options.iter().try_for_each(|o| short(o))?;
            if let Some(v) = value
                && !options.contains(v)
            {
                return Err(PanelError::TooManyOptions);
            }
            Ok(())
        }
        Node::Image { attachment_id } => short(attachment_id),
        Node::Link { label, url } => check_link(label, url),
        Node::Progress { value } => {
            if !value.is_finite() {
                return Err(PanelError::BadNumber);
            }
            *value = value.clamp(0.0, 1.0);
            Ok(())
        }
        Node::Divider => Ok(()),
    }
}

/// Every child of `node`.
fn children(node: &Node) -> &[Node] {
    match node {
        Node::Row { children, .. } | Node::Column { children, .. } => children,
        Node::List { items } => items,
        _ => &[],
    }
}

/// Visits `node` and everything under it, in order. For trees already
/// checked: their depth is bounded.
fn visit<'a>(node: &'a Node, f: &mut impl FnMut(&'a Node)) {
    f(node);
    for child in children(node) {
        visit(child, f);
    }
}

/// The attachment IDs of every image in the tree; the router checks that
/// each is one the plugin may read. Pure.
pub fn images(node: &Node) -> Vec<&str> {
    let mut out = Vec::new();
    visit(node, &mut |n| {
        if let Node::Image { attachment_id } = n {
            out.push(attachment_id.as_str());
        }
    });
    out
}

/// The IDs of every input, toggle and select: the ones a press may carry
/// a value for. Pure.
pub fn inputs(node: &Node) -> Vec<&str> {
    let mut out = Vec::new();
    visit(node, &mut |n| {
        if let Node::Input { id, .. } | Node::Toggle { id, .. } | Node::Select { id, .. } = n {
            out.push(id.as_str());
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel_model::{TextStyle, Tone};
    use proptest::prelude::*;
    use serde_json::json;

    fn text(s: &str) -> Node {
        Node::Text {
            value: s.into(),
            style: TextStyle::Body,
        }
    }

    fn column(children: Vec<Node>) -> Node {
        Node::Column {
            children,
            gap: None,
        }
    }

    fn ok(node: &Node) -> Result<Node, PanelError> {
        check(&serde_json::to_value(node).unwrap())
    }

    #[test]
    fn a_plain_panel_passes() {
        let tree = json!({"type":"column","gap":8,"children":[
            {"type":"heading","value":"Clips"},
            {"type":"input","id":"q","label":"Search"},
            {"type":"button","id":"go","label":"Go","tone":"primary"},
            {"type":"link","label":"Docs","url":"https://docs.example/x"},
            {"type":"image","attachmentId":"abc"},
            {"type":"divider"}]});
        let node = check(&tree).unwrap();
        assert_eq!(inputs(&node), ["q"]);
        assert_eq!(images(&node), ["abc"]);
    }

    #[test]
    fn unknown_things_are_refused() {
        for tree in [
            json!({"type":"iframe","src":"x"}),
            json!({"type":"text","value":"a","onclick":"x"}),
            json!({"type":"text"}),
            json!(5),
        ] {
            assert!(
                matches!(check(&tree), Err(PanelError::Malformed(_))),
                "{tree}"
            );
        }
    }

    #[test]
    fn caps_are_enforced() {
        let wide = column(vec![text("a"); MAX_NODES]);
        assert_eq!(ok(&wide), Err(PanelError::TooManyNodes));
        assert!(ok(&column(vec![text("a"); MAX_NODES - 1])).is_ok());
        let mut deep = text("a");
        for _ in 0..MAX_DEPTH - 1 {
            deep = column(vec![deep]);
        }
        assert!(ok(&deep).is_ok());
        assert_eq!(ok(&column(vec![deep])), Err(PanelError::TooDeep));
        assert_eq!(
            ok(&text(&"x".repeat(MAX_STRING + 1))),
            Err(PanelError::StringTooLong)
        );
        assert!(ok(&text(&"x".repeat(MAX_STRING))).is_ok());
        let big = column(vec![text(&"x".repeat(MAX_STRING)); 40]);
        assert_eq!(ok(&big), Err(PanelError::TooLarge));
        let mut hostile = json!("x");
        for _ in 0..100 {
            hostile = json!([hostile]);
        }
        assert_eq!(check(&hostile), Err(PanelError::TooDeep));
    }

    #[test]
    fn ids_are_well_formed_and_unique() {
        let button = |id: &str| Node::Button {
            id: id.into(),
            label: "b".into(),
            tone: Tone::Normal,
        };
        assert!(ok(&button("go-1_x")).is_ok());
        for bad in ["", "a b", "a/b", &"x".repeat(65), "é"] {
            assert!(
                matches!(ok(&button(bad)), Err(PanelError::BadId(_))),
                "{bad}"
            );
        }
        let two = column(vec![
            button("a"),
            Node::Toggle {
                id: "a".into(),
                label: "t".into(),
                value: false,
            },
        ]);
        assert_eq!(ok(&two), Err(PanelError::DuplicateId("a".into())));
    }

    #[test]
    fn links_are_plain_https() {
        let link = |u: &str| Node::Link {
            label: "l".into(),
            url: u.into(),
        };
        assert!(ok(&link("https://example.com/a?b=c")).is_ok());
        for bad in [
            "http://example.com",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://u:p@example.com/",
            "https://u@example.com/",
            "example.com",
            "data:text/html,x",
            "kanso://x",
        ] {
            assert!(
                matches!(ok(&link(bad)), Err(PanelError::BadLink(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn lists_and_selects_are_constrained() {
        let list = Node::List {
            items: vec![
                text("a"),
                Node::Row {
                    children: vec![],
                    gap: None,
                },
            ],
        };
        assert!(ok(&list).is_ok());
        let bad = Node::List {
            items: vec![Node::Divider],
        };
        assert_eq!(ok(&bad), Err(PanelError::BadList));
        let select = |options: Vec<String>, value: Option<&str>| Node::Select {
            id: "s".into(),
            label: "s".into(),
            options,
            value: value.map(str::to_owned),
        };
        assert!(ok(&select(vec!["a".into()], Some("a"))).is_ok());
        assert_eq!(
            ok(&select(vec!["a".into()], Some("b"))),
            Err(PanelError::TooManyOptions)
        );
        let many = (0..=MAX_OPTIONS).map(|i| i.to_string()).collect();
        assert_eq!(ok(&select(many, None)), Err(PanelError::TooManyOptions));
    }

    #[test]
    fn numbers_are_clamped_or_refused() {
        let Node::Progress { value } = ok(&Node::Progress { value: 7.0 }).unwrap() else {
            panic!()
        };
        assert_eq!(value, 1.0);
        let Node::Progress { value } = ok(&Node::Progress { value: -3.0 }).unwrap() else {
            panic!()
        };
        assert_eq!(value, 0.0);
        let Node::Row { gap, .. } = ok(&Node::Row {
            children: vec![],
            gap: Some(900.0),
        })
        .unwrap() else {
            panic!()
        };
        assert_eq!(gap, Some(32.0));
        // 1e40 doesn't fit an f32.
        assert_eq!(
            check(&json!({"type":"progress","value":1e40})),
            Err(PanelError::BadNumber)
        );
    }

    #[test]
    fn markup_stays_text() {
        let node = ok(&text("<script>alert(1)</script> **bold**")).unwrap();
        assert_eq!(node, text("<script>alert(1)</script> **bold**"));
    }

    fn arb_node() -> impl Strategy<Value = Node> {
        let id = "[a-c]";
        let leaf = prop_oneof![
            "[a-z]{0,6}".prop_map(|v| text(&v)),
            id.prop_map(|id| Node::Button {
                id,
                label: "b".into(),
                tone: Tone::Normal
            }),
            id.prop_map(|id| Node::Toggle {
                id,
                label: "t".into(),
                value: true
            }),
            id.prop_map(|id| Node::Input {
                id,
                label: "i".into(),
                value: String::new(),
                placeholder: String::new()
            }),
            prop_oneof![
                Just("https://example.com/x"),
                Just("http://example.com"),
                Just("javascript:x"),
                Just("https://1.2.3.4/"),
                Just("https://u:p@a.com/")
            ]
            .prop_map(|u| Node::Link {
                label: "l".into(),
                url: u.into()
            }),
            (-5.0f32..5.0).prop_map(|value| Node::Progress { value }),
            Just(Node::Divider),
            "[a-z]{0,3}".prop_map(|v| Node::Badge { value: v }),
            "[a-z]{0,4}".prop_map(|a| Node::Image { attachment_id: a }),
        ];
        leaf.prop_recursive(12, 700, 8, |inner| {
            prop_oneof![
                proptest::collection::vec(inner.clone(), 0..8).prop_map(|children| Node::Column {
                    children,
                    gap: Some(50.0)
                }),
                proptest::collection::vec(inner.clone(), 0..8).prop_map(|children| Node::Row {
                    children,
                    gap: None
                }),
                proptest::collection::vec(inner, 0..8).prop_map(|items| Node::List { items }),
            ]
        })
    }

    fn depth(n: &Node) -> usize {
        1 + children(n).iter().map(depth).max().unwrap_or(0)
    }

    fn count(n: &Node) -> usize {
        1 + children(n).iter().map(count).sum::<usize>()
    }

    fn all_links(n: &Node, out: &mut Vec<String>) {
        visit(n, &mut |n| {
            if let Node::Link { url, .. } = n {
                out.push(url.clone());
            }
        });
    }

    proptest! {
        /// Invariant: any accepted tree is within the node and depth caps,
        /// has unique IDs, only https links and in-range numbers.
        #[test]
        fn prop_accepted_trees_hold_the_caps(node in arb_node()) {
            let Ok(checked) = ok(&node) else { return Ok(()); };
            prop_assert!(count(&checked) <= MAX_NODES);
            prop_assert!(depth(&checked) <= MAX_DEPTH);
            let mut ids = Vec::new();
            visit(&checked, &mut |n| {
                if let Node::Button { id, .. } | Node::Toggle { id, .. } | Node::Input { id, .. } = n {
                    ids.push(id.clone());
                }
                match n {
                    Node::Progress { value } => assert!((0.0..=1.0).contains(value)),
                    Node::Row { gap: Some(g), .. } | Node::Column { gap: Some(g), .. } => {
                        assert!((0.0..=32.0).contains(g));
                    }
                    Node::List { items } => assert!(items.iter().all(|i| matches!(i, Node::Text { .. } | Node::Row { .. }))),
                    _ => {}
                }
            });
            let unique: BTreeSet<_> = ids.iter().collect();
            prop_assert_eq!(unique.len(), ids.len());
            let mut links = Vec::new();
            all_links(&checked, &mut links);
            prop_assert!(links.iter().all(|l| l.starts_with("https://")));
        }

        /// Invariant: a tree over the node cap or the depth cap is refused.
        #[test]
        fn prop_trees_over_the_caps_are_refused(node in arb_node()) {
            if count(&node) > MAX_NODES || depth(&node) > MAX_DEPTH {
                prop_assert!(ok(&node).is_err());
            }
        }

        /// Invariant: a string over the cap anywhere is refused.
        #[test]
        fn prop_long_strings_are_refused(extra in 1usize..50, slot in 0usize..3) {
            let long = "x".repeat(MAX_STRING + extra);
            let node = match slot {
                0 => text(&long),
                1 => Node::Button { id: "a".into(), label: long, tone: Tone::Normal },
                _ => Node::Image { attachment_id: long },
            };
            prop_assert_eq!(ok(&node), Err(PanelError::StringTooLong));
        }

        /// Invariant: every accepted input ID is reported by `inputs` and
        /// every image by `images`.
        #[test]
        fn prop_inputs_and_images_are_found(node in arb_node()) {
            let Ok(checked) = ok(&node) else { return Ok(()); };
            let mut want_inputs = 0;
            let mut want_images = 0;
            visit(&checked, &mut |n| match n {
                Node::Input { .. } | Node::Toggle { .. } | Node::Select { .. } => want_inputs += 1,
                Node::Image { .. } => want_images += 1,
                _ => {}
            });
            prop_assert_eq!(inputs(&checked).len(), want_inputs);
            prop_assert_eq!(images(&checked).len(), want_images);
        }
    }
}
