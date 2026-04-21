/**
 * @file Sea of Nodes IR node description file
 * @author Floweynt
 * @license GPL
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const ID_REGEX =  /[a-zA-Z_][a-zA-Z_0-9]*/;

const comma_separated_nonempty = $ => seq(repeat(seq($, ",")), optional($))
const comma_separated = $ => optional(comma_separated_nonempty($));

module.exports = grammar({
    name: "nodes",

    word: $ => $.identifier,

    extras: ($) => [
        /\s/,
        $.comment,
    ],

    rules: {
        source_file: $ => repeat($.top_level),

        identifier: $ => ID_REGEX,
        tag: $ => token(seq("#", ID_REGEX)),
        type: $ => token(seq("@", ID_REGEX)),

        comment: $ => token(
            choice(
                seq("//", /.*/),
                seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")
            )
        ),

        class_def: $ => seq(
            "class",
            field("name", $.identifier),
            field("params", optional($.parameters)),
            ":",
            field("extends", comma_separated_nonempty($.class_instantiation)),
            "{",
            repeat($.class_def_body),
            "}"
        ),

        node_def: $ => seq(
            "node",
            field("name", $.identifier),
            ":",
            field("extends", comma_separated_nonempty($.class_instantiation)),
            choice(
                seq("{", repeat($.node_def_body), "}"), 
                ";"
            )
        ),

        class_def_body: $ => choice(
            $.ins_block,
            $.trait_def,
            $.outs_block,
            $.type_check_block,
            $.constructor_block,
            $.let_def
        ),

        node_def_body: $ => choice(
            $.ins_block,
            $.outs_block,
            $.type_check_block,
            $.constructor_block,
            $.let_def
        ),

        trait_def: $ => seq(
            "trait",
            $.declaration,
            ";"
        ),

        let_def: $ => seq(
            "let",
            field("name", $.identifier),
            "=",
            field("value", $.expression),
            ";"
        ),

        ins_block: $ => seq(
            "ins",
            "{",
            repeat(choice($.statement, $.ins_replace)),
            "}"
        ),

        ins_replace: $ => seq(
            "shadow", $.identifier, "{",
            repeat($.statement),
            "}"
        ),

        outs_block: $ => seq(
            "outs",
            "{",
            repeat(choice($.statement, $.outs_use)),
            "}"
        ),

        outs_use: $ => seq("use", $.identifier, ";"),

        type_check_block: $ => seq(
            "type_check",
            "{",
            repeat($.statement),
            "}"
        ),

        constructor_block: $ => seq(
            "constructor",
            optional($.parameters),
            "{",
            repeat($.statement),
            "}"
        ),

        parameters: $ => seq(
            "(",
            comma_separated($.declaration),
            ")",
        ),

        declaration: $ => seq(
            field("name", $.identifier),
            ":",
            field("type", $.expression)
        ),

        class_instantiation: $ => choice(
            $.identifier, 
            $.call
        ),

        call: $ => seq(
            field("name", $.identifier),
            "(",
            field("args", comma_separated($.expression)),
            ")"
        ),

        statement: $ => seq(
            choice(
                $.expression,
                $.def_statement
            ), 
            ";"
        ),

        dec_number: $ => token(/[1-9][0-9]*|0/),
        hex_number: $ => token(/0[xX][a-zA-Z]+/),
        string: $ => token(/"([^"\\]*(\\.[^"\\]*)*)"/),

        number: $ => choice(
            $.dec_number,
            $.hex_number
        ),

        expression: $ => choice(
            $.identifier,
            $.number,
            $.tag,
            $.unary_expression,
            $.binary_expression,
            $.range_expression,
            $.call,
            $.string,
            $.type,
            "nullptr",
            "true",
            "false"
        ),

        def_statement: $ => seq($.identifier, ":", $.expression),

        unary_expression: $ => choice(
            prec(20, seq('!', $.expression)),
            prec(21, seq($.expression, '?'))
        ),

        range_expression: $ => choice(
            prec.left(15, seq($.expression, '..')),
            prec.left(15, seq('..', $.expression)),
            prec.left(15, seq($.expression, '..', $.expression))
        ),

        binary_expression: $ => choice(
            prec.left(21, seq($.expression, choice('::'), $.expression)),
            prec.left(10, seq($.expression, choice('*', '/', '%'), $.expression)),
            prec.left(9, seq($.expression, choice('+', '-'), $.expression)),
            prec.left(8, seq($.expression, choice('<', '<=', '>', '>='), $.expression)),
            prec.left(7, seq($.expression, 'is', $.identifier)),
            prec.left(6, seq($.expression, choice('==', '!='), $.expression)),
            prec.left(5, seq($.expression, choice('='), $.expression)),
        ),

        mod_def: $ => seq(
            "mod",
            $.string,
            ";"
        ),

        top_level: $ => choice(
            $.class_def,
            $.node_def,
            $.mod_def
        )
    }
});

