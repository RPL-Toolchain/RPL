//@ compile-flags: -Z inline-mir=false
//@ normalize-stderr-test: "see `.*/mir_dump/(.*)` for dumpped" -> "see `./mir_dump/$1` for dumpped"
//@ normalize-stderr-test: "/.*/lib/rustlib/src/rust/library" -> "$$SRC_DIR"

#[rpl::dump_mir(dump_cfg, dump_ddg)]
//~^ ERROR: abort due to debugging
//~| NOTE: `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
//~| NOTE: this error is to remind you removing these attributes
//~| HELP: remove this attribute
fn test() {
    //~^ NOTE: MIR of `test`
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.test\.-------\.dump_mir\.\.mir` for dumpped MIR/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.test\.-------\.dump_mir\.\.mir\.cfg\.dot` for dumpped control flow graph/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.test\.-------\.dump_mir\.\.mir\.ddg\.dot` for dumpped data dependency graph/
    let mut arr: [u8; 20] = [1; 20];
    let mut j = 2;
    // FIXME: this note is not supposed to be here
    for i in (0..10).map(|i| i * 2).filter(|&i| i < 10) {
        //~^ NOTE: locals and scopes in this MIR
        //~| NOTE: bb0: {
        //~| NOTE: bb1: {
        //~| NOTE: bb2: {
        //~| NOTE: bb3: {
        //~| NOTE: bb4: {
        //~| NOTE: bb5: {
        //~| NOTE: bb6: {
        //~| NOTE: bb10: {
        arr[i] = arr[j];
        //~^ NOTE: bb7: {
        //~| NOTE: bb9: {
        j = j + 1;
    }
} //~ NOTE: bb8: {

fn f2() -> bool {
    unimplemented!()
}

#[rpl::dump_mir(dump_cfg, dump_ddg)] //~ HELP: remove this attribute
fn critical(i: i16, j: i16) -> Result<i16, ()> {
    //~^ NOTE: MIR of `critical`
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.critical\.-------\.dump_mir\.\.mir` for dumpped MIR/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.critical\.-------\.dump_mir\.\.mir\.cfg\.dot` for dumpped control flow graph/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.critical\.-------\.dump_mir\.\.mir\.ddg\.dot` for dumpped data dependency graph/
    //~| NOTE: locals and scopes in this MIR
    let result = 0_i16;
    let k = 3_i16 * i + j * j;
    if f2() {
        //~^ NOTE: bb0: {
        //~| NOTE: bb1: {
        if k > 0 {
            //~^ NOTE: bb2: {
            return Err(());
        }
    }
    Ok(result)
}
//~^ NOTE: bb3: {
//~| NOTE: bb4: {
//~| NOTE: bb5: {

fn main() {
    #[rpl::dump_mir(dump_cfg, dump_ddg)] //~ HELP: remove this attribute
    let _ = std::alloc::alloc;
    //~^ NOTE: MIR of `std::alloc::alloc`
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]alloc\.alloc-alloc\.-------\.dump_mir\.\.mir` for dumpped MIR/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]alloc\.alloc-alloc\.-------\.dump_mir\.\.mir\.cfg\.dot` for dumpped control flow graph/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]alloc\.alloc-alloc\.-------\.dump_mir\.\.mir\.ddg\.dot` for dumpped data dependency graph/
    //~| NOTE: locals and scopes in this MIR
    //~| NOTE: bb0: {
    //~| NOTE: bb1: {
    //~| NOTE: bb2: {
    //~| NOTE: bb3: {
    //~| NOTE: bb4: {
    //~| NOTE: bb5: {

    #[rpl::dump_mir(dump_cfg, dump_ddg)] //~ HELP: remove this attribute
    let _ = |x: i32| x + 1;
    //~^ NOTE: MIR of `main::{closure#0}`
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.main-\{closure#0\}\.-------\.dump_mir\.\.mir` for dumpped MIR/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.main-\{closure#0\}\.-------\.dump_mir\.\.mir\.cfg\.dot` for dumpped control flow graph/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]dump_mir\.main-\{closure#0\}\.-------\.dump_mir\.\.mir\.ddg\.dot` for dumpped data dependency graph/
    //~| NOTE: locals and scopes in this MIR
    //~| NOTE: bb0: {

    #[rpl::dump_mir(dump_cfg, dump_ddg)] //~ HELP: remove this attribute
    let _ = <std::result::IntoIter<&str> as std::iter::Iterator>::next;
    //~^ NOTE: MIR of `<std::result::IntoIter<T> as std::iter::Iterator>::next`
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]core\.result-\{impl#20\}-next\.-------\.dump_mir\.\.mir` for dumpped MIR/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]core\.result-\{impl#20\}-next\.-------\.dump_mir\.\.mir\.cfg\.dot` for dumpped control flow graph/
    //~| NOTE: /see `.*[/\\]mir_dump[/\\]core\.result-\{impl#20\}-next\.-------\.dump_mir\.\.mir\.ddg\.dot` for dumpped data dependency graph/
    //~| NOTE: locals and scopes in this MIR
    //~| NOTE: bb0: {
}
