//! One plugin's sandboxed JavaScript runtime.
//!
//! A [`Plugin`] owns a QuickJS runtime and context that hold only the
//! standard JavaScript builtins and a frozen `kanso` object (`api.js`).
//! There is no module loader, console, timer, `require`, `fetch` or `eval`,
//! and the two native functions behind `kanso` are deleted from the global
//! object before the plugin's code runs. Every `kanso.*` call is a message
//! to the app through a [`Link`], held only for the length of one request.
//!
//! Each request has a CPU deadline. The clock stops while the plugin waits
//! for the app, so a slow note read doesn't count against the plugin. Memory
//! and stack are capped by the engine. Renderers are pure: while one runs,
//! every host call is refused here without reaching the [`Link`].

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use rquickjs::{Context, Ctx, Function, Object, Persistent, Runtime};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::protocol::{
    self, Budgets, CODE_CONFLICT, CODE_DENIED, CODE_INVALID, CODE_LIMIT, CODE_NOT_FOUND,
    CODE_THROWN, CODE_TIMEOUT, CODE_UNAVAILABLE, CommandSpec, Failure, MAX_FROM_HOST, PluginInfo,
    Registered, Theme,
};

/// The shim that builds the `kanso` object.
const API: &str = include_str!("api.js");

/// The longest message a failure carries, in characters.
const MAX_MESSAGE: usize = 1_000;
/// The longest log line, in characters.
const MAX_LOG: usize = 8_192;

/// How the runtime reaches the app. The host binary implements it over the
/// pipes; tests implement it with a fake.
pub trait Link {
    /// Makes the host call `call` (one of [`protocol::CALLS`]) and waits
    /// for the app's answer.
    fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure>;
    /// Passes on `kanso.log`. Never answered.
    fn log(&mut self, level: &str, text: &str);
}

/// The CPU clock for requests: a deadline in microseconds since the plugin
/// started, or `u64::MAX` when none is running.
struct Clock {
    started: Instant,
    deadline: Arc<AtomicU64>,
    timed_out: Arc<AtomicBool>,
}

impl Clock {
    fn now(&self) -> u64 {
        self.started.elapsed().as_micros() as u64
    }

    fn arm(&self, millis: u64) {
        self.timed_out.store(false, Ordering::Relaxed);
        let limit = self.now().saturating_add(millis.saturating_mul(1_000));
        self.deadline.store(limit, Ordering::Relaxed);
    }

    fn disarm(&self) {
        self.deadline.store(u64::MAX, Ordering::Relaxed);
    }
}

/// What the native functions share with the plugin that owns them.
struct Shared {
    /// The app's end, set only while a request runs.
    link: Cell<Option<*mut dyn Link>>,
    /// Whether the running request is a renderer.
    rendering: Cell<bool>,
    deadline: Arc<AtomicU64>,
    started: Instant,
}

impl Shared {
    /// `__kansoCall`: one host call, as JSON in and JSON out.
    fn host_call(&self, name: &str, json: &str) -> String {
        let answer = self.try_host_call(name, json);
        let envelope = match answer {
            Ok(value) => serde_json::json!({ "ok": true, "value": value }),
            Err(failure) => {
                serde_json::json!({ "ok": false, "code": failure.code, "message": failure.message })
            }
        };
        envelope.to_string()
    }

    fn try_host_call(&self, name: &str, json: &str) -> Result<Value, Failure> {
        if self.rendering.get() {
            return Err(Failure::new(
                CODE_DENIED,
                format!("a renderer can't use kanso.{name}"),
            ));
        }
        if !protocol::is_call(name) {
            return Err(Failure::new(
                CODE_INVALID,
                "that isn't something kanso can do",
            ));
        }
        if json.len() > MAX_FROM_HOST {
            return Err(Failure::new(CODE_LIMIT, "the arguments are too large"));
        }
        let args: Value = serde_json::from_str(json)
            .map_err(|_| Failure::new(CODE_INVALID, "the arguments aren't JSON"))?;
        let Some(link) = self.link.get() else {
            return Err(Failure::new(CODE_UNAVAILABLE, "the app isn't listening"));
        };
        // The clock stops while the app works.
        let before = self.deadline.swap(u64::MAX, Ordering::Relaxed);
        let paused_at = self.started.elapsed().as_micros() as u64;
        // SAFETY: the pointer is set by `Plugin::exec` from a live
        // `&mut dyn Link` and cleared before that borrow ends; the engine
        // runs on this thread only and a host call never re-enters itself.
        let answer = unsafe { (*link).call(name, args) };
        if before != u64::MAX {
            let left = before.saturating_sub(paused_at);
            let now = self.started.elapsed().as_micros() as u64;
            self.deadline
                .store(now.saturating_add(left), Ordering::Relaxed);
        }
        answer
    }

    /// `__kansoLog`: `kanso.log`, cut to a sane length.
    fn log(&self, level: &str, text: &str) {
        let Some(link) = self.link.get() else { return };
        let text: String = text.chars().take(MAX_LOG).collect();
        // SAFETY: as in `try_host_call`.
        unsafe { (*link).log(level, &text) };
    }
}

/// Sets the link for the length of one request and clears it after.
struct LinkGuard<'a>(&'a Shared);

