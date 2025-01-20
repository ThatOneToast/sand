module.exports = grammar({
    name: 'sand',

    rules: {
        source_file: $ => repeat($._definition),

        identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
        number: $ => /[-]?\d+(\.\d+)?/,

        comment: $ => token(seq(
            '//',           // Comment start
            /[^\\]*/,       // Any characters except backslash
            '\\\\'          // Comment end with double backslash
        )),


    }
});
