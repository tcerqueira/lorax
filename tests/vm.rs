const BACKEND: rlox::test_utils::Backend = rlox::test_utils::Backend::Vm;

// The VM implements the full Lox language; the only remaining `#[ignore]`s are
// intentional semantic deviations from the book (each annotated with why).
// Note the test runner only checks that *some* error occurred for error-cases,
// not the exact one — so enable a test only once the feature is really there,
// never just because an error happens to fire.

#[test]
fn examples() {
    rlox::test_utils::run_examples(BACKEND, "examples");
}

mod assignment {
    rlox::lox_tests!(
        "assignment",
        [
            associativity,
            global,
            grouping,
            infix_operator,
            local,
            prefix_operator,
            syntax,
            to_this,
            undefined,
        ]
    );
}

mod block {
    rlox::lox_tests!("block", [empty, scope,]);
}

mod bool_ {
    rlox::lox_tests!("bool", [equality, not,]);
}

mod call {
    rlox::lox_tests!("call", [bool, nil, num, object, string,]);
}

mod class {
    rlox::lox_tests!(
        "class",
        [
            empty,
            inherit_self,
            inherited_method,
            local_inherit_other,
            local_inherit_self,
            local_reference_self,
            reference_self,
        ]
    );
}

mod closure {
    rlox::lox_tests!(
        "closure",
        [
            assign_to_closure,
            assign_to_shadowed_later,
            close_over_function_parameter,
            close_over_later_variable,
            close_over_method_parameter,
            closed_closure_in_function,
            nested_closure,
            open_closure_in_function,
            reference_closure_multiple_times,
            reuse_closure_slot,
            shadow_closure_with_local,
            unused_closure,
            unused_later_closure,
        ]
    );
}

mod comments {
    rlox::lox_tests!(
        "comments",
        [
            line_at_eof,
            only_line_comment,
            only_line_comment_and_line,
            unicode,
        ]
    );
}

mod constructor {
    rlox::lox_tests!(
        "constructor",
        [
            arguments,
            call_init_early_return,
            call_init_explicitly,
            default,
            default_arguments,
            early_return,
            extra_arguments,
            init_not_method,
            missing_arguments,
            return_in_nested_function,
            return_value,
        ]
    );
}

mod expressions {
    rlox::lox_tests!(
        "expressions",
        [
            evaluate,
            #[ignore = "lorax has no AST-dump mode (single-pass compiler, no tree)"]
            parse,
        ]
    );
}

mod root {
    rlox::lox_tests!([empty_file, precedence, unexpected_character]);
}

mod field {
    rlox::lox_tests!(
        "field",
        [
            call_function_field,
            call_nonfunction_field,
            get_and_set_method,
            get_on_bool,
            get_on_class,
            get_on_function,
            get_on_nil,
            get_on_num,
            get_on_string,
            many,
            method,
            method_binds_this,
            on_instance,
            set_evaluation_order,
            set_on_bool,
            set_on_class,
            set_on_function,
            set_on_nil,
            set_on_num,
            set_on_string,
            undefined,
        ]
    );
}

mod for_loop {
    rlox::lox_tests!(
        "for",
        [
            class_in_body,
            closure_in_body,
            fun_in_body,
            return_closure,
            return_inside,
            scope,
            statement_condition,
            statement_increment,
            statement_initializer,
            syntax,
            var_in_body,
        ]
    );
}

mod function {
    rlox::lox_tests!(
        "function",
        [
            body_must_be_block,
            empty_body,
            extra_arguments,
            local_mutual_recursion,
            local_recursion,
            missing_arguments,
            missing_comma_in_parameters,
            mutual_recursion,
            nested_call_with_arguments,
            parameters,
            print,
            recursion,
            too_many_arguments,
            too_many_parameters,
        ]
    );
}

mod if_stmt {
    rlox::lox_tests!(
        "if",
        [
            class_in_else,
            class_in_then,
            dangling_else,
            r#else,
            fun_in_else,
            fun_in_then,
            r#if,
            truth,
            var_in_else,
            var_in_then,
        ]
    );
}

mod inheritance {
    rlox::lox_tests!(
        "inheritance",
        [
            constructor,
            inherit_from_function,
            inherit_from_nil,
            inherit_from_number,
            inherit_methods,
            parenthesized_superclass,
            set_fields_from_base_class,
        ]
    );
}

mod limit {
    rlox::lox_tests!(
        "limit",
        [
            loop_too_large,
            #[ignore = "lorax deviates: constants are deduplicated"]
            no_reuse_constants,
            stack_overflow,
            too_many_constants,
            too_many_locals,
            too_many_upvalues,
        ]
    );
}

