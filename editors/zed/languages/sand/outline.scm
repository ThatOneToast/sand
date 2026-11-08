(function_item
  (visibility)? @context
  "fn" @context
  name: (identifier) @name) @item

(extern_function
  "extern" @context
  name: (identifier) @name) @item

(state_item
  scope: (scope) @context
  "state" @context
  name: (identifier) @name) @item

(field_declaration
  name: (field_identifier) @name) @item

(enum_item
  "enum" @context
  name: (identifier) @name) @item

(variant
  name: (identifier) @name) @item

(flow_item
  "flow" @context
  name: (identifier) @name) @item

(const_item
  "const" @context
  name: (identifier) @name) @item

(mod_item
  "mod" @context
  name: (identifier) @name) @item

(data_item
  kind: (data_kind) @context
  name: (identifier) @name) @item