impl<'a> LinkGuard<'a> {
    fn new(shared: &'a Shared, link: &mut dyn Link, rendering: bool) -> Self {
        let pointer: *mut dyn Link = link;
        // SAFETY: only the lifetime is erased. The guard clears the pointer
        // when it drops, before the borrow it came from ends.
        let pointer: *mut (dyn Link + 'static) = unsafe { std::mem::transmute(pointer) };
        shared.link.set(Some(pointer));
        shared.rendering.set(rendering);
        LinkGuard(shared)
    }
}

impl Drop for LinkGuard<'_> {
    fn drop(&mut self) {
        self.0.link.set(None);
        self.0.rendering.set(false);
    }
}

/// A running plugin: its engine, its limits and what it registered.
pub struct Plugin {
    // Declared first so it drops before the engine it points into.
    internals: Persistent<Object<'static>>,
    context: Context,
    runtime: Runtime,
    shared: Rc<Shared>,
    clock: Clock,
    budgets: Budgets,
    info: PluginInfo,
}

/// What `api.js` answers every request with.
#[derive(Deserialize)]
struct Envelope {
    ok: bool,
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

impl Plugin {
    /// Starts the engine and runs the entry script `code` once, within the
    /// startup budget, then returns what it registered. Fails with
    /// `thrown` if the script doesn't run, `timeout` or `limit` if it
    /// overruns, and `invalid` if what it registered breaks the rules.
    pub fn start(
        info: &PluginInfo,
        code: &str,
        budgets: Budgets,
        link: &mut dyn Link,
    ) -> Result<(Plugin, Registered), Failure> {
        let mut plugin = Plugin::boot(info, budgets, link)?;
        plugin.exec(budgets.startup_ms, false, link, |ctx, internals| {
            ctx.eval::<rquickjs::Value, _>(code)?;
            let seal: Function = internals.get("seal")?;
            seal.call::<_, ()>(())?;
            Ok(())
        })?;
        let json = plugin.exec(budgets.startup_ms, false, link, |_, internals| {
            let registered: Function = internals.get("registered")?;
            registered.call::<_, String>(())
        })?;
        let registered: Registered = serde_json::from_str(&json).map_err(|_| {
            Failure::new(CODE_THROWN, "the plugin's registrations couldn't be read")
        })?;
        check_registered(&registered)?;
        Ok((plugin, registered))
    }

    /// The plugin this runtime belongs to.
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }

    /// Runs the command `command` with `args` and returns what it returned
    /// (`undefined` is `null`). `not_found` if there is no such command.
    pub fn invoke(
        &mut self,
        command: &str,
        args: Value,
        link: &mut dyn Link,
    ) -> Result<Value, Failure> {
        let args = serde_json::to_string(&args)
            .map_err(|_| Failure::new(CODE_INVALID, "the arguments aren't JSON"))?;
        self.dispatch(
            "invoke",
            command,
            &args,
            "",
            self.budgets.command_ms,
            false,
            link,
        )
    }

    /// Draws the fenced block `source` of `language` for `theme`. Host calls
    /// are refused while it runs. `not_found` if no renderer has that
    /// language, `invalid` if it doesn't return a string.
    pub fn render(
        &mut self,
        language: &str,
        source: &str,
        theme: &Theme,
        link: &mut dyn Link,
    ) -> Result<String, Failure> {
        let theme = serde_json::to_string(theme)
            .map_err(|_| Failure::new(CODE_INVALID, "the theme isn't JSON"))?;
        let drawn = self.dispatch(
            "render",
            language,
            source,
            &theme,
            self.budgets.render_ms,
            true,
            link,
        )?;
        match drawn {
            Value::String(svg) => Ok(svg),
            _ => Err(Failure::new(
                CODE_INVALID,
                "a renderer must return a string",
            )),
        }
    }

    /// Calls the press handler for `button` with the panel's `values`.
    /// `not_found` if it has none.
    pub fn press(
        &mut self,
        button: &str,
        values: Map<String, Value>,
        link: &mut dyn Link,
    ) -> Result<Value, Failure> {
        let values = serde_json::to_string(&values)
            .map_err(|_| Failure::new(CODE_INVALID, "the values aren't JSON"))?;
        self.dispatch(
            "press",
            button,
            &values,
            "",
            self.budgets.command_ms,
            false,
            link,
        )
    }

    /// Builds the engine with its limits and the `kanso` object, but runs
    /// none of the plugin's code.
    fn boot(info: &PluginInfo, budgets: Budgets, link: &mut dyn Link) -> Result<Plugin, Failure> {
        let unavailable = |what: &dyn std::fmt::Display| {
            Failure::new(
                CODE_UNAVAILABLE,
                format!("the plugin engine didn't start: {what}"),
            )
        };
        let runtime = Runtime::new().map_err(|e| unavailable(&e))?;
        runtime.set_memory_limit(budgets.memory);
        runtime.set_max_stack_size(budgets.stack);
        let started = Instant::now();
        let deadline = Arc::new(AtomicU64::new(u64::MAX));
        let timed_out = Arc::new(AtomicBool::new(false));
        {
            let deadline = deadline.clone();
            let timed_out = timed_out.clone();
            runtime.set_interrupt_handler(Some(Box::new(move || {
                let over = started.elapsed().as_micros() as u64 > deadline.load(Ordering::Relaxed);
                if over {
                    timed_out.store(true, Ordering::Relaxed);
                }
                over
            })));
        }
        let context = Context::full(&runtime).map_err(|e| unavailable(&e))?;
        let shared = Rc::new(Shared {
            link: Cell::new(None),
            rendering: Cell::new(false),
            deadline: deadline.clone(),
            started,
        });
        let clock = Clock {
            started,
            deadline,
            timed_out,
        };
        clock.arm(budgets.startup_ms);
        let internals = {
            let guard = LinkGuard::new(&shared, link, false);
            let made = context.with(|ctx| -> Result<_, Failure> {
                let describe = |e| describe(&ctx, e, &clock);
                let call = {
                    let shared = shared.clone();
                    Function::new(ctx.clone(), move |name: String, json: String| {
                        shared.host_call(&name, &json)
                    })
                    .map_err(describe)?
                };
                let log = {
                    let shared = shared.clone();
                    Function::new(ctx.clone(), move |level: String, text: String| {
                        shared.log(&level, &text)
                    })
                    .map_err(describe)?
                };
                let globals = ctx.globals();
                globals.set("__kansoCall", call).map_err(describe)?;
                globals.set("__kansoLog", log).map_err(describe)?;
                let internals: Object = ctx.eval(API).map_err(describe)?;
                Ok(Persistent::save(&ctx, internals))
            });
            drop(guard);
            made
        };
        clock.disarm();
        let internals = internals?;
        Ok(Plugin {
            internals,
            context,
            runtime,
            shared,
            clock,
            budgets,
            info: info.clone(),
        })
    }

