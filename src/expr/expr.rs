#![crate_name = "expr"]

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Roman Gafiyatullin <r.gafiyatullin@me.com>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

extern crate getopts;
extern crate libc;

#[macro_use]
extern crate uucore;

mod tokens;
mod syntax_tree;

use std::io::{Write};

static NAME: &'static str = "expr";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn uumain(args: Vec<String>) -> i32 {
	// For expr utility we do not want getopts.
	// The following usage should work without escaping hyphens: `expr -15 = 1 +  2 \* \( 3 - -4 \)`

	if maybe_handle_help_or_version( &args ) { 0 }
	else {
		let token_strings = args[1..].to_vec();

		match process_expr( &token_strings ) {
			Ok( expr_result ) => print_expr_ok( &expr_result ),
			Err( expr_error ) => print_expr_error( &expr_error )
		}
	}
}

fn process_expr( token_strings: &Vec<String> ) -> Result< String, String > {
	let maybe_tokens = tokens::strings_to_tokens( &token_strings );
	let maybe_ast = syntax_tree::tokens_to_ast( maybe_tokens );
	let maybe_result = evaluate_ast( maybe_ast );

	maybe_result
}

fn print_expr_ok( expr_result: &String ) -> i32 {
	println!("{}", expr_result);
	if expr_result == "0" || expr_result == "" { 1 }
	else { 0 }
}

fn print_expr_error( expr_error: &String ) -> ! {
	crash!(2, "{}", expr_error)
}

fn evaluate_ast( maybe_ast: Result<Box<syntax_tree::ASTNode>, String> ) -> Result<String, String> {
	if maybe_ast.is_err() { Err( maybe_ast.err().unwrap() ) }
	else { maybe_ast.ok().unwrap().evaluate() }
}



fn maybe_handle_help_or_version( args: &Vec<String> ) -> bool {
	if args.len() == 2 {
		if args[1] == "--help" { print_help(); true }
		else if args[1] == "--version" { print_version(); true }
		else { false }
	}
	else { false }
}

fn print_help() {
	//! The following is taken from GNU coreutils' "expr --help" output.
	print!(
r#"Usage: expr EXPRESSION
  or:  expr OPTION

      --help       display this help and exit
      --version    output version information and exit

Print the value of EXPRESSION to standard output.  A blank line below
separates increasing precedence groups.  EXPRESSION may be:

  ARG1 | ARG2       ARG1 if it is neither null nor 0, otherwise ARG2

  ARG1 & ARG2       ARG1 if neither argument is null or 0, otherwise 0

  ARG1 < ARG2       ARG1 is less than ARG2
  ARG1 <= ARG2      ARG1 is less than or equal to ARG2
  ARG1 = ARG2       ARG1 is equal to ARG2
  ARG1 != ARG2      ARG1 is unequal to ARG2
  ARG1 >= ARG2      ARG1 is greater than or equal to ARG2
  ARG1 > ARG2       ARG1 is greater than ARG2

  ARG1 + ARG2       arithmetic sum of ARG1 and ARG2
  ARG1 - ARG2       arithmetic difference of ARG1 and ARG2

  ARG1 * ARG2       arithmetic product of ARG1 and ARG2
  ARG1 / ARG2       arithmetic quotient of ARG1 divided by ARG2
  ARG1 % ARG2       arithmetic remainder of ARG1 divided by ARG2

  STRING : REGEXP   [NOT IMPLEMENTED] anchored pattern match of REGEXP in STRING

  match STRING REGEXP        [NOT IMPLEMENTED] same as STRING : REGEXP
  substr STRING POS LENGTH   substring of STRING, POS counted from 1
  index STRING CHARS         index in STRING where any CHARS is found, or 0
  length STRING              length of STRING
  + TOKEN                    interpret TOKEN as a string, even if it is a
                               keyword like 'match' or an operator like '/'

  ( EXPRESSION )             value of EXPRESSION

Beware that many operators need to be escaped or quoted for shells.
Comparisons are arithmetic if both ARGs are numbers, else lexicographical.
Pattern matches return the string matched between \( and \) or null; if
\( and \) are not used, they return the number of characters matched or 0.

Exit status is 0 if EXPRESSION is neither null nor 0, 1 if EXPRESSION is null
or 0, 2 if EXPRESSION is syntactically invalid, and 3 if an error occurred.

Environment variables:
	* EXPR_DEBUG_TOKENS=1   dump expression's tokens
	* EXPR_DEBUG_RPN=1      dump expression represented in reverse polish notation
	* EXPR_DEBUG_SYA_STEP=1 dump each parser step
	* EXPR_DEBUG_AST=1      dump expression represented abstract syntax tree
"#
	);
}

fn print_version() {
	println!("{} {}", NAME, VERSION);
}

#[allow(dead_code)]
fn main() {
    std::process::exit(uumain(std::env::args().collect()));
}
