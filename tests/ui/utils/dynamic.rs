#[rpl::dynamic(
    primary_message = "Dynamic RPL pattern",
    help = "You can use `#[rpl::dynamic]` to create a customizable lint.",
    note = "This is a dynamic RPL pattern, which can be customized during runtime."
)]
fn main() {
    //~^ERROR: Dynamic RPL pattern
    //~|HELP: You can use `#[rpl::dynamic]` to create a customizable lint.
    //~|NOTE: This is a dynamic RPL pattern, which can be customized during runtime.
    //~|NOTE: `#[forbid(rpl::dynamic)]` on by default
    // This function is used to trigger the dynamic pattern check.
    // It does not need to do anything.
}
