const BACKEND: rlox::test_utils::Backend = rlox::test_utils::Backend::TreeWalk;

mod assignment {
    rlox::lox_tests!("assignment", [
        associativity,
        global,
        grouping,
        infix_operator,
        local,
        prefix_operator,
        syntax,
        #[ignore = "unimplemented: this"] to_this,
        undefined,
    ]);
}

mod block {
    rlox::lox_tests!("block", [
        empty,
        scope,
    ]);
}

mod bool_ {
    rlox::lox_tests!("bool", [
        equality,
        not,
    ]);
}

mod call {
    rlox::lox_tests!("call", [
        bool,
        nil,
        num,
        #[ignore = "unimplemented: classes"] object,
        string,
    ]);
}

mod class {
    rlox::lox_tests!("class", [
        #[ignore = "unimplemented: classes"] empty,
        #[ignore = "unimplemented: classes"] inherit_self,
        #[ignore = "unimplemented: classes"] inherited_method,
        #[ignore = "unimplemented: classes"] local_inherit_other,
        #[ignore = "unimplemented: classes"] local_inherit_self,
        #[ignore = "unimplemented: classes"] local_reference_self,
        #[ignore = "unimplemented: classes"] reference_self,
    ]);
}

mod closure {
    rlox::lox_tests!("closure", [
        assign_to_closure,
        assign_to_shadowed_later,
        close_over_function_parameter,
        close_over_later_variable,
        #[ignore = "unimplemented: classes"] close_over_method_parameter,
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
    rlox::lox_tests!("comments", [
        line_at_eof,
        only_line_comment,
        only_line_comment_and_line,
        unicode,
    ]);
}

mod constructor {
    rlox::lox_tests!("constructor", [
        #[ignore = "unimplemented: classes"] arguments,
        #[ignore = "unimplemented: classes"] call_init_early_return,
        #[ignore = "unimplemented: classes"] call_init_explicitly,
        #[ignore = "unimplemented: classes"] default,
        #[ignore = "unimplemented: classes"] default_arguments,
        #[ignore = "unimplemented: classes"] early_return,
        #[ignore = "unimplemented: classes"] extra_arguments,
        #[ignore = "unimplemented: classes"] init_not_method,
        #[ignore = "unimplemented: classes"] missing_arguments,
        #[ignore = "unimplemented: classes"] return_in_nested_function,
        #[ignore = "unimplemented: classes"] return_value,
    ]);
}

mod expressions {
    rlox::lox_tests!("expressions", [
        evaluate,
        #[ignore = "chapter-specific: AST printer mode"] parse,
    ]);
}

mod root {
    rlox::lox_tests!([empty_file, precedence, unexpected_character]);
}

mod field {
    rlox::lox_tests!("field", [
        #[ignore = "unimplemented: classes"] call_function_field,
        #[ignore = "unimplemented: classes"] call_nonfunction_field,
        #[ignore = "unimplemented: classes"] get_and_set_method,
        #[ignore = "unimplemented: classes"] get_on_bool,
        #[ignore = "unimplemented: classes"] get_on_class,
        #[ignore = "unimplemented: classes"] get_on_function,
        #[ignore = "unimplemented: classes"] get_on_nil,
        #[ignore = "unimplemented: classes"] get_on_num,
        #[ignore = "unimplemented: classes"] get_on_string,
        #[ignore = "unimplemented: classes"] many,
        #[ignore = "unimplemented: classes"] method,
        #[ignore = "unimplemented: classes"] method_binds_this,
        #[ignore = "unimplemented: classes"] on_instance,
        #[ignore = "unimplemented: classes"] set_evaluation_order,
        #[ignore = "unimplemented: classes"] set_on_bool,
        #[ignore = "unimplemented: classes"] set_on_class,
        #[ignore = "unimplemented: classes"] set_on_function,
        #[ignore = "unimplemented: classes"] set_on_nil,
        #[ignore = "unimplemented: classes"] set_on_num,
        #[ignore = "unimplemented: classes"] set_on_string,
        #[ignore = "unimplemented: classes"] undefined,
    ]);
}

mod for_loop {
    rlox::lox_tests!("for", [
        #[ignore = "unimplemented: classes"] class_in_body,
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
    rlox::lox_tests!("function", [
        body_must_be_block,
        empty_body,
        extra_arguments,
        #[ignore = "behavior: no forward-reference error in tree-walk"] local_mutual_recursion,
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
    rlox::lox_tests!("if", [
        #[ignore = "unimplemented: classes"] class_in_else,
        #[ignore = "unimplemented: classes"] class_in_then,
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
    rlox::lox_tests!("inheritance", [
        #[ignore = "unimplemented: classes"] constructor,
        #[ignore = "unimplemented: classes"] inherit_from_function,
        #[ignore = "unimplemented: classes"] inherit_from_nil,
        #[ignore = "unimplemented: classes"] inherit_from_number,
        #[ignore = "unimplemented: classes"] inherit_methods,
        #[ignore = "unimplemented: classes"] parenthesized_superclass,
        #[ignore = "unimplemented: classes"] set_fields_from_base_class,
    ]);
}

mod limit {
    rlox::lox_tests!("limit", [
        #[ignore = "VM-specific"] loop_too_large,
        #[ignore = "VM-specific"] no_reuse_constants,
        stack_overflow,
        #[ignore = "VM-specific"] too_many_constants,
        #[ignore = "VM-specific"] too_many_locals,
        #[ignore = "VM-specific"] too_many_upvalues,
    ]);
}

mod logical_operator {
    rlox::lox_tests!("logical_operator", [
        and,
        and_truth,
        or,
        or_truth,
    ]);
}

mod method {
    rlox::lox_tests!("method", [
        #[ignore = "unimplemented: classes"] arity,
        #[ignore = "unimplemented: classes"] empty_block,
        #[ignore = "unimplemented: classes"] extra_arguments,
        #[ignore = "unimplemented: classes"] missing_arguments,
        #[ignore = "unimplemented: classes"] not_found,
        #[ignore = "unimplemented: classes"] print_bound_method,
        #[ignore = "unimplemented: classes"] refer_to_name,
        #[ignore = "unimplemented: classes"] too_many_arguments,
        #[ignore = "unimplemented: classes"] too_many_parameters,
    ]);
}

mod nil_ {
    rlox::lox_tests!("nil", [
        literal,
    ]);
}

mod number {
    rlox::lox_tests!("number", [
        decimal_point_at_eof,
        leading_dot,
        literals,
        nan_equality,
        trailing_dot,
    ]);
}

mod operator {
    rlox::lox_tests!("operator", [
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
        #[ignore = "unimplemented: classes"] equals_class,
        #[ignore = "unimplemented: classes"] equals_method,
        #[ignore = "behavior: no operand type-check for comparisons"] greater_nonnum_num,
        #[ignore = "behavior: no operand type-check for comparisons"] greater_num_nonnum,
        #[ignore = "behavior: no operand type-check for comparisons"] greater_or_equal_nonnum_num,
        #[ignore = "behavior: no operand type-check for comparisons"] greater_or_equal_num_nonnum,
        #[ignore = "behavior: no operand type-check for comparisons"] less_nonnum_num,
        #[ignore = "behavior: no operand type-check for comparisons"] less_num_nonnum,
        #[ignore = "behavior: no operand type-check for comparisons"] less_or_equal_nonnum_num,
        #[ignore = "behavior: no operand type-check for comparisons"] less_or_equal_num_nonnum,
        multiply,
        multiply_nonnum_num,
        multiply_num_nonnum,
        negate,
        negate_nonnum,
        not,
        #[ignore = "unimplemented: classes"] not_class,
        not_equals,
        subtract,
        subtract_nonnum_num,
        subtract_num_nonnum,
    ]);
}

mod scanning {
    rlox::lox_tests!("scanning", [
        identifiers,
        keywords,
        numbers,
        punctuators,
        strings,
        whitespace,
    ]);
}

mod print_ {
    rlox::lox_tests!("print", [
        missing_argument,
    ]);
}

mod regression {
    #[test]
    #[ignore = "unimplemented: classes"]
    fn regression_394() {
        rlox::test_utils::run_test(env!("CARGO_BIN_EXE_rlox"), super::BACKEND, "tests/sources/regression/394.lox");
    }

    #[test]
    fn regression_40() {
        rlox::test_utils::run_test(env!("CARGO_BIN_EXE_rlox"), super::BACKEND, "tests/sources/regression/40.lox");
    }
}

mod return_stmt {
    rlox::lox_tests!("return", [
        after_else,
        after_if,
        after_while,
        at_top_level,
        in_function,
        #[ignore = "unimplemented: classes"] in_method,
        return_nil_if_no_value,
    ]);
}

mod string {
    rlox::lox_tests!("string", [
        error_after_multiline,
        literals,
        multiline,
        unterminated,
    ]);
}

mod super_ {
    rlox::lox_tests!("super", [
        #[ignore = "unimplemented: super"] bound_method,
        #[ignore = "unimplemented: super"] call_other_method,
        #[ignore = "unimplemented: super"] call_same_method,
        #[ignore = "unimplemented: super"] closure,
        #[ignore = "unimplemented: super"] constructor,
        #[ignore = "unimplemented: super"] extra_arguments,
        #[ignore = "unimplemented: super"] indirectly_inherited,
        #[ignore = "unimplemented: super"] missing_arguments,
        #[ignore = "unimplemented: super"] no_superclass_bind,
        #[ignore = "unimplemented: super"] no_superclass_call,
        #[ignore = "unimplemented: super"] no_superclass_method,
        #[ignore = "unimplemented: super"] parenthesized,
        #[ignore = "unimplemented: super"] reassign_superclass,
        #[ignore = "unimplemented: super"] super_at_top_level,
        #[ignore = "unimplemented: super"] super_in_closure_in_inherited_method,
        #[ignore = "unimplemented: super"] super_in_inherited_method,
        #[ignore = "unimplemented: super"] super_in_top_level_function,
        #[ignore = "unimplemented: super"] super_without_dot,
        #[ignore = "unimplemented: super"] super_without_name,
        #[ignore = "unimplemented: super"] this_in_superclass_method,
    ]);
}

mod this_ {
    rlox::lox_tests!("this", [
        #[ignore = "unimplemented: this"] closure,
        #[ignore = "unimplemented: this"] nested_class,
        #[ignore = "unimplemented: this"] nested_closure,
        #[ignore = "unimplemented: this"] this_at_top_level,
        #[ignore = "unimplemented: this"] this_in_method,
        #[ignore = "unimplemented: this"] this_in_top_level_function,
    ]);
}

mod variable {
    rlox::lox_tests!("variable", [
        #[ignore = "behavior: resolver doesn't detect parameter collision"] collide_with_parameter,
        #[ignore = "behavior: resolver doesn't detect duplicate locals"] duplicate_local,
        #[ignore = "behavior: resolver doesn't detect duplicate parameters"] duplicate_parameter,
        early_bound,
        in_middle_of_block,
        in_nested_block,
        #[ignore = "unimplemented: classes"] local_from_method,
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
        #[ignore = "behavior: resolver doesn't detect self-referencing initializer"] use_local_in_initializer,
        use_nil_as_var,
        #[ignore = "unimplemented: this"] use_this_as_var,
    ]);
}

mod while_loop {
    rlox::lox_tests!("while", [
        #[ignore = "unimplemented: classes"] class_in_body,
        closure_in_body,
        fun_in_body,
        return_closure,
        return_inside,
        syntax,
        var_in_body,
    ]);
}
