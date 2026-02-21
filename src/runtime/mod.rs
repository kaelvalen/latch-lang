// ── Latch Standard Library Modules ───────────────────────────
// These module names and method signatures are LOCKED as of v0.3.0.
// Do not rename or remove existing methods. New methods may be added.
//
// Locked names:
//   fs   : read, write, append, readlines, exists, glob, mkdir, remove, stat
//   proc : exec (string or list), pipe
//   http : get, post                  → returns HttpResponse
//   time : sleep, now
//   ai   : ask, summarize
//   json : parse, stringify
//   env  : get, set, list
//   path : join, basename, dirname, ext, abs
//
// Built-in functions (locked):
//   print, len, str, int, float, typeof, push, keys, values,
//   range, split, trim, lower, upper, starts_with, ends_with,
//   contains, replace, sort, filter, map, each

pub mod fs;
pub mod proc;
pub mod http;
pub mod time;
pub mod ai;
pub mod json;
pub mod env;
pub mod path;
