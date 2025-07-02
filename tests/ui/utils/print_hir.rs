#![feature(stmt_expr_attributes)]

#[rpl::print_hir] //~ ERROR: abort due to debugging
//~^ HELP: remove this attribute
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
use std::sync::Arc; //~ NOTE: use std::sync::Arc;

#[rpl::print_hir] //~ ERROR: abort due to debugging
//~^ HELP: remove this attribute
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
mod m {
    //~^ NOTE: mod m {
    pub fn foo() {}
}

#[rpl::print_hir] //~ ERROR: abort due to debugging
//~^ HELP: remove this attribute
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
trait Foo {
    //~^ NOTE: trait Foo {
    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    const N: usize; //~ NOTE: const N: usize;
}

#[rpl::print_hir] //~ ERROR: abort due to debugging
//~^ HELP: remove this attribute
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
impl Foo for () {
    //~^ NOTE: impl Foo for () {
    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    const N: usize = 0_usize; //~ NOTE: const N: usize = 0usize;
}

#[rpl::print_hir] //~ ERROR: abort due to debugging
//~^ HELP: remove this attribute
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
fn main() {
    //~^ NOTE: fn main() {
    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    let x = Arc::new(0_usize); //~ NOTE: let x = Arc::new(0usize);

    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    fn foo() {
        //~^ NOTE: fn foo() {
        #[rpl::print_hir] //~ ERROR: abort due to debugging
        //~^ HELP: remove this attribute
        //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
        //~| NOTE: this error is to remind you removing these attributes
        {} //~ NOTE: { }
    }

    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    if true {
        //~^ NOTE: if true {
    } else {
    }

    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    std::thread::spawn(move || {
        //~^ NOTE: std::thread::spawn(move ||
        println!("{x}");
    });

    #[rpl::print_hir] //~ ERROR: abort due to debugging
    //~^ HELP: remove this attribute
    //~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    macro_rules! mac {
        //~^ NOTE: macro_rules! mac {
        () => {
            #[rpl::print_hir] // No effect after expansion.
            println!("test");
        };
    }

    #[rpl::print_hir] // No effect after expansion.
    mac!();
}
