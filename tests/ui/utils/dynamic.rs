#[rpl::dynamic(
    primary_message = "{$kind} {$lang} pattern",
    args(kind = "Dynamic", lang = "RPL"),
    help = "You can use `#[rpl::dynamic]` to create a customizable lint.",
    note = "This is a dynamic RPL pattern, which can be customized during runtime."
)]
fn main() {
    //~^ERROR: Dynamic RPL pattern
    //~|HELP: You can use `#[rpl::dynamic]` to create a customizable lint.
    //~|NOTE: This is a dynamic RPL pattern, which can be customized during runtime.
    // This function is used to trigger the dynamic pattern check.
    // It does not need to do anything.
}
