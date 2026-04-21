":" @punctuation.delimiter
"," @punctuation.delimiter
";" @punctuation.delimiter
"::" @operator
"*" @operator
"/" @operator
"%" @operator
"+" @operator
"-" @operator
"<" @operator
"<=" @operator
">" @operator
">=" @operator
"==" @operator
"!=" @operator
".." @operator
"!" @operator
"?" @operator
"=" @operator
"ins" @keyword
"outs" @keyword
"trait" @keyword
"let" @keyword
"type_check" @keyword
"class" @keyword
"node" @keyword
"shadow" @keyword
"use" @keyword
"constructor" @constructor
"nullptr" @constant.builtin
"false" @boolean
"true" @boolean
"mod" @keyword

(number) @number
(string) @string
(type) @type.builtin
(tag) @type.builtin
(identifier) @variable
(comment) @comment

(class_def 
  name: (identifier) @type)

(node_def 
  name: (identifier) @type)

(class_instantiation
  (identifier) @type)

(let_def
  name: (identifier) @property)

(trait_def
  (declaration 
    (identifier) @property))

(call
  name: (identifier) @function)

(class_instantiation
  (call
    name: (identifier) @type))

(binary_expression 
  (expression (identifier) @keyword) (#eq? @keyword "ins")  
  "::"
  (_))

(binary_expression 
  (expression (identifier) @keyword) (#eq? @keyword "outs")  
  "::"
  (_))

(binary_expression 
  (expression (identifier) @keyword) (#eq? @keyword "func")  
  "::"
  (_))

(binary_expression 
  (expression (identifier) @keyword) (#eq? @keyword "types")  
  "::"
  (_))

(binary_expression 
  (_)  
  "is" @operator
  (identifier) @type)

(declaration
  (_)
  (expression (identifier)) @type)

(declaration
  (_)
  (expression
    (range_expression 
      (expression 
        (identifier) @type))))

(def_statement
  (identifier) @property)