mod logical_operator {
    rlox::lox_tests!("logical_operator", [and, and_truth, or, or_truth,]);
}

mod method {
    rlox::lox_tests!(
        "method",
        [
            arity,
            empty_block,
            extra_arguments,
            missing_arguments,
            not_found,
            print_bound_method,
            refer_to_name,
            too_many_arguments,
            too_many_parameters,
        ]
    );
}

mod nil_ {
    rlox::lox_tests!("nil", [literal,]);
}

mod number {
    rlox::lox_tests!(
        "number",
        [
            decimal_point_at_eof,
            leading_dot,
            literals,
            nan_equality,
            trailing_dot,
        ]
    );
}

mod operator {
    rlox::lox_tests!(
        "operator",
        [
            add,
            add_bool_nil,
            add_bool_num,
            add_bool_string,
            add_nil_nil,
            add_num_nil,
            add_string_nil,
            comparison,
            divide,
            divide_nonnum_num,
            divide_num_nonnum,
            equals,
            equals_class,
            equals_method,
            greater_nonnum_num,
            greater_num_nonnum,
            greater_or_equal_nonnum_num,
            greater_or_equal_num_nonnum,
            less_nonnum_num,
            less_num_nonnum,
            less_or_equal_nonnum_num,
            less_or_equal_num_nonnum,
            multiply,
            multiply_nonnum_num,
            multiply_num_nonnum,
            negate,
            negate_nonnum,
            not,
            not_class,
            not_equals,
            subtract,
            subtract_nonnum_num,
            subtract_num_nonnum,
        ]
    );
}

mod scanning {
    rlox::lox_tests!(
        "scanning",
        [
            identifiers,
            keywords,
            numbers,
            punctuators,
            strings,
            whitespace,
        ]
    );
}

mod print_ {
    rlox::lox_tests!("print", [missing_argument,]);
}

mod regression {
    #[test]
    fn regression_394() {
        rlox::test_utils::run_test(
            env!("CARGO_BIN_EXE_rlox"),
            super::BACKEND,
            "tests/sources/regression/394.lox",
        );
    }

    #[test]
    fn regression_40() {
        rlox::test_utils::run_test(
            env!("CARGO_BIN_EXE_rlox"),
            super::BACKEND,
            "tests/sources/regression/40.lox",
        );
    }
}

mod return_stmt {
    rlox::lox_tests!(
        "return",
        [
            after_else,
            after_if,
            after_while,
            at_top_level,
            in_function,
            in_method,
            return_nil_if_no_value,
        ]
    );
}

mod string {
    rlox::lox_tests!(
        "string",
        [error_after_multiline, literals, multiline, unterminated,]
    );
}

mod super_ {
    rlox::lox_tests!(
        "super",
        [
            bound_method,
            call_other_method,
            call_same_method,
            closure,
            constructor,
            extra_arguments,
            indirectly_inherited,
            missing_arguments,
            no_superclass_bind,
            no_superclass_call,
            no_superclass_method,
            parenthesized,
            reassign_superclass,
            super_at_top_level,
            super_in_closure_in_inherited_method,
            super_in_inherited_method,
            super_in_top_level_function,
            super_without_dot,
            super_without_name,
            this_in_superclass_method,
        ]
    );
}

mod this_ {
    rlox::lox_tests!(
        "this",
        [
            closure,
            nested_class,
            nested_closure,
            this_at_top_level,
            this_in_method,
            this_in_top_level_function,
        ]
    );
}

mod variable {
    rlox::lox_tests!(
        "variable",
        [
            #[ignore = "lorax deviates: same-scope shadowing is legal"]
            collide_with_parameter,
            #[ignore = "lorax deviates: same-scope shadowing is legal"]
            duplicate_local,
            #[ignore = "lorax deviates: same-scope shadowing is legal"]
            duplicate_parameter,
            early_bound,
            in_middle_of_block,
            in_nested_block,
            local_from_method,
            redeclare_global,
            redefine_global,
            scope_reuse_in_different_blocks,
            shadow_and_local,
            shadow_global,
            shadow_local,
            // lorax-specific shadowing tests.
            shadow_same_scope,
            shadow_use_previous,
            undefined_global,
            undefined_local,
            uninitialized,
            unreached_undefined,
            use_false_as_var,
            use_global_in_initializer,
            #[ignore = "lorax deviates: initializer can reference previous binding"]
            use_local_in_initializer,
            use_nil_as_var,
            use_this_as_var,
        ]
    );
}

mod while_loop {
    rlox::lox_tests!(
        "while",
        [
            class_in_body,
            closure_in_body,
            fun_in_body,
            return_closure,
            return_inside,
            syntax,
            var_in_body,
        ]
    );
}
