" Vim syntax file
" Language:	Wop grammar
" Current Maintainer: Defmc (https://github.com/Defmc)
" Last Change:	2023 Dec 9

	" *Constant	any constant
	"  String		a string constant: "this is a string"
	"  Character	a character constant: 'c', '\n'
	"  Number		a number constant: 234, 0xff
	"  Boolean	a boolean constant: TRUE, false
	"  Float		a floating point constant: 2.3e10
	"
	" *Identifier	any variable name
	"  Function	function name (also: methods for classes)
	"
	" *Statement	any statement
	"  Conditional	if, then, else, endif, switch, etc.
	"  Repeat		for, do, while, etc.
	"  Label		case, default, etc.
	"  Operator	"sizeof", "+", "*", etc.
	"  Keyword	any other keyword
	"  Exception	try, catch, throw
	"
	" *PreProc	generic Preprocessor
	"  Include	preprocessor #include
	"  Define		preprocessor #define
	"  Macro		same as Define
	"  PreCondit	preprocessor #if, #else, #endif, etc.
	"
	" *Type		int, long, char, etc.
	"  StorageClass	static, register, volatile, etc.
	"  Structure	struct, union, enum, etc.
	"  Typedef	A typedef
	"
	" *Special	any special symbol
	"  SpecialChar	special character in a constant
	"  Tag		you can use CTRL-] on this
	"  Delimiter	character that needs attention
	"  SpecialComment	special things inside a comment
	"  Debug		debugging statements
	"
	" *Underlined	text that stands out, HTML links
	"
	" *Ignore		left blank, hidden  |hl-Ignore|
	"
	" *Error		any erroneous construct
	"
	" *Todo		anything that needs extra attention; mostly the
	" 		keywords TODO FIXME and XXX


if exists("b:current_syntax")
    finish
endif

syn keyword wopKeyword use alias
syn match wopOperator "[=\|\*(\-\>)|(\:\:)|\:]"
syn match wopString '\"[^\"]*\"'
syn match wopIdentifier "[a-zA-Z_]\w*"
syn match wopFunction "%"
syn match wopComment "//.*"

hi def link wopKeyword Keyword
hi def link wopString String
hi def link wopOperator Operator
hi def link wopIdentifier Identifier
hi def link wopComment Comment
hi def link wopFunction Function
