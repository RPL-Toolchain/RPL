rpl_utils_abort_due_to_debugging = abort due to debugging
    .note = `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    .remove_note = this error is to remind you removing these attributes

rpl_utils_abort_due_to_debugging_sugg = remove this attribute

rpl_utils_error_due_to_debugging = abort due to debugging
    .note = `#[rpl::dump_hir]`, `#[rpl::print_hir]` and `#[rpl::dump_mir]` are only used for debugging
    .remove_note = this error is to remind you removing these attributes
    .suggestion = remove this attribute

rpl_utils_dump_or_print_diag = {$message}
    .label = {$kind ->
        [dump_hir] HIR dumped
        [print_hir] HIR printed
        *[other] {""}
    } because of this attribute

rpl_utils_dump_mir = MIR of `{$def_id}`
    .label = MIR dumped because of this attribute

rpl_utils_dump_mir_block = {$block}

rpl_utils_dump_mir_file = see `{$file}` for dumped {$content}

rpl_utils_dump_mir_locals_and_source_scopes = locals and scopes in this MIR

rpl_utils_dump_mir_not_available = MIR of `{$instance}` is not available

rpl_utils_dump_mir_not_fn_path = expect a function path

rpl_utils_dump_mir_invalid = `#[rpl::dump_mir]` cannot be used here

rpl_utils_dump_mir_expect_init = expect an initialization
    .suggestion = try add an initialization