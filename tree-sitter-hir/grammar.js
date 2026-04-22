/**
 * @file Stack machine based IR for target independent optimization and compilation
 * @author Flowey
 * @license GPL-3.0-or-later
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

function commaSep(rule) {
    return optional(seq(rule, repeat(seq(',', rule))));
}

export default grammar({
    name: "hir",
    word: $ => $.keyword_internal,
    rules: {
        source_file: $ => repeat(choice($.declare, $.define_var, $.define_fun)),

        identifier: $ => /@[\w.$]+/,
        keyword_internal: $ => /[a-zA-Z_][\w.]*/,

        _keyword: $ => alias($.keyword_internal, $.keyword),

        visibility: $ => choice("internal", "external", "private"),
        number: $ => choice(/[+-]?\d+/, /[+-]?0x[0-9a-fA-F]+/),
		    float: $ => /[+-]?\d+\.\d*([Ee][+-]?\d+)?/,

        declare: $ => seq(
            "declare", 
            field("visibility", $.visibility), 
            field("name", $.identifier), 
            ";"
        ),

        type: $ => /[a-zA-Z0-9_]\w*/,

        define_var: $ => seq(
            "define", 
            field("type", $.type), 
            field("name", $.identifier), 
            ";"
        ),

        define_fun: $ => seq(
            "define",
            field("return_type", $.type),
            field("name", $.identifier),
            "(", 
                commaSep(
                    seq(
                        field("param_type", $.type),
                        field("param_name", $.identifier)
                    )
                ), 
            ")",
            "{",
                field("vars", repeat($.var_decl)),
                field("body", repeat(choice(
                    $.instruction,
                    $.label
                ))),
            "}"
        ),

        var_decl: $ => seq(
            "var", 
            field("type", $.type),
            field("name", $.identifier),
            ";"
        ),

        instruction: $ => seq(
            field("opcode", $._keyword),
            field("type", $.type),
            commaSep($.operand),
            ";"
        ),

        label: $ => seq(
            field("name", $.identifier),
            ":"
        ),

        operand: $ => choice($.identifier, $.number, $.float)
    }
});
