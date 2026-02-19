// ── Latch Standard Library Modules ───────────────────────────
// These module names and method signatures are LOCKED as of v0.1.0.
// Do not rename or remove existing methods. New methods may be added.
//
// Locked names:
//   fs   : read, write, exists, glob
//   proc : exec, pipe
//   http : get, post
//   time : sleep, now
//   ai   : ask, summarize
//
// Built-in functions (locked):
//   print, len, str, int, float, typeof, push, keys, values,
//   range, split, trim, contains, replace

pub mod fs;
pub mod proc;
pub mod http;
pub mod time;
pub mod ai;
