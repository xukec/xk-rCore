#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle] //禁止编译器混淆，导致链接失败
fn main() -> i32 {
    println!("Hello, world!");
    0
}