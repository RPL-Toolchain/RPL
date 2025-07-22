//@ revisions: inline regular
//@[inline] compile-flags: -Z inline-mir=true
//@[regular] compile-flags: -Z inline-mir=false

use std::slice;

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn copy_from_raw_parts_iterator_next<I, IU>(mut src: I, mut src_linesize: IU)
//~^ERROR: it is unsound to trust pointers from passed-in iterators in a public safe function
where
    I: Iterator<Item = *const u8>,
    IU: Iterator<Item = usize>,
{
    let rr = src.next().unwrap();
    let s_linesize = src_linesize.next().unwrap();
    let ss = unsafe { slice::from_raw_parts(rr, s_linesize) };
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn copy_from_raw_parts_for_iterator<I, IU>(mut src: I, mut src_linesize: IU)
where
    I: Iterator<Item = *const u8>,
    IU: Iterator<Item = usize>,
{
    for rr in src {
        let s_linesize = src_linesize.next().unwrap();
        let ss = unsafe { slice::from_raw_parts(rr, s_linesize) };
        //FIXME: detect this case
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn copy_from_raw_parts_for_iterator_zip<I, IU>(mut src: I, mut src_linesize: IU)
where
    I: Iterator<Item = *const u8>,
    IU: Iterator<Item = usize>,
{
    for (rr, s_linesize) in src.zip(src_linesize) {
        let ss = unsafe { slice::from_raw_parts(rr, s_linesize) };
        // ...
        //FIXME: detect this case
    }
}

fn main() {
    panic!("Todo.");
}