    /// Asks `api.js` to run one request and reads its answer.
    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &mut self,
        kind: &str,
        name: &str,
        first: &str,
        second: &str,
        millis: u64,
        rendering: bool,
        link: &mut dyn Link,
    ) -> Result<Value, Failure> {
        let answer = self.exec(millis, rendering, link, |_, internals| {
            let dispatch: Function = internals.get("dispatch")?;
            dispatch.call::<_, String>((kind, name, first, second))
        })?;
        let envelope: Envelope = serde_json::from_str(&answer)
            .map_err(|_| Failure::new(CODE_THROWN, "the plugin's answer couldn't be read"))?;
        if !envelope.ok {
            return Err(failure_from(envelope));
        }
        let value = envelope.value.unwrap_or(Value::Null);
        match serde_json::to_vec(&value) {
            Ok(bytes) if bytes.len() <= MAX_FROM_HOST => Ok(value),
            _ => Err(Failure::new(CODE_LIMIT, "its answer was too large")),
        }
    }

    /// Runs `f` inside the engine with the link set, the deadline armed and
    /// the engine's errors turned into failures.
    fn exec<R>(
        &mut self,
        millis: u64,
        rendering: bool,
        link: &mut dyn Link,
        f: impl for<'js> FnOnce(&Ctx<'js>, Object<'js>) -> rquickjs::Result<R>,
    ) -> Result<R, Failure> {
        let guard = LinkGuard::new(&self.shared, link, rendering);
        self.clock.arm(millis);
        let out = self.context.with(|ctx| {
            let internals = self
                .internals
                .clone()
                .restore(&ctx)
                .map_err(|e| describe(&ctx, e, &self.clock))?;
            f(&ctx, internals).map_err(|e| describe(&ctx, e, &self.clock))
        });
        self.clock.disarm();
        drop(guard);
        if matches!(&out, Err(failure) if failure.code == CODE_LIMIT || failure.code == CODE_TIMEOUT)
        {
            self.runtime.run_gc();
        }
        out
    }
}

/// Turns the engine's error into a failure, reading the thrown value if
/// there is one.
fn describe(ctx: &Ctx<'_>, error: rquickjs::Error, clock: &Clock) -> Failure {
    let mut code = CODE_THROWN;
    let (message, out_of_memory) = match error {
        rquickjs::Error::Allocation => (String::new(), true),
        rquickjs::Error::Exception => {
            let thrown = ctx.catch();
            if let Some(carried) = thrown
                .as_object()
                .and_then(|o| o.get::<_, String>("code").ok())
                .and_then(|c| known_code(&c))
            {
                code = carried;
            }
            let message = thrown
                .as_exception()
                .and_then(|e| e.message())
                .or_else(|| thrown.as_string().and_then(|s| s.to_string().ok()))
                .unwrap_or_default();
            let out = is_out_of_memory(&message);
            (message, out)
        }
        other => (other.to_string(), false),
    };
    if clock.timed_out.load(Ordering::Relaxed) {
        return took_too_long();
    }
    if out_of_memory {
        return used_too_much_memory();
    }
    let message = if message.is_empty() {
        "the engine stopped the script".to_owned()
    } else {
        message
    };
    Failure::new(code, cut(&message))
}

fn took_too_long() -> Failure {
    Failure::new(CODE_TIMEOUT, "it took too long")
}

fn used_too_much_memory() -> Failure {
    Failure::new(CODE_LIMIT, "it used too much memory")
}

fn is_out_of_memory(message: &str) -> bool {
    message.to_ascii_lowercase().contains("out of memory")
}

/// The first `MAX_MESSAGE` characters of `message`. Pure.
fn cut(message: &str) -> String {
    message.chars().take(MAX_MESSAGE).collect()
}

/// `code` as one of the protocol's failure codes, if it is one. Pure.
fn known_code(code: &str) -> Option<&'static str> {
    [
        CODE_NOT_FOUND,
        CODE_CONFLICT,
        CODE_DENIED,
        CODE_INVALID,
        CODE_LIMIT,
        CODE_TIMEOUT,
        CODE_UNAVAILABLE,
        CODE_THROWN,
    ]
    .into_iter()
    .find(|known| *known == code)
}

