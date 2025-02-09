#![crate_name= "true"]

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Jordi Boggiano <j.boggiano@seld.be>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */


pub fn uumain(_: Vec<String>) -> i32 {
    0
}

#[allow(dead_code)]
fn main() {
    std::process::exit(uumain(std::env::args().collect()));
}
