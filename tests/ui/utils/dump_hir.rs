//@ normalize-stderr-test: "\d+:\d+ ~ (\w+)\[[0-9a-f]{4}\]" -> "$1"
//@ normalize-stderr-test: "/.*/lib/rustlib/src/rust/library" -> "$$SRC_DIR"
#![feature(stmt_expr_attributes)]

#[rpl::dump_hir] //~ ERROR: abort due to debugging
//~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
use std::sync::Arc; //~ NOTE: Item

#[rpl::dump_hir] //~ ERROR: abort due to debugging
//~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
mod m {
    //~^ NOTE: Item
    pub fn foo() {}
}

#[rpl::dump_hir] //~ ERROR: abort due to debugging
//~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
trait Foo {
    //~^ NOTE: Item
    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    const N: usize; //~ NOTE: Item
}

#[rpl::dump_hir] //~ ERROR: abort due to debugging
//~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
impl Foo for () {
    //~^ NOTE: Item
    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    const N: usize = 0_usize; //~ NOTE: Item
}

#[rpl::dump_hir] //~ ERROR: abort due to debugging
//~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
fn main() {
    //~^ NOTE: Item
    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    let x = Arc::new(0_usize); //~ NOTE: Stmt

    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    fn foo() {
        //~^ NOTE: Item
        #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
        {} //~ NOTE: Expr
    }

    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    if true {
        //~^ NOTE: Expr
    } else {
    }

    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    std::thread::spawn(move || {
        //~^ NOTE: Expr
        println!("{x}");
    });

    #[rpl::dump_hir] //~ ERROR: abort due to debugging
    //~^ NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    //~| NOTE: this error is to remind you removing these attributes
    //~| HELP: remove this attribute
    macro_rules! mac {
        //~^ NOTE: Item
        () => {
            #[rpl::dump_hir] // No effect after expansion.
            println!("test");
        };
    }

    #[rpl::dump_hir] // No effect after expansion.
    mac!();
}
