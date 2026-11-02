// The `kanso` object a plugin sees. runtime.rs evaluates this once, before
// the plugin's own script, as a global script that evaluates to the
// internals object it keeps (the plugin never gets that object).
//
// The two native functions the host registers are captured here and then
// deleted from the global object, so plugin code can only use `kanso.*`.
// Every host call is synchronous: it hands JSON to the native function and
// gets JSON back, `{"ok":true,"value":...}` or `{"ok":false,"code":...,
// "message":...}`. A failed call throws an Error with `.code` and
// `.message`.
(function (nativeCall, nativeLog) {
  'use strict';

  // Keep what we need from the builtins, so a plugin that edits a
  // prototype later can't change how this file behaves.
  const stringify = JSON.stringify;
  const parse = JSON.parse;
  const freeze = Object.freeze;
  const isArray = Array.isArray;
  const create = Object.create;
  const defineProperty = Object.defineProperty;
  const getGlobal = () => globalThis;

  const global = getGlobal();
  delete global.__kansoCall;
  delete global.__kansoLog;
  delete global.eval;
  delete global.queueMicrotask;

  const ID = /^[a-z0-9-]{1,64}$/;
  const LANGUAGE = /^[A-Za-z0-9][A-Za-z0-9_+#.-]{0,31}$/;
  const WHEN = ['always', 'note-open', 'selection'];
  const LEVELS = ['debug', 'info', 'warn', 'error'];
  const MAX_COMMANDS = 100;
  const MAX_FENCES = 50;
  const MAX_PRESSES = 100;
  const MAX_KEYWORDS = 20;

  function failure(code, message) {
    const error = new Error(message);
    error.code = code;
    return error;
  }

  // Characters as a person counts them, not UTF-16 units.
  function characters(text) {
    let count = 0;
    for (let i = 0; i < text.length; i++) {
      const unit = text.charCodeAt(i);
      if (unit >= 0xdc00 && unit <= 0xdfff && i > 0) {
        const before = text.charCodeAt(i - 1);
        if (before >= 0xd800 && before <= 0xdbff) continue;
      }
      count++;
    }
    return count;
  }

  function oneOf(list, value) {
    for (let i = 0; i < list.length; i++) {
      if (list[i] === value) return true;
    }
    return false;
  }

  function needString(value, what) {
    if (typeof value !== 'string') throw failure('invalid', what + ' must be a string');
    return value;
  }

  function needObject(value, what) {
    if (value === null || typeof value !== 'object' || isArray(value)) {
      throw failure('invalid', what + ' must be an object');
    }
    return value;
  }

  function call(name, args) {
    const json = stringify(args === undefined ? null : args);
    const answer = parse(nativeCall(name, json === undefined ? 'null' : json));
    if (answer.ok) return answer.value === undefined ? null : answer.value;
    throw failure(answer.code, answer.message);
  }

  // Registration happens while the entry script runs, then is sealed.
  let sealed = false;
  const commands = create(null);
  const commandList = [];
  const fences = create(null);
  const fenceList = [];
  const presses = create(null);
  const pressList = [];

  function needOpen(what) {
    if (sealed) throw failure('invalid', what + ' can only be registered while the plugin starts');
  }

  function registerCommand(spec) {
    needOpen('a command');
    needObject(spec, 'the command');
    const id = needString(spec.id, 'the command id');
    if (!ID.test(id)) throw failure('invalid', 'a command id is 1 to 64 of a-z, 0-9 and -');
    if (id in commands) throw failure('invalid', 'there is already a command "' + id + '"');
    const title = needString(spec.title, 'the command title');
    const size = characters(title);
    if (size < 1 || size > 80) throw failure('invalid', 'a command title is 1 to 80 characters');
    let keywords = [];
    if (spec.keywords !== undefined) {
      if (!isArray(spec.keywords) || spec.keywords.length > MAX_KEYWORDS) {
        throw failure('invalid', 'keywords is a list of at most ' + MAX_KEYWORDS + ' words');
      }
      for (let i = 0; i < spec.keywords.length; i++) {
        const word = needString(spec.keywords[i], 'a keyword');
        if (characters(word) > 80) throw failure('invalid', 'a keyword is at most 80 characters');
        keywords.push(word);
      }
    }
    const when = spec.when === undefined ? 'always' : spec.when;
    if (!oneOf(WHEN, when)) throw failure('invalid', 'when is always, note-open or selection');
    if (typeof spec.run !== 'function') throw failure('invalid', 'a command needs a run function');
    if (commandList.length >= MAX_COMMANDS) throw failure('limit', 'too many commands');
    commands[id] = spec.run;
    commandList.push({ id: id, title: title, keywords: keywords, when: when });
  }

  function registerFence(language, render) {
    needOpen('a renderer');
    needString(language, 'the language');
    if (!LANGUAGE.test(language)) throw failure('invalid', 'that isn\'t a fence language');
    if (typeof render !== 'function') throw failure('invalid', 'a renderer is a function');
    if (language in fences) throw failure('invalid', 'there is already a renderer for "' + language + '"');
    if (fenceList.length >= MAX_FENCES) throw failure('limit', 'too many renderers');
    fences[language] = render;
    fenceList.push(language);
  }

  function onPress(id, handler) {
    needOpen('a button');
    needString(id, 'the button id');
    const size = characters(id);
    if (size < 1 || size > 64) throw failure('invalid', 'a button id is 1 to 64 characters');
    if (typeof handler !== 'function') throw failure('invalid', 'a press handler is a function');
    if (id in presses) throw failure('invalid', 'there is already a handler for "' + id + '"');
    if (pressList.length >= MAX_PRESSES) throw failure('limit', 'too many buttons');
    presses[id] = handler;
    pressList.push(id);
  }

  const kanso = {
    commands: freeze({ register: registerCommand }),
    notes: freeze({
      list: (opts) => call('notes.list', opts === undefined ? {} : needObject(opts, 'the options')),
      get: (id) => call('notes.get', { id: needString(id, 'the note id') }),
      create: (note) => call('notes.create', needObject(note, 'the note')),
      update: (id, fields, opts) =>
        call('notes.update', {
          id: needString(id, 'the note id'),
          fields: needObject(fields, 'the fields'),
          baseRev: opts === undefined || opts === null ? undefined : opts.baseRev,
        }),
      pluginData: freeze({
        get: (id) => call('notes.pluginData.get', { id: needString(id, 'the note id') }),
        set: (id, value) => call('notes.pluginData.set', { id: needString(id, 'the note id'), value: value }),
      }),
    }),
    attachments: freeze({
      get: (fileId) => call('attachments.get', { fileId: needString(fileId, 'the file id') }),
    }),
    editor: freeze({
      noteId: () => call('editor.noteId', {}),
      selection: () => call('editor.selection', {}),
      replaceSelection: (change) => call('editor.replaceSelection', needObject(change, 'the change')),
    }),
    fences: freeze({ register: registerFence }),
    ui: freeze({
      panel: (tree) => call('ui.panel', { tree: tree }),
      onPress: onPress,
    }),
    settings: freeze({
      get: () => call('settings.get', {}),
      set: (value) => call('settings.set', { value: value }),
    }),
    secrets: freeze({
      get: (name) => call('secrets.get', { name: needString(name, 'the secret name') }),
    }),
    fetch: (url, opts) => {
      const o = opts === undefined || opts === null ? {} : needObject(opts, 'the options');
      return call('fetch', {
        url: needString(url, 'the url'),
        method: o.method,
        headers: o.headers,
        body: o.body,
      });
    },
    log: (level, text) => {
      const name = oneOf(LEVELS, level) ? level : 'info';
      nativeLog(name, String(text));
    },
  };
  freeze(kanso);
  defineProperty(global, 'kanso', { value: kanso, writable: false, configurable: false, enumerable: false });

  // What the Rust side uses. Every function answers with an envelope string
  // and never throws what the plugin threw.
  function ok(value) {
    const json = stringify(value);
    return '{"ok":true,"value":' + (json === undefined ? 'null' : json) + '}';
  }

  function thrown(error) {
    // QuickJS throws null when it can't even make the out-of-memory error.
    if (error === null) {
      return stringify({ ok: false, code: 'limit', message: 'it used too much memory' });
    }
    let code = 'thrown';
    let message;
    try {
      if (error !== null && typeof error === 'object') {
        message = typeof error.message === 'string' ? error.message : String(error);
        if (typeof error.code === 'string') code = error.code;
      } else {
        message = String(error);
      }
    } catch (_) {
      message = 'the plugin threw something that can\'t be read';
    }
    return stringify({ ok: false, code: code, message: message });
  }

  function missing(what) {
    return stringify({ ok: false, code: 'not_found', message: what });
  }

  function dispatch(kind, name, first, second) {
    try {
      if (kind === 'invoke') {
        if (!(name in commands)) return missing('there is no command "' + name + '"');
        return ok(commands[name](parse(first)));
      }
      if (kind === 'render') {
        if (!(name in fences)) return missing('there is no renderer for "' + name + '"');
        const drawn = fences[name](freeze({ source: first, theme: parse(second) }));
        if (typeof drawn !== 'string') {
          return stringify({ ok: false, code: 'invalid', message: 'a renderer must return a string' });
        }
        return ok(drawn);
      }
      if (kind === 'press') {
        if (!(name in presses)) return missing('there is no button "' + name + '"');
        return ok(presses[name](parse(first)));
      }
      return missing('unknown request');
    } catch (error) {
      return thrown(error);
    }
  }

  return freeze({
    dispatch: dispatch,
    seal: () => {
      sealed = true;
    },
    registered: () => stringify({ commands: commandList, fences: fenceList, presses: pressList }),
  });
})(globalThis.__kansoCall, globalThis.__kansoLog)
