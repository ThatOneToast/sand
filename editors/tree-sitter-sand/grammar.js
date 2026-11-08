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
  unary: 7,
  cast: 8,
};

const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule)), optional(',')));

module.exports = grammar({
  name: 'sand',

  extras: ($) => [/\s/, $.doc_comment, $.line_comment, $.block_comment],

  word: ($) => $.identifier,

  conflicts: ($) => [[$._condition, $._expression]],

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
        $.flow_item,
        $.extern_function,
        $.function_item,
        $.data_item,
      ),

    data_item: ($) =>
      seq(
        repeat($.attribute),
        optional($.visibility),
        field('kind', alias(choice('item', 'recipe', 'tag'), $.data_kind)),
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
    _data_value: ($) => choice($.data_array, $.data_object, $._expression),
    data_array: ($) => seq('[', commaSep($._data_value), ']'),

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
        field('name', alias($.identifier, $.field_identifier)),
        ':',
        field('type', $.type),
        optional(seq('=', field('default', $._expression))),
      ),
    type: ($) => $.path,

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
        $.parameters,
        optional(seq('->', field('return_type', $.type))),
        field('body', $.block),
      ),
    parameters: ($) => seq('(', commaSep($.parameter), ')'),
    parameter: ($) => seq(field('name', $.identifier), ':', field('type', $.type)),

    // ── Statements ─────────────────────────────────────────────────────────
    block: ($) => seq('{', repeat($._statement), '}'),
    _statement: ($) =>
      choice(
        $.let_statement,
        $.return_statement,
        $.continue_statement,
        $.while_statement,
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
    mutable_specifier: (_) => 'mut',
    return_statement: ($) => seq('return', optional($._value_or_condition), ';'),
    _value_or_condition: ($) => choice($._expression, $.comparison, $.logical_condition, $.not_condition, $.in_range),
    continue_statement: (_) => seq('continue', ';'),
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
        field('source', $.call_expression),
        field('body', $.block),
      ),
    macro_invocation: ($) =>
      seq(field('macro', alias($.identifier, $.macro_name)), '!', $.arguments, ';'),
    assignment: ($) =>
      seq(
        field('target', choice($.place, $.identifier)),
        field('operator', choice('=', '+=', '-=', '*=', '/=', '%=')),
        field('value', $._value_or_condition),
        ';',
      ),
    call_statement: ($) => seq($.call_expression, ';'),

    // ── Conditions ─────────────────────────────────────────────────────────
    _condition: ($) =>
      choice($.logical_condition, $.not_condition, $.comparison, $.in_range, $.parenthesized_condition, $.call_expression, $.place, $.path, $.boolean),
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
    parenthesized_condition: ($) => seq('(', $._condition, ')'),
    range: ($) =>
      prec.right(seq(optional($._expression), choice('..', '..='), optional($._expression))),

    // ── Expressions ────────────────────────────────────────────────────────
    _expression: ($) =>
      choice(
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
      ),
    call_expression: ($) => seq(field('function', $.path), $.arguments),
    arguments: ($) => seq('(', commaSep($.argument), ')'),
    argument: ($) =>
      choice(seq(field('name', alias($.identifier, $.argument_name)), '=', $._argument_value), $._argument_value),
    _argument_value: ($) => choice($.range, $._expression),
    place: ($) =>
      seq(field('state', $.path), '.', field('field', alias($.identifier, $.field_identifier))),
    unary_expression: ($) => prec(PREC.unary, seq('-', $._expression)),
    binary_expression: ($) =>
      choice(
        prec.left(PREC.additive, seq($._expression, choice('+', '-'), $._expression)),
        prec.left(PREC.multiplicative, seq($._expression, choice('*', '/', '%'), $._expression)),
      ),
    cast_expression: ($) => prec(PREC.cast, seq($._expression, 'as', $.type)),
    parenthesized_expression: ($) => seq('(', $._expression, ')'),

    path: ($) => prec.left(seq($._path_segment, repeat(seq('::', $._path_segment)))),
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
