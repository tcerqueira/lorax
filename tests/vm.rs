const BACKEND: rlox::test_utils::Backend = rlox::test_utils::Backend::Vm;

// Shadow lox_tests! to blanket-ignore all VM tests since the VM compiler is not yet implemented.
macro_rules! lox_tests {
    ($category:literal, [$($(#[$attr:meta])* $name:ident),* $(,)?]) => {
        $(
            #[test]
            #[ignore = "VM not yet implemented"]
            fn $name() {
                rlox::test_utils::run_test(
                    env!("CARGO_BIN_EXE_rlox"),
                    super::BACKEND,
                    concat!("tests/sources/", $category, "/", stringify!($name), ".lox"),
                );
            }
        )*
    };
    ([$($(#[$attr:meta])* $name:ident),* $(,)?]) => {
        $(
            #[test]
            #[ignore = "VM not yet implemented"]
            fn $name() {
                rlox::test_utils::run_test(
                    env!("CARGO_BIN_EXE_rlox"),
                    super::BACKEND,
                    concat!("tests/sources/", stringify!($name), ".lox"),
                );
            }
        )*
    };
}

mod assignment {
    lox_tests!("assignment", [
        associativity,
        global,
        grouping,
        infix_operator,
        local,
        prefix_operator,
        syntax,
        to_this,
        undefined,
    ]);
}

mod block {
    lox_tests!("block", [
        empty,
        scope,
    ]);
}

mod bool_ {
    lox_tests!("bool", [
        equality,
        not,
    ]);
}

mod call {
    lox_tests!("call", [
        bool,
        nil,
        num,
        object,
        string,
    ]);
}

mod class {
    lox_tests!("class", [
        empty,
        inherit_self,
        inherited_method,
        local_inherit_other,
        local_inherit_self,
        local_reference_self,
        reference_self,
    ]);
}

mod closure {
    lox_tests!("closure", [
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
    ]);
}

mod comments {
    lox_tests!("comments", [
        line_at_eof,
        only_line_comment,
        only_line_comment_and_line,
        unicode,
    ]);
}

mod constructor {
    lox_tests!("constructor", [
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
    ]);
}

mod expressions {
    lox_tests!("expressions", [
        evaluate,
        parse,
    ]);
}

mod root {
    lox_tests!([empty_file, precedence, unexpected_character]);
}

mod field {
    lox_tests!("field", [
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
    ]);
}

mod for_loop {
    lox_tests!("for", [
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
    ]);
}

mod function {
    lox_tests!("function", [
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
    ]);
}

mod if_stmt {
    lox_tests!("if", [
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
    ]);
}

mod inheritance {
    lox_tests!("inheritance", [
        constructor,
        inherit_from_function,
        inherit_from_nil,
        inherit_from_number,
        inherit_methods,
        parenthesized_superclass,
        set_fields_from_base_class,
    ]);
}

mod limit {
    lox_tests!("limit", [
        loop_too_large,
        no_reuse_constants,
        stack_overflow,
        too_many_constants,
        too_many_locals,
        too_many_upvalues,
    ]);
}

mod logical_operator {
    lox_tests!("logical_operator", [
        and,
        and_truth,
        or,
        or_truth,
    ]);
}

mod method {
    lox_tests!("method", [
        arity,
        empty_block,
        extra_arguments,
        missing_arguments,
        not_found,
        print_bound_method,
        refer_to_name,
        too_many_arguments,
        too_many_parameters,
    ]);
}

mod nil_ {
    lox_tests!("nil", [
        literal,
    ]);
}

mod number {
    lox_tests!("number", [
        decimal_point_at_eof,
        leading_dot,
        literals,
        nan_equality,
        trailing_dot,
    ]);
}

mod operator {
    lox_tests!("operator", [
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
    ]);
}

mod scanning {
    lox_tests!("scanning", [
        identifiers,
        keywords,
        numbers,
        punctuators,
        strings,
        whitespace,
    ]);
}

mod print_ {
    lox_tests!("print", [
        missing_argument,
    ]);
}

mod regression {
    #[test]
    #[ignore = "VM not yet implemented"]
    fn regression_394() {
        rlox::test_utils::run_test(env!("CARGO_BIN_EXE_rlox"), super::BACKEND, "tests/sources/regression/394.lox");
    }

    #[test]
    #[ignore = "VM not yet implemented"]
    fn regression_40() {
        rlox::test_utils::run_test(env!("CARGO_BIN_EXE_rlox"), super::BACKEND, "tests/sources/regression/40.lox");
    }
}

mod return_stmt {
    lox_tests!("return", [
        after_else,
        after_if,
        after_while,
        at_top_level,
        in_function,
        in_method,
        return_nil_if_no_value,
    ]);
}

mod string {
    lox_tests!("string", [
        error_after_multiline,
        literals,
        multiline,
        unterminated,
    ]);
}

mod super_ {
    lox_tests!("super", [
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
    ]);
}

mod this_ {
    lox_tests!("this", [
        closure,
        nested_class,
        nested_closure,
        this_at_top_level,
        this_in_method,
        this_in_top_level_function,
    ]);
}

mod variable {
    lox_tests!("variable", [
        collide_with_parameter,
        duplicate_local,
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
        undefined_global,
        undefined_local,
        uninitialized,
        unreached_undefined,
        use_false_as_var,
        use_global_in_initializer,
        use_local_in_initializer,
        use_nil_as_var,
        use_this_as_var,
    ]);
}

mod while_loop {
    lox_tests!("while", [
        class_in_body,
        closure_in_body,
        fun_in_body,
        return_closure,
        return_inside,
        syntax,
        var_in_body,
    ]);
}
