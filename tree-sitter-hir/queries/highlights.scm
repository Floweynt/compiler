(visibility) @keyword
(keyword) @keyword
"declare" @keyword
"define" @keyword
"var" @keyword

(number) @number
(float) @number.float

(identifier) @variable

(define_fun
  name: (identifier) @function
  param_name: (identifier) @variable.parameter)

(type) @type

(label
  name: (identifier) @label)