/// A failed envelope as a failure: the thrown message, with a known code
/// kept and any other code read as `thrown`. Pure.
fn failure_from(envelope: Envelope) -> Failure {
    let message = envelope.message.unwrap_or_default();
    let code = envelope
        .code
        .as_deref()
        .and_then(known_code)
        .unwrap_or(CODE_THROWN);
    if code == CODE_THROWN && is_out_of_memory(&message) {
        return used_too_much_memory();
    }
    Failure::new(code, cut(&message))
}

/// Whether `id` is a command ID: 1 to 64 of `a-z`, `0-9` and `-`. Pure.
fn is_command_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Re-checks what `api.js` reported, so the app never trusts the script's
/// own validation alone.
fn check_registered(registered: &Registered) -> Result<(), Failure> {
    let bad = |what: &str| Err(Failure::new(CODE_INVALID, what.to_owned()));
    let mut seen: Vec<&str> = Vec::new();
    for CommandSpec { id, title, .. } in &registered.commands {
        if !is_command_id(id) {
            return bad("a command id is 1 to 64 of a-z, 0-9 and -");
        }
        if !(1..=80).contains(&title.chars().count()) {
            return bad("a command title is 1 to 80 characters");
        }
        if seen.contains(&id.as_str()) {
            return bad("two commands have the same id");
        }
        seen.push(id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::cell::RefCell;

    type Handler = Box<dyn FnMut(&str, Value) -> Result<Value, Failure>>;

    /// An app that answers from a closure and remembers what it was asked.
    struct Fake {
        handler: Handler,
        calls: Rc<RefCell<Vec<(String, Value)>>>,
        logs: Rc<RefCell<Vec<(String, String)>>>,
    }

    impl Fake {
        fn new(handler: impl FnMut(&str, Value) -> Result<Value, Failure> + 'static) -> Fake {
            Fake {
                handler: Box::new(handler),
                calls: Rc::default(),
                logs: Rc::default(),
            }
        }

        fn silent() -> Fake {
            Fake::new(|_, _| Ok(Value::Null))
        }
    }

    impl Link for Fake {
        fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure> {
            self.calls
                .borrow_mut()
                .push((call.to_owned(), args.clone()));
            (self.handler)(call, args)
        }

        fn log(&mut self, level: &str, text: &str) {
            self.logs
                .borrow_mut()
                .push((level.to_owned(), text.to_owned()));
        }
    }

    fn info() -> PluginInfo {
        PluginInfo {
            id: "sample".into(),
            name: "Sample".into(),
            version: "1.0.0".into(),
        }
    }

    fn small() -> Budgets {
        Budgets {
            command_ms: 200,
            render_ms: 200,
            startup_ms: 500,
            memory: 16 << 20,
            stack: 1 << 20,
        }
    }

    fn theme() -> Theme {
        Theme {
            dark: false,
            background: "#ffffff".into(),
            text: "#000000".into(),
            muted: "#666666".into(),
            accent: "#0066ff".into(),
            border: "#cccccc".into(),
            surface: "#f4f4f4".into(),
        }
    }

    fn start(code: &str, link: &mut Fake) -> Result<(Plugin, Registered), Failure> {
        Plugin::start(&info(), code, small(), link)
    }

    fn started(code: &str, link: &mut Fake) -> Plugin {
        start(code, link).unwrap_or_else(|f| panic!("{f:?}")).0
    }

    /// Numbers compare as numbers, since JavaScript has only one kind. Pure.
    fn same(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
            }
            (Value::Object(x), Value::Object(y)) => {
                x.len() == y.len() && x.iter().all(|(k, p)| y.get(k).is_some_and(|q| same(p, q)))
            }
            _ => a == b,
        }
    }

    fn json_value() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            (-(1i64 << 53)..=(1i64 << 53)).prop_map(Value::from),
            // Eighths are short in decimal and exact in binary, so any parser
            // reads them back exactly.
            (-(1i64 << 24)..(1i64 << 24)).prop_map(|n| {
                serde_json::Number::from_f64(n as f64 / 8.0).map_or(Value::Null, Value::Number)
            }),
            "\\PC{0,16}".prop_map(Value::String),
        ];
        leaf.prop_recursive(3, 24, 4, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                prop::collection::btree_map("\\PC{0,6}", inner, 0..4)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    #[test]
    fn a_command_registers_and_runs() {
        let mut link = Fake::new(|call, args| {
            assert_eq!(call, "notes.get");
            assert_eq!(args, serde_json::json!({ "id": "note:a" }));
            Ok(serde_json::json!({ "id": "note:a", "body": "one two three" }))
        });
        let code = r#"
            kanso.commands.register({
                id: 'count-words', title: 'Count words', keywords: ['words'], when: 'note-open',
                run(args) {
                    const note = kanso.notes.get(args.id);
                    return { text: 'Words: ' + note.body.split(' ').length };
                },
            });
        "#;
        let (mut plugin, registered) = start(code, &mut link).unwrap();
        assert_eq!(registered.commands.len(), 1);
        assert_eq!(registered.commands[0].id, "count-words");
        assert_eq!(registered.commands[0].keywords, vec!["words".to_owned()]);
        assert_eq!(registered.commands[0].when, protocol::When::NoteOpen);
        let out = plugin
            .invoke(
                "count-words",
                serde_json::json!({ "id": "note:a" }),
                &mut link,
            )
            .unwrap();
        assert_eq!(out, serde_json::json!({ "text": "Words: 3" }));
        assert_eq!(link.calls.borrow().len(), 1);
    }

    #[test]
    fn what_a_plugin_registers_is_reported() {
        let code = r#"
            kanso.commands.register({ id: 'a', title: 'A', run() {} });
            kanso.fences.register('chart', ({ source, theme }) => '<svg/>');
            kanso.ui.onPress('go', () => 1);
        "#;
        let (_, registered) = start(code, &mut Fake::silent()).unwrap();
        assert_eq!(registered.fences, vec!["chart".to_owned()]);
        assert_eq!(registered.presses, vec!["go".to_owned()]);
        assert_eq!(registered.commands[0].when, protocol::When::Always);
    }

    #[test]
    fn registering_after_start_is_refused() {
        let code = "kanso.commands.register({ id: 'a', title: 'A', run() { kanso.commands.register({ id: 'b', title: 'B', run() {} }); } });";
        let mut link = Fake::silent();
        let mut plugin = started(code, &mut link);
        let err = plugin.invoke("a", Value::Null, &mut link).unwrap_err();
        assert_eq!(err.code, CODE_INVALID);
    }

    #[test]
    fn bad_registrations_fail_the_start() {
        for code in [
            "kanso.commands.register({ id: 'Bad', title: 'T', run() {} })",
            "kanso.commands.register({ id: '', title: 'T', run() {} })",
            "kanso.commands.register({ id: 'a', title: '', run() {} })",
            "kanso.commands.register({ id: 'a', title: 'T' })",
            "kanso.commands.register({ id: 'a', title: 'T', when: 'never', run() {} })",
            "kanso.commands.register({ id: 'a', title: 'T', run() {} }); kanso.commands.register({ id: 'a', title: 'T', run() {} })",
            "kanso.fences.register('has space', () => '')",
            "kanso.ui.onPress('', () => 1)",
        ] {
            let err = start(code, &mut Fake::silent())
                .err()
                .unwrap_or_else(|| panic!("{code}"));
            assert_eq!(err.code, CODE_INVALID, "{code}");
        }
    }

    #[test]
    fn an_unknown_command_button_or_language_is_not_found() {
        let mut link = Fake::silent();
        let mut plugin = started("", &mut link);
        assert_eq!(
            plugin
                .invoke("nope", Value::Null, &mut link)
                .unwrap_err()
                .code,
            CODE_NOT_FOUND
        );
        assert_eq!(
            plugin
                .press("nope", Map::new(), &mut link)
                .unwrap_err()
                .code,
            CODE_NOT_FOUND
        );
        assert_eq!(
            plugin
                .render("nope", "", &theme(), &mut link)
                .unwrap_err()
                .code,
            CODE_NOT_FOUND
        );
    }

    #[test]
    fn a_throw_carries_its_message_and_a_host_failure_keeps_its_code() {
        let mut link =
            Fake::new(|_, _| Err(Failure::new(CODE_CONFLICT, "changed since you read it")));
        let code = r#"
            kanso.commands.register({ id: 'boom', title: 'Boom', run() { throw new Error('it broke'); } });
            kanso.commands.register({ id: 'host', title: 'Host', run() { return kanso.notes.get('x'); } });
            kanso.commands.register({ id: 'text', title: 'Text', run() { throw 'plain'; } });
            kanso.commands.register({ id: 'caught', title: 'Caught', run() {
                try { kanso.notes.get('x'); } catch (e) { return [e.code, e.message]; }
            } });
        "#;
        let mut plugin = started(code, &mut link);
        let boom = plugin.invoke("boom", Value::Null, &mut link).unwrap_err();
        assert_eq!(
            (boom.code.as_str(), boom.message.as_str()),
            (CODE_THROWN, "it broke")
        );
        let host = plugin.invoke("host", Value::Null, &mut link).unwrap_err();
        assert_eq!(
            (host.code.as_str(), host.message.as_str()),
            (CODE_CONFLICT, "changed since you read it")
        );
        let text = plugin.invoke("text", Value::Null, &mut link).unwrap_err();
        assert_eq!(
            (text.code.as_str(), text.message.as_str()),
            (CODE_THROWN, "plain")
        );
        let caught = plugin.invoke("caught", Value::Null, &mut link).unwrap();
        assert_eq!(
            caught,
            serde_json::json!([CODE_CONFLICT, "changed since you read it"])
        );
    }

    #[test]
    fn a_syntax_error_fails_the_start() {
        let err = start("this is not ( javascript", &mut Fake::silent())
            .err()
            .unwrap();
        assert_eq!(err.code, CODE_THROWN);
        assert!(!err.message.is_empty());
    }

    #[test]
    fn an_endless_loop_times_out_and_the_plugin_still_works() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.commands.register({ id: 'spin', title: 'Spin', run() { while (true) {} } });
            kanso.commands.register({ id: 'sneaky', title: 'Sneaky', run() { try { while (true) {} } catch (e) { return 'escaped'; } } });
            kanso.commands.register({ id: 'ok', title: 'Ok', run() { return 1; } });
        "#;
        let mut plugin = started(code, &mut link);
        for command in ["spin", "sneaky"] {
            let began = Instant::now();
            let err = plugin.invoke(command, Value::Null, &mut link).unwrap_err();
            assert_eq!(err.code, CODE_TIMEOUT);
            assert_eq!(err.message, "it took too long");
            assert!(began.elapsed().as_millis() < 2_000);
        }
        assert_eq!(
            plugin.invoke("ok", Value::Null, &mut link).unwrap(),
            serde_json::json!(1)
        );
    }

    #[test]
    fn an_endless_loop_at_start_times_out() {
        let err = start("while (true) {}", &mut Fake::silent()).err().unwrap();
        assert_eq!(err.code, CODE_TIMEOUT);
    }

    #[test]
    fn time_spent_waiting_for_the_app_is_not_counted() {
        let mut link = Fake::new(|_, _| {
            std::thread::sleep(std::time::Duration::from_millis(350));
            Ok(Value::Null)
        });
        let code = "kanso.commands.register({ id: 'slow', title: 'Slow', run() { kanso.notes.get('a'); kanso.notes.get('b'); return 'done'; } });";
        let mut plugin = started(code, &mut link);
        assert_eq!(
            plugin.invoke("slow", Value::Null, &mut link).unwrap(),
            serde_json::json!("done")
        );
    }

    #[test]
    fn running_out_of_memory_is_a_limit_and_the_plugin_still_works() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.commands.register({ id: 'array', title: 'A', run() { return new Array(1e9).fill(1).length; } });
            kanso.commands.register({ id: 'string', title: 'S', run() { const a = []; for (let i = 0; ; i++) a.push('x'.repeat(1000) + i); } });
            kanso.commands.register({ id: 'ok', title: 'Ok', run() { return 1; } });
        "#;
        let mut plugin = started(code, &mut link);
        for command in ["array", "string"] {
            let err = plugin.invoke(command, Value::Null, &mut link).unwrap_err();
            assert_eq!(err.code, CODE_LIMIT, "{command}: {err:?}");
            assert_eq!(err.message, "it used too much memory");
        }
        assert_eq!(
            plugin.invoke("ok", Value::Null, &mut link).unwrap(),
            serde_json::json!(1)
        );
    }

    #[test]
    fn endless_recursion_is_an_error_not_a_crash() {
        let mut link = Fake::silent();
        let code = "function f() { return f() + 1; } kanso.commands.register({ id: 'deep', title: 'D', run() { return f(); } });";
        let mut plugin = started(code, &mut link);
        assert!(plugin.invoke("deep", Value::Null, &mut link).is_err());
    }

    #[test]
    fn a_renderer_cannot_reach_the_app() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.fences.register('bad', () => kanso.fetch('https://example.com'));
            kanso.fences.register('sneaky', () => { try { kanso.notes.list(); } catch (e) {} return kanso.settings.get(); });
            kanso.fences.register('logs', ({ source }) => { kanso.log('info', 'drawing ' + source); return '<svg/>'; });
            kanso.fences.register('number', () => 5);
            kanso.fences.register('echo', ({ source, theme }) => source + theme.accent + (theme.dark ? 'd' : 'l'));
        "#;
        let mut plugin = started(code, &mut link);
        let err = plugin.render("bad", "", &theme(), &mut link).unwrap_err();
        assert_eq!(err.code, CODE_DENIED);
        assert_eq!(err.message, "a renderer can't use kanso.fetch");
        let err = plugin
            .render("sneaky", "", &theme(), &mut link)
            .unwrap_err();
        assert_eq!(err.code, CODE_DENIED);
        assert!(link.calls.borrow().is_empty());
        assert_eq!(
            plugin.render("logs", "x", &theme(), &mut link).unwrap(),
            "<svg/>"
        );
        assert_eq!(
            link.logs.borrow().as_slice(),
            [("info".to_owned(), "drawing x".to_owned())]
        );
        assert_eq!(
            plugin
                .render("number", "", &theme(), &mut link)
                .unwrap_err()
                .code,
            CODE_INVALID
        );
        assert_eq!(
            plugin.render("echo", "s", &theme(), &mut link).unwrap(),
            "s#0066ffl"
        );
    }

    #[test]
    fn host_calls_work_again_after_a_render() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.fences.register('a', () => '<svg/>');
            kanso.commands.register({ id: 'c', title: 'C', run() { return kanso.settings.get(); } });
        "#;
        let mut plugin = started(code, &mut link);
        plugin.render("a", "", &theme(), &mut link).unwrap();
        assert_eq!(
            plugin.invoke("c", Value::Null, &mut link).unwrap(),
            Value::Null
        );
        assert_eq!(link.calls.borrow().len(), 1);
    }

    #[test]
    fn a_press_gets_the_panel_values() {
        let mut link = Fake::silent();
        let mut plugin = started(
            "kanso.ui.onPress('go', (values) => values.name + '!');",
            &mut link,
        );
        let mut values = Map::new();
        values.insert("name".into(), Value::String("Ada".into()));
        assert_eq!(
            plugin.press("go", values, &mut link).unwrap(),
            serde_json::json!("Ada!")
        );
    }

    #[test]
    fn every_call_sends_the_shape_the_protocol_names() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.commands.register({ id: 'all', title: 'All', run() {
                kanso.notes.list({ limit: 5 });
                kanso.notes.get('n');
                kanso.notes.create({ bookId: 'b', title: 't', body: 'x' });
                kanso.notes.update('n', { title: 'u' }, { baseRev: 'r1' });
                kanso.notes.pluginData.get('n');
                kanso.notes.pluginData.set('n', { a: 1 });
                kanso.attachments.get('f');
                kanso.editor.noteId();
                kanso.editor.selection();
                kanso.editor.replaceSelection({ text: 'hi', selection: [0, 1] });
                kanso.ui.panel({ type: 'column' });
                kanso.settings.get();
                kanso.settings.set({ k: 1 });
                kanso.secrets.get('token');
                kanso.fetch('https://example.com', { method: 'POST', headers: { a: 'b' }, body: 'x' });
            } });
        "#;
        let mut plugin = started(code, &mut link);
        plugin.invoke("all", Value::Null, &mut link).unwrap();
        let calls: Vec<(String, Value)> = link.calls.borrow().clone();
        let expected = [
            ("notes.list", serde_json::json!({ "limit": 5 })),
            ("notes.get", serde_json::json!({ "id": "n" })),
            (
                "notes.create",
                serde_json::json!({ "bookId": "b", "title": "t", "body": "x" }),
            ),
            (
                "notes.update",
                serde_json::json!({ "id": "n", "fields": { "title": "u" }, "baseRev": "r1" }),
            ),
            ("notes.pluginData.get", serde_json::json!({ "id": "n" })),
            (
                "notes.pluginData.set",
                serde_json::json!({ "id": "n", "value": { "a": 1 } }),
            ),
            ("attachments.get", serde_json::json!({ "fileId": "f" })),
            ("editor.noteId", serde_json::json!({})),
            ("editor.selection", serde_json::json!({})),
            (
                "editor.replaceSelection",
                serde_json::json!({ "text": "hi", "selection": [0, 1] }),
            ),
            (
                "ui.panel",
                serde_json::json!({ "tree": { "type": "column" } }),
            ),
            ("settings.get", serde_json::json!({})),
            ("settings.set", serde_json::json!({ "value": { "k": 1 } })),
            ("secrets.get", serde_json::json!({ "name": "token" })),
            (
                "fetch",
                serde_json::json!({ "url": "https://example.com", "method": "POST", "headers": { "a": "b" }, "body": "x" }),
            ),
        ];
        assert_eq!(calls.len(), expected.len());
        for ((name, args), (want_name, want_args)) in calls.iter().zip(&expected) {
            assert_eq!(name, want_name);
            assert_eq!(args, want_args, "{name}");
            assert!(protocol::is_call(name));
        }
        assert_eq!(protocol::CALLS.len(), expected.len());
    }

    #[test]
    fn the_global_object_holds_only_builtins_and_kanso() {
        let mut link = Fake::silent();
        let names = [
            "require",
            "console",
            "setTimeout",
            "setInterval",
            "setImmediate",
            "clearTimeout",
            "process",
            "std",
            "os",
            "fetch",
            "eval",
            "XMLHttpRequest",
            "__kansoCall",
            "__kansoLog",
            "print",
            "scriptArgs",
            "module",
            "exports",
            "WebSocket",
            "queueMicrotask",
        ];
        let probe = names
            .iter()
            .map(|n| format!("typeof {n}"))
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!(
            "kanso.commands.register({{ id: 'probe', title: 'P', run() {{ return [{probe}]; }} }});"
        );
        let mut plugin = started(&code, &mut link);
        let seen = plugin.invoke("probe", Value::Null, &mut link).unwrap();
        for (name, kind) in names.iter().zip(seen.as_array().unwrap()) {
            assert_eq!(kind, "undefined", "{name}");
        }
    }

    #[test]
    fn kanso_is_frozen_and_cannot_be_replaced() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.commands.register({ id: 'try', title: 'T', run() {
                'use strict';
                const out = [];
                for (const edit of [
                    () => { kanso.fetch = null; },
                    () => { kanso.notes.get = null; },
                    () => { kanso.notes.pluginData.get = null; },
                    () => { kanso = {}; },
                    () => { delete globalThis.kanso; },
                ]) { try { edit(); out.push('changed'); } catch (e) { out.push('refused'); } }
                return out;
            } });
        "#;
        let mut plugin = started(code, &mut link);
        let out = plugin.invoke("try", Value::Null, &mut link).unwrap();
        assert_eq!(
            out,
            serde_json::json!(["refused", "refused", "refused", "refused", "refused"])
        );
    }

    #[test]
    fn results_are_json_and_undefined_is_null() {
        let mut link = Fake::silent();
        let code = r#"
            kanso.commands.register({ id: 'undef', title: 'U', run() {} });
            kanso.commands.register({ id: 'big', title: 'B', run() { return 'x'.repeat(2000000); } });
            kanso.commands.register({ id: 'cycle', title: 'C', run() { const a = {}; a.a = a; return a; } });
            kanso.commands.register({ id: 'bigint', title: 'I', run() { return 10n; } });
            kanso.commands.register({ id: 'nan', title: 'N', run() { return [NaN, Infinity, undefined, () => 1]; } });
        "#;
        let mut plugin = started(code, &mut link);
        assert_eq!(
            plugin.invoke("undef", Value::Null, &mut link).unwrap(),
            Value::Null
        );
        assert_eq!(
            plugin
                .invoke("big", Value::Null, &mut link)
                .unwrap_err()
                .code,
            CODE_LIMIT
        );
        assert_eq!(
            plugin
                .invoke("cycle", Value::Null, &mut link)
                .unwrap_err()
                .code,
            CODE_THROWN
        );
        assert_eq!(
            plugin
                .invoke("bigint", Value::Null, &mut link)
                .unwrap_err()
                .code,
            CODE_THROWN
        );
        assert_eq!(
            plugin.invoke("nan", Value::Null, &mut link).unwrap(),
            serde_json::json!([null, null, null, null])
        );
    }

    #[test]
    fn oversized_arguments_are_refused_before_the_app() {
        let mut link = Fake::silent();
        let code = "kanso.commands.register({ id: 'x', title: 'X', run() { return kanso.notes.create({ body: 'x'.repeat(2000000) }); } });";
        let mut plugin = started(code, &mut link);
        assert_eq!(
            plugin.invoke("x", Value::Null, &mut link).unwrap_err().code,
            CODE_LIMIT
        );
        assert!(link.calls.borrow().is_empty());
    }

    #[test]
    fn log_levels_are_kept_or_made_info() {
        let mut link = Fake::silent();
        let code = "kanso.log('warn', 'a'); kanso.log('shout', 'b'); kanso.log('error', 42);";
        started(code, &mut link);
        assert_eq!(
            link.logs.borrow().as_slice(),
            [
                ("warn".to_owned(), "a".to_owned()),
                ("info".to_owned(), "b".to_owned()),
                ("error".to_owned(), "42".to_owned()),
            ]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Invariant: any JSON a command is given comes back as the same
        /// JSON, numbers aside from their integer-or-float spelling.
        #[test]
        fn prop_arguments_round_trip(value in json_value()) {
            let mut link = Fake::silent();
            let mut plugin = started("kanso.commands.register({ id: 'echo', title: 'E', run(a) { return a; } });", &mut link);
            let back = plugin.invoke("echo", value.clone(), &mut link).unwrap();
            prop_assert!(same(&back, &value), "{back} vs {value}");
        }

        /// Invariant: any JSON the app answers a host call with reaches the
        /// script unchanged, and the script's own copy goes back out unchanged.
        #[test]
        fn prop_host_answers_round_trip(value in json_value()) {
            let answer = value.clone();
            let mut link = Fake::new(move |_, _| Ok(answer.clone()));
            let mut plugin = started("kanso.commands.register({ id: 'get', title: 'G', run() { return kanso.settings.get(); } });", &mut link);
            let back = plugin.invoke("get", Value::Null, &mut link).unwrap();
            prop_assert!(same(&back, &value), "{back} vs {value}");
        }

        /// Invariant: any JSON a script passes to a host call reaches the
        /// app unchanged, wrapped as the protocol says.
        #[test]
        fn prop_host_arguments_round_trip(value in json_value()) {
            let mut link = Fake::silent();
            let mut plugin = started("kanso.commands.register({ id: 'set', title: 'S', run(a) { kanso.settings.set(a); } });", &mut link);
            plugin.invoke("set", value.clone(), &mut link).unwrap();
            let calls = link.calls.borrow();
            prop_assert_eq!(calls.len(), 1);
            prop_assert_eq!(calls[0].0.as_str(), "settings.set");
            prop_assert!(same(&calls[0].1["value"], &value));
        }

        /// Invariant: a command registers if and only if its id is 1 to 64
        /// of a-z, 0-9 and "-" and its title is 1 to 80 characters.
        #[test]
        fn prop_command_rules(
            id in prop_oneof!["[a-z0-9-]{0,70}", "\\PC{0,6}", "[a-z0-9-]{1,3}[A-Z _.][a-z]{0,3}"],
            title in prop_oneof!["\\PC{0,90}", "[a-z]{78,82}", "\\s{0,3}"],
        ) {
            let code = format!(
                "kanso.commands.register({{ id: {}, title: {}, run() {{}} }});",
                serde_json::to_string(&id).unwrap(),
                serde_json::to_string(&title).unwrap(),
            );
            let allowed = is_command_id(&id) && (1..=80).contains(&title.chars().count());
            let result = start(&code, &mut Fake::silent());
            prop_assert_eq!(result.is_ok(), allowed, "{:?} {:?}", id, title);
            if let Ok((_, registered)) = result {
                prop_assert_eq!(&registered.commands[0].id, &id);
                prop_assert_eq!(&registered.commands[0].title, &title);
            }
        }

        /// Invariant: a command that returns a string is answered with that
        /// exact string, however long, so nothing in the pipeline rewrites text.
        #[test]
        fn prop_strings_are_kept(text in "\\PC{0,200}") {
            let mut link = Fake::silent();
            let mut plugin = started("kanso.commands.register({ id: 'echo', title: 'E', run(a) { return a.text + ''; } });", &mut link);
            let back = plugin.invoke("echo", serde_json::json!({ "text": text }), &mut link).unwrap();
            prop_assert_eq!(back, Value::String(text));
        }
    }

    proptest! {
        /// Invariant: no failure code from the script is passed on unless
        /// it is one the protocol defines; anything else reads as `thrown`.
        #[test]
        fn prop_unknown_codes_read_as_thrown(code in "[a-z_]{0,12}", message in "\\PC{0,30}") {
            let known = [CODE_NOT_FOUND, CODE_CONFLICT, CODE_DENIED, CODE_INVALID, CODE_LIMIT, CODE_TIMEOUT, CODE_UNAVAILABLE];
            let failure = failure_from(Envelope {
                ok: false,
                value: None,
                code: Some(code.clone()),
                message: Some(message),
            });
            if known.contains(&code.as_str()) {
                prop_assert_eq!(failure.code, code);
            } else {
                prop_assert_eq!(failure.code.as_str(), CODE_THROWN);
            }
        }
    }
}
