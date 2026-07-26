; Keywords
[
  "if"
  "else"
  "while"
  "for"
  "in"
  "return"
  "break"
  "continue"
  "try"
  "catch"
  "parallel"
  "workers"
  "import"
  "from"
  "export"
  "let"
  "const"
] @keyword

[
  "and"
  "or"
  "not"
] @keyword.operator

; Function declaration
(function_definition
  name: (identifier) @function.definition)

(fn_keyword) @keyword.function
"fn" @keyword.function

; Types
[
  "int"
  "float"
  "string"
  "bool"
  "list"
  "dict"
  "void"
] @type

; Constants
[
  "true"
  "false"
  "null"
] @constant.builtin

; Builtin Modules
(module_identifier) @type.builtin

; Function Calls
(call_expression
  function: (identifier) @function.call)

(method_call_expression
  method: (identifier) @function.method)

; Operators
[
  ":="
  "+="
  "-="
  "*="
  "/="
  "%="
  "="
  "=="
  "!="
  "<="
  ">="
  "<"
  ">"
  "??"
  "?."
  "|>"
  ".."
  "+"
  "-"
  "*"
  "/"
  "%"
] @operator

; Strings & Interpolation
(string_literal) @string
(escape_sequence) @string.escape
(interpolation
  "${" @punctuation.section.interpolation.begin
  "}" @punctuation.section.interpolation.end)

; Numbers
(number_literal) @number

; Comments
(comment) @comment
(line_comment) @comment
(block_comment) @comment

; Punctuation
[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  ","
  ";"
  "."
] @punctuation.delimiter
