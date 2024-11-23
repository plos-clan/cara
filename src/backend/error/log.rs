/*
* @file    :   log.rs
* @time    :   2024/03/30 10:52:24
* @author  :   zzjcarrot
*/

use std::fmt;

pub enum Colors {
    Red,
    Blue,
    Yellow,
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        $crate::backend::_print(
            $crate::backend::Colors::Red,
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        $crate::_print(
            $crate::Colors::Yellow,
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        $crate::_print(
            $crate::Colors::Blue,
            format_args!($($arg)*)
        )
    )
}

#[inline]
pub fn _print(color: Colors, args: fmt::Arguments) {
    let s = args.to_string();

    match color {
        Colors::Red => println!("\x1b[7;31m 错误 \x1b[0m \x1b[31m{}\x1b[0m", s),
        Colors::Blue => println!("\x1b[7;36m 信息 \x1b[0m \x1b[36m{}\x1b[0m", s),
        Colors::Yellow => println!("\x1b[7;33m 警告 \x1b[0m \x1b[33m{}\x1b[0m", s),
    }
}
