//@ignore-on-host
//@compile-flags: -Z deduplicate-diagnostics=yes
use cassandra_cpp_sys::CassIterator as _CassIterator;
use cassandra_cpp_sys::{
    cass_false, cass_iterator_get_aggregate_meta, cass_iterator_next, cass_true,
    CassAggregateMeta as _CassAggregateMeta,
};

use std::iter::Iterator;

pub struct AggregateMeta(*const _CassAggregateMeta);

impl AggregateMeta {
    fn build(inner: *const _CassAggregateMeta) -> Self {
        if inner.is_null() {
            panic!("Unexpected null pointer")
        };
        AggregateMeta(inner)
    }
}

#[derive(Debug)]
pub struct AggregateIterator(*mut _CassIterator);

impl Iterator for AggregateIterator {
    type Item = AggregateMeta;
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            match cass_iterator_next(self.0) {
                //~^ ERROR: it will be an undefined behavior to pass a pointer returned by `cass_iterator_next` to `cass_iterator_get_*` in a `std::iter::Iterator` implementation
                //~| HELP: consider implementing a `LendingIterator` instead
                //~| NOTE: `#[deny(cassandra_iter_next_ptr_passed_to_cass_iter_get)]` on by default
                cass_false => None,
                cass_true => {
                    let field_value = cass_iterator_get_aggregate_meta(self.0);
                    Some(AggregateMeta::build(field_value))
                }
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
