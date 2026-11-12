// The calls Quick Capture and Add Task make. Each uses a Capture key's tools
// only: create-note, kanso-add-task, kanso-parse-when, and two reads.

import { Env, callTool } from "./client";

export const createNote = (env: Env, port: number, key: string, args: object) =>
  callTool(env, port, key, "create-note", args);

export const addTask = (env: Env, port: number, key: string, args: object) =>
  callTool(env, port, key, "kanso-add-task", args);

export const parseWhen = (env: Env, port: number, key: string, text: string) =>
  callTool(env, port, key, "kanso-parse-when", { text });

export const listNotebooks = (env: Env, port: number, key: string) => callTool(env, port, key, "list-notebooks", {});

export const listRecentNotes = (env: Env, port: number, key: string) =>
  callTool(env, port, key, "list-notes", { sort: "updatedAt", limit: 30 });
