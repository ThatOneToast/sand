; Comments
(doc_comment) @comment.doc
(line_comment) @comment
(block_comment) @comment

; Literals
(string) @string
(escape_sequence) @string.escape
(brace_escape) @string.escape
(interpolation
  "{" @punctuation.special
  "}" @punctuation.special) @embedded
(interpolation_content) @variable.special
(integer) @number
(float) @number
(boolean) @boolean

; Attributes: #[tick], #[on(death)], #![pack(...)]
(attribute "#" @attribute "[" @attribute "]" @attribute)
(inner_attribute "#" @attribute "!" @attribute "[" @attribute "]" @attribute)
(attribute_name) @attribute
(argument_name) @property

; Declarations
(function_item name: (identifier) @function)
(extern_function name: (identifier) @function)
(state_item name: (identifier) @type)
(enum_item name: (identifier) @type)
(flow_item name: (identifier) @type)
(mod_item name: (identifier) @namespace)
(const_item name: (identifier) @constant)
(variant name: (identifier) @variant)
(field_declaration name: (field_identifier) @property)
(data_item name: (identifier) @type)
(data_kind) @keyword
(data_field key: (field_identifier) @property)
(parameter name: (identifier) @variable.parameter)
(let_statement name: (identifier) @variable)
(mutable_specifier) @keyword.modifier
(type) @type
(scope) @keyword.modifier
(hook_phase) @keyword

; Uses
(place field: (field_identifier) @property)
(macro_invocation macro: (macro_name) @function.macro "!" @function.macro)
(call_expression function: (path (identifier) @function .))
(transition from: (path (identifier) @variant))
(transition to: (path (identifier) @variant))
(hook state: (path (identifier) @variant))
(hook function: (path (identifier) @function .))

; Paths: `Type::Variant`, `module::item`, upper-case names as types
((identifier) @type
  (#match? @type "^[A-Z][A-Za-z0-9_]*$"))
((identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]+$"))
(crate) @keyword
(self) @keyword
(super) @keyword

; Keywords
[
  "fn"
  "mod"
  "use"
  "const"
  "enum"
  "state"
  "flow"
  "extern"
  "as"
  "on"
  "let"
] @keyword

[
  "if"
  "else"
  "for"
  "in"
  "every"
  "priority"
  "while"
  "return"
  "continue"
] @keyword.control

(visibility) @keyword

; Operators and punctuation
[
  "=" "+=" "-=" "*=" "/=" "%="
  "==" "!=" "<" "<=" ">" ">="
  "&&" "||" "!"
  "+" "-" "*" "/" "%"
  ".." "..="
  "->" "=>"
] @operator

["(" ")" "{" "}" "[" "]"] @punctuation.bracket
["," ";" ":" "::" "."] @punctuation.delimiter
