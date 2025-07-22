//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
//@[inline]check-pass: no pattern available
#![expect(clippy::missing_transmute_annotations)]

#[derive(Debug)]
#[repr(u8)]
#[expect(dead_code)]
enum Opcode {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}

fn base_case(op: u8) -> Option<Opcode> {
    (op < 4).then_some(unsafe { std::mem::transmute::<_, Opcode>(op) })
    //~[normal]^ eager_transmute
}

fn cross_function_cond(op: u8) -> Option<Opcode> {
    fn check_op(op: u8) -> bool {
        op < 4
    }
    check_op(op).then_some(unsafe { std::mem::transmute::<_, Opcode>(op) })
    //~[normal]^ eager_transmute
}

fn cross_function_value(op: u8) -> Option<Opcode> {
    unsafe fn cvt_op(op: u8) -> Opcode {
        unsafe { std::mem::transmute(op) }
    }
    (op < 4).then_some(unsafe { cvt_op(op) })
    //FIXME: ~^ eager_transmute
}

fn cross_statement_cond(op: u8) -> Option<Opcode> {
    let cond = op < 4;
    cond.then_some(unsafe { std::mem::transmute::<_, Opcode>(op) })
    //~[normal]^ eager_transmute
}

#[cfg_attr(test, test)]
fn base_case_test() {
    dbg!(base_case(0));
    dbg!(base_case(4));
}

#[cfg_attr(test, test)]
fn cross_function_cond_test() {
    dbg!(cross_function_cond(0));
    dbg!(cross_function_cond(4));
}

#[cfg_attr(test, test)]
fn cross_function_value_test() {
    dbg!(cross_function_value(0));
    dbg!(cross_function_value(4));
}

#[cfg_attr(test, test)]
fn cross_statement_cond_test() {
    dbg!(cross_statement_cond(0));
    dbg!(cross_statement_cond(4));
}

pub(crate) fn main() {
    base_case_test();
    cross_function_cond_test();
    cross_function_value_test();
    cross_statement_cond_test();
}
