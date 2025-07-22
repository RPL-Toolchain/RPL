# TODOs

-   The refactor of the RPL frontend using the pest-parser-generator (pass2 and interface);
-   Treat struct patterns as a meta variable, make them involved in the pattern matching algorithm;
-   Projection of place meta variables;
-   Fix the bug that `rdep_start_end` carries locals that have been consumed;
-   Predicates;
-   `match_stmt_locals` treat `copy` and `move` on a `Copy`-type differently.
-   When `-Z inline-mir` is on, for a type meta variable `$T`, some specific functions related to `$T` may be inlined, and cannot be recognized in MIR any more.

    For example, there are two Rust code snippets that contain double `drop` on a `ManuallyDrop`, and we'd like to write a pattern to match both of them:

    - One is a `String` type:

        ```rust
        #[rpl::dump_mir(dump_cfg, dump_ddg)]
        fn double_drop_string() {
            let mut s = ManuallyDrop::new("1".to_owned());
            unsafe {
                ManuallyDrop::drop(&mut s);
                ManuallyDrop::drop(&mut s);
            }
        }
        ```

        ![CFG of `double_drop_string`](./inline_test_1.svg)

    - The other is a generic function that can take any type:

        ```rust
        #[rpl::dump_mir(dump_cfg, dump_ddg)]
        fn double_drop<T>(value: T) {
            let mut s = ManuallyDrop::new(value);
            unsafe {
                ManuallyDrop::drop(&mut s);
                ManuallyDrop::drop(&mut s);
            }
        }
        ```

        ![CFG of `double_drop`](./inline_test_3.svg)

    As they are inlined in different ways, we can't write a usable pattern to detect both of them. What's worse, the first code snippet is inlined in a way that the inner implementation detail of `ManuallyDrop<String>` is involved.

    Currently the only reasonable solution that I can think of is to replace the type meta variable `$T` with concrete types, try to inline function calls in the pattern, and re-scan using new patterns if any function calls in the pattern are inlined. Then the remained question is that which concrete types should be used to replace `$T`? I can think of two possible solutions:

    - For each concrete type `T1` involved in the crate, replace `$T` with `T1` and try inlining the resulting pattern.
    - For each candidate type `T1` of `$T` in the original pattern, replace `$T` with `T1` and try inlining the resulting pattern. This relies on the assumption that candidates types include all (or at least the most) of possible types that `$T` can be instantiated with.

        This is a reasonable assumption, as for those statements that do not rely on the actual type of `$T`, candidate types of `$T` are the only available types. However, this may not be true for all cases, and we need to be careful about it. For example, if all statements maybe affected by the actual type of `$T`, then this solution won't work. In this case, we can use the first solution instead.
