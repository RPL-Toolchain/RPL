#[rpl::dynamic(
    primary_message = "Dynamic RPL pattern",
    help = "You can use `#[rpl::dynamic]` to create a customizable lint.",
    note = "This is a dynamic RPL pattern, which can be customized during runtime."
)]
fn f1() {
    //~^ERROR: Dynamic RPL pattern
    //~|HELP: You can use `#[rpl::dynamic]` to create a customizable lint.
    //~|NOTE: This is a dynamic RPL pattern, which can be customized during runtime.
    //~|NOTE: `#[deny(rpl::dynamic)]` on by default
}

#[rpl::dynamic(
    primary_message = "Dynamic RPL pattern",
    unknown_attribute = "what's this?" 
    //~^ERROR: Unknown attribute key
    //~|NOTE: Allowed attribute keys are: `primary_message`, `labels`, `note`, `help`
)]
fn f2() {}

#[rpl::dynamic(help = "HELP!")] //~ERROR: Missing primary message
fn f3() {}

#[rpl::dynamic(primary_message = "test", help = 1)] //~ERROR: Expected a string value
fn f4() {}
