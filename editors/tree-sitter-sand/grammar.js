/**
 * Tree-sitter grammar for the Sand datapack language (`.sand`).
 *
 * This grammar drives editor highlighting, brackets, indentation, and
 * outlines. The authoritative grammar used by the compiler and language
 * server is `sand-lang/src/sand.pest`; keep the two in step.
 */

const PREC = {
  or: 1,
  and: 2,
  not: 3,
  compare: 4,
  additive: 5,
  multiplicative: 6,
  unary: 8,
  cast: 7,
};

const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule)), optional(',')));

module.exports = grammar({
  name: 'sand',

  extras: ($) => [/\s/, $.doc_comment, $.line_comment, $.block_comment],

  word: ($) => $.identifier,

  conflicts: ($) => [[$.type, $.applied_type], [$._condition, $._expression], [$._expression, $.place], [$.record_expression, $._condition], [$.record_expression, $._expression]],

  rules: {
    source_file: ($) => repeat(choice($.inner_attribute, $._item)),

    // ── Comments ───────────────────────────────────────────────────────────
    doc_comment: (_) => token(prec(2, seq(choice('///', '//!'), /.*/))),
    line_comment: (_) => token(prec(1, seq('//', /.*/))),
    block_comment: (_) => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

    // ── Attributes ─────────────────────────────────────────────────────────
    attribute: ($) => seq('#', '[', $._attribute_body, ']'),
    inner_attribute: ($) => seq('#', '!', '[', $._attribute_body, ']'),
    _attribute_body: ($) =>
      seq(field('name', alias($.identifier, $.attribute_name)), optional($.attribute_arguments)),
    attribute_arguments: ($) => seq('(', commaSep($.attribute_argument), ')'),
    attribute_argument: ($) =>
      choice(seq(field('key', alias($.identifier, $.argument_name)), '=', $._value), $._value),
    _value: ($) => choice($.range, $._expression),

    visibility: (_) => 'pub',

    // ── Items ──────────────────────────────────────────────────────────────
    _item: ($) =>
      choice(
        $.mod_item,
        $.use_item,
        $.const_item,
        $.enum_item,
        $.state_item,
        $.struct_item,
        $.trait_item,
        $.impl_item,
        $.flow_item,
        $.extern_function,
        $.function_item,
        $.event_item,
        $.data_item,
      ),

    struct_item: ($) => seq(repeat($.attribute), optional($.visibility), 'struct', field('name', $.identifier), optional($.generic_parameters), '{', commaSep($.field_declaration), '}'),
    trait_item: ($) => seq(repeat($.attribute), optional($.visibility), 'trait', field('name', $.identifier), '{', repeat($.trait_method), '}'),
    trait_method: ($) => seq('fn', field('name', $.identifier), $.parameters, optional(seq('->', field('return_type', $.type))), ';'),
    impl_item: ($) => seq('impl', optional($.generic_parameters), optional(seq(field('trait', $.path), 'for')), field('type', $.path), optional($.type_arguments), '{', repeat($.function_item), '}'),
    record_expression: ($) => seq(field('type', $.path), '{', $.record_field, repeat(seq(',', $.record_field)), optional(','), '}'),
    record_field: ($) => seq(field('name', $.identifier), optional(seq(':', $._value_or_condition))),
    method_expression: ($) => prec.left(10, seq(field('receiver', $._expression), '.', field('method', $.identifier), optional(seq('::', $.type_arguments)), $.arguments)),
    field_expression: ($) => prec.left(10, seq(field('value', $._expression), '.', field('field', $.identifier))),
    receiver: (_) => seq(optional('&'), optional('mut'), 'self'),

    event_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'event',
        field('name', $.identifier),
        optional($.parameters),
        ';',
      ),

    data_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        field('kind', alias(choice('item', 'recipe', 'tag', 'bossbar', 'loot', 'damage_type', 'predicate', 'advancement', 'dialog', 'dimension'), $.data_kind)),
        field('name', $.identifier),
        optional(seq(':', field('type', $.type))),
        $.data_object,
      ),
    data_object: ($) => seq('{', commaSep($.data_field), '}'),
    data_field: ($) =>
      seq(
        field('key', choice(alias($.identifier, $.field_identifier), $.string)),
        ':',
        field('value', $._data_value),
      ),
    _data_value: ($) => choice($.data_object, $._expression),

    mod_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'mod',
        field('name', $.identifier),
        choice(';', seq('{', repeat($._item), '}')),
      ),

    use_item: ($) => seq(repeat($.attribute), optional($.visibility), 'use', $.use_tree, ';'),
    // Segments are spelled out (rather than reusing `path`) so `a::b::{c}`
    // needs only one token of lookahead after `::`.
    use_tree: ($) =>
      seq(
        $._path_segment,
        repeat(seq('::', $._path_segment)),
        optional(choice(seq('::', $.use_group), seq('as', field('alias', $.identifier)))),
      ),
    use_group: ($) => seq('{', commaSep($.use_tree), '}'),

    const_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'const',
        field('name', $.identifier),
        ':',
        field('type', $.type),
        '=',
        field('value', $._expression),
        ';',
      ),

    enum_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'enum',
        field('name', $.identifier),
        '{',
        commaSep($.variant),
        '}',
      ),
    variant: ($) => seq(field('name', $.identifier), optional(seq('=', field('value', $._expression)))),

    state_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        field('scope', alias(choice('player', 'global', 'entity'), $.scope)),
        'state',
        field('name', $.identifier),
        '{',
        commaSep($.field_declaration),
        '}',
      ),
    field_declaration: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        field('name', alias($.identifier, $.field_identifier)),
        ':',
        field('type', $.type),
        optional(seq('=', field('default', $._expression))),
      ),
    type: ($) => choice($.function_type, $.array_type, $.applied_type, $.path),
    function_type: ($) => prec.right(seq("fn", "(", commaSep($.type), ")", optional(seq("->", $.type)))),
    applied_type: ($) => seq(field('name', $.path), '<', $.type, repeat(seq(',', $.type)), optional(','), '>'),
    array_type: ($) => seq('[', field('element', $.type), ';', field('length', choice($.integer, $.path)), ']'),
    array_expression: ($) => seq('[', choice(commaSep(choice($.data_object, $._value_or_condition)), seq($._expression, ';', choice($.integer, $.path))), ']'),
    index_expression: ($) => prec.left(10, seq(field('value', $._expression), '[', field('index', $._expression), ']')),

    flow_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'flow',
        field('name', $.identifier),
        'on',
        field('field', $.place),
        '{',
        repeat(choice($.transition, $.hook)),
        '}',
      ),
    transition: ($) =>
      seq(
        field('from', $._expression),
        '->',
        field('to', $._expression),
        optional(seq('if', field('guard', $._condition))),
        optional(seq('priority', field('priority', $._expression))),
        ';',
      ),
    hook: ($) =>
      seq(
        field('phase', alias(choice('enter', 'exit', 'tick'), $.hook_phase)),
        field('state', $._expression),
        optional(seq('every', field('every', $._expression))),
        choice(seq('=>', field('function', $.path), ';'), field('body', $.block)),
      ),

    extern_function: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'extern',
        field('id', $.string),
        'fn',
        field('name', $.identifier),
        '(',
        ')',
        ';',
      ),

    function_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        'fn',
        field('name', $.identifier),
        optional($.function_type_parameters),
        $.parameters,
        optional(seq('->', field('return_type', $.type))),
        field('body', $.block),
      ),
    parameters: ($) => seq('(', commaSep(choice($.receiver, $.parameter)), ')'),
    parameter: ($) => seq(optional($.mutable_specifier), field('name', $.identifier), ':', field('type', $.type)),

    // ── Statements ─────────────────────────────────────────────────────────
    block: ($) => seq('{', repeat($._statement), '}'),
    _statement: ($) =>
      choice(
        $.destructure_statement,
        $.let_statement,
        $.return_statement,
        $.continue_statement,
        $.break_statement,
        $.emit_statement,
        $.match_statement,
        $.while_statement,
        $.loop_statement,
        $.if_statement,
        $.for_statement,
        $.macro_invocation,
        $.assignment,
        $.call_statement,
      ),
    let_statement: ($) =>
      seq(
        'let',
        optional($.mutable_specifier),
        field('name', $.identifier),
        optional(seq(':', field('type', $.type))),
        optional(seq('=', field('value', $._value_or_condition))),
        ';',
      ),
    destructure_statement: ($) => seq('let', field('pattern', choice($.record_pattern, $.array_pattern, '_')), optional(seq(':', field('type', $.type))), '=', field('value', $._value_or_condition), ';'),
    _value_pattern: ($) => choice($.record_pattern, $.array_pattern, '_', $.binding_pattern),
    binding_pattern: ($) => seq(optional($.mutable_specifier), field('name', $.identifier)),
    record_pattern: ($) => seq(field('type', $.path), '{', optional(choice(seq($.pattern_field, repeat(seq(',', $.pattern_field)), optional(seq(',', '..')), optional(',')), '..')), '}'),
    pattern_field: ($) => seq(field('name', $.identifier), optional(seq(':', $._value_pattern))),
    array_pattern: ($) => seq('[', commaSep($._value_pattern), ']'),
    mutable_specifier: (_) => 'mut',
    return_statement: ($) => seq('return', optional($._value_or_condition), ';'),
    _value_or_condition: ($) => choice($._expression, $.comparison, $.logical_condition, $.not_condition, $.in_range),
    continue_statement: (_) => seq('continue', ';'),
    break_statement: (_) => seq('break', ';'),
    emit_statement: ($) =>
      seq('emit', field('event', $.path), optional($.arguments), ';'),
    loop_statement: ($) => seq('loop', field('body', $.block)),
    match_statement: ($) =>
      seq('match', field('value', $._expression), '{', repeat($.match_arm), '}'),
    match_expression: ($) => seq('match', field('value', $._expression), '{', $.match_value_arm, repeat(seq(',', $.match_value_arm)), optional(','), '}'),
    match_value_arm: ($) => seq(field('pattern', choice($.match_binding, '_', seq($._expression, repeat(seq('|', $._expression))))), optional($.match_guard), '=>', field('value', $._value_or_condition)),
    match_arm: ($) =>
      seq(field('pattern', choice($.match_binding, '_', seq($._expression, repeat(seq('|', $._expression))))),
        optional($.match_guard), '=>', field('body', $.block), optional(',')),
    match_binding: ($) => seq('let', field('binding', $._value_pattern)),
    match_guard: ($) => seq('if', field('condition', $._condition)),
    while_statement: ($) =>
      seq('while', field('condition', $._condition), field('body', $.block)),
    if_statement: ($) =>
      seq(
        'if',
        field('condition', $._condition),
        field('consequence', $.block),
        optional(seq('else', field('alternative', choice($.if_statement, $.block)))),
      ),
    for_statement: ($) =>
      seq(
        'for',
        field('binding', $.identifier),
        'in',
        field('source', choice($._expression, $.count_range)),
        field('body', $.block),
      ),
    count_range: ($) => choice($.range,
      seq('(', $.range, ')', optional(seq('.', 'rev', '(', ')')),
        optional(seq('.', 'step_by', '(', $._expression, ')')))),
    macro_invocation: ($) =>
      seq(field('macro', alias($.identifier, $.macro_name)), '!', $.arguments, ';'),
    assignment: ($) =>
      seq(
        field('target', $._expression),
        field('operator', choice('=', '+=', '-=', '*=', '/=', '%=')),
        field('value', $._value_or_condition),
        ';',
      ),
    call_statement: ($) => seq(choice($.call_expression, $.method_expression), ';'),

    // ── Conditions ─────────────────────────────────────────────────────────
    _condition: ($) =>
      choice($.logical_condition, $.not_condition, $.comparison, $.in_range, $.parenthesized_expression, $.call_expression, $.method_expression, $.field_expression, $.index_expression, $.place, $.path, $.boolean),
    logical_condition: ($) =>
      choice(
        prec.left(PREC.or, seq($._condition, '||', $._condition)),
        prec.left(PREC.and, seq($._condition, '&&', $._condition)),
      ),
    not_condition: ($) => prec(PREC.not, seq('!', $._condition)),
    comparison: ($) =>
      prec.left(
        PREC.compare,
        seq($._expression, choice('==', '!=', '<', '<=', '>', '>='), $._expression),
      ),
    in_range: ($) => prec(PREC.compare, seq($._expression, 'in', $.range)),
    range: ($) =>
      prec.right(seq(optional($._expression), choice('..', '..='), optional($._expression))),

    // ── Expressions ────────────────────────────────────────────────────────
    closure_expression: ($) => seq('move', '|', commaSep($.parameter), '|', optional(seq('->', $.type)), field('body', $.block)),
    block_expression: ($) =>
      seq('{', repeat($._statement), field('value', $._value_or_condition), '}'),
    _expression: ($) =>
      choice(
        $.block_expression,
        $.match_expression,
        $.closure_expression,
        $.integer,
        $.float,
        $.boolean,
        $.string,
        $.place,
        $.path,
        $.call_expression,
        $.unary_expression,
        $.binary_expression,
        $.cast_expression,
        $.parenthesized_expression,
        $.record_expression,
        $.method_expression,
        $.field_expression,
        $.array_expression,
        $.index_expression,
      ),
    call_expression: ($) => seq(field('function', $.path), $.arguments),
    arguments: ($) => seq('(', commaSep($.argument), ')'),
    argument: ($) =>
      choice(seq(field('name', alias($.identifier, $.argument_name)), '=', $._argument_value), $._argument_value),
    _argument_value: ($) => choice($.range, $._value_or_condition),
    place: ($) =>
      seq(field('state', $.path), '.', field('field', alias($.identifier, $.field_identifier))),
    unary_expression: ($) => prec(PREC.unary, seq('-', $._expression)),
    binary_expression: ($) =>
      choice(
        prec.left(PREC.additive, seq($._expression, choice('+', '-'), $._expression)),
        prec.left(PREC.multiplicative, seq($._expression, choice('*', '/', '%'), $._expression)),
      ),
    cast_expression: ($) => prec(PREC.cast, seq($._expression, 'as', $.type)),
    parenthesized_expression: ($) => seq('(', $._value_or_condition, ')'),

    function_type_parameters: ($) => seq('<', $.function_type_parameter, repeat(seq(',', $.function_type_parameter)), optional(','), '>'),
    function_type_parameter: ($) => seq(field('name', $.identifier), optional(seq(':', field('bound', $.path), repeat(seq('+', field('bound', $.path)))))),
    generic_parameters: ($) => seq('<', $.identifier, repeat(seq(',', $.identifier)), optional(','), '>'),
    type_arguments: ($) => seq('<', $.type, repeat(seq(',', $.type)), optional(','), '>'),
    path: ($) => prec.left(seq($._path_segment, optional(seq('::', $.type_arguments)), repeat(seq('::', $._path_segment, optional(seq('::', $.type_arguments)))))),
    _path_segment: ($) => choice($.identifier, $.crate, $.self, $.super),
    crate: (_) => 'crate',
    self: (_) => 'self',
    super: (_) => 'super',

    // ── Literals ───────────────────────────────────────────────────────────
    integer: (_) => /[0-9][0-9_]*/,
    float: (_) => /[0-9]+\.[0-9]+/,
    boolean: (_) => choice('true', 'false'),
    string: ($) =>
      seq(
        '"',
        repeat(choice($.string_content, $.escape_sequence, $.brace_escape, $.interpolation)),
        token.immediate('"'),
      ),
    string_content: (_) => token.immediate(prec(1, /[^"\\{}]+/)),
    escape_sequence: (_) => token.immediate(seq('\\', /./)),
    brace_escape: (_) => token.immediate(prec(2, choice('{{', '}}'))),
    interpolation: ($) =>
      seq(
        token.immediate('{'),
        optional(alias(token.immediate(/[^{}"]+/), $.interpolation_content)),
        token.immediate('}'),
      ),

    identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
  },
});
