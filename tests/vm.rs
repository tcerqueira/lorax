const BACKEND: rlox::test_utils::Backend = rlox::test_utils::Backend::Vm;

// Tests are marked `#[ignore = "VM not yet implemented"]` when they exercise a
// feature the VM doesn't support yet (locals, control flow, functions, classes,
// closures, etc.) — even if they currently happen to pass (e.g. because the
// test runner only checks that *some* error occurred, not that the right one
// did). Un-ignore as each feature lands.

#[test]
#[ignore = "VM not yet implemented"]
fn examples() {
    rlox::test_utils::run_examples(BACKEND, "examples");
}

mod assignment {
    rlox::lox_tests!("assignment", [
        associativity,
        global,
        grouping,
        infix_operator,
        #[ignore = "VM not yet implemented"] local,
        prefix_operator,
        syntax,
        #[ignore = "VM not yet implemented"] to_this,
        undefined,
    ]);
}

mod block {
    rlox::lox_tests!("block", [
        #[ignore = "VM not yet implemented"] empty,
        #[ignore = "VM not yet implemented"] scope,
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
        #[ignore = "VM not yet implemented"] bool,
        #[ignore = "VM not yet implemented"] nil,
        #[ignore = "VM not yet implemented"] num,
        #[ignore = "VM not yet implemented"] object,
        #[ignore = "VM not yet implemented"] string,
    ]);
}

mod class {
    rlox::lox_tests!("class", [
        #[ignore = "VM not yet implemented"] empty,
        #[ignore = "VM not yet implemented"] inherit_self,
        #[ignore = "VM not yet implemented"] inherited_method,
        #[ignore = "VM not yet implemented"] local_inherit_other,
        #[ignore = "VM not yet implemented"] local_inherit_self,
        #[ignore = "VM not yet implemented"] local_reference_self,
        #[ignore = "VM not yet implemented"] reference_self,
    ]);
}

mod closure {
    rlox::lox_tests!("closure", [
        #[ignore = "VM not yet implemented"] assign_to_closure,
        #[ignore = "VM not yet implemented"] assign_to_shadowed_later,
        #[ignore = "VM not yet implemented"] close_over_function_parameter,
        #[ignore = "VM not yet implemented"] close_over_later_variable,
        #[ignore = "VM not yet implemented"] close_over_method_parameter,
        #[ignore = "VM not yet implemented"] closed_closure_in_function,
        #[ignore = "VM not yet implemented"] nested_closure,
        #[ignore = "VM not yet implemented"] open_closure_in_function,
        #[ignore = "VM not yet implemented"] reference_closure_multiple_times,
        #[ignore = "VM not yet implemented"] reuse_closure_slot,
        #[ignore = "VM not yet implemented"] shadow_closure_with_local,
        #[ignore = "VM not yet implemented"] unused_closure,
        #[ignore = "VM not yet implemented"] unused_later_closure,
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
        #[ignore = "VM not yet implemented"] arguments,
        #[ignore = "VM not yet implemented"] call_init_early_return,
        #[ignore = "VM not yet implemented"] call_init_explicitly,
        #[ignore = "VM not yet implemented"] default,
        #[ignore = "VM not yet implemented"] default_arguments,
        #[ignore = "VM not yet implemented"] early_return,
        #[ignore = "VM not yet implemented"] extra_arguments,
        #[ignore = "VM not yet implemented"] init_not_method,
        #[ignore = "VM not yet implemented"] missing_arguments,
        #[ignore = "VM not yet implemented"] return_in_nested_function,
        #[ignore = "VM not yet implemented"] return_value,
    ]);
}

mod expressions {
    rlox::lox_tests!("expressions", [
        evaluate,
        #[ignore = "VM not yet implemented"] parse,
    ]);
}

mod root {
    rlox::lox_tests!([empty_file, precedence, unexpected_character]);
}

mod field {
    rlox::lox_tests!("field", [
        #[ignore = "VM not yet implemented"] call_function_field,
        #[ignore = "VM not yet implemented"] call_nonfunction_field,
        #[ignore = "VM not yet implemented"] get_and_set_method,
        #[ignore = "VM not yet implemented"] get_on_bool,
        #[ignore = "VM not yet implemented"] get_on_class,
        #[ignore = "VM not yet implemented"] get_on_function,
        #[ignore = "VM not yet implemented"] get_on_nil,
        #[ignore = "VM not yet implemented"] get_on_num,
        #[ignore = "VM not yet implemented"] get_on_string,
        #[ignore = "VM not yet implemented"] many,
        #[ignore = "VM not yet implemented"] method,
        #[ignore = "VM not yet implemented"] method_binds_this,
        #[ignore = "VM not yet implemented"] on_instance,
        #[ignore = "VM not yet implemented"] set_evaluation_order,
        #[ignore = "VM not yet implemented"] set_on_bool,
        #[ignore = "VM not yet implemented"] set_on_class,
        #[ignore = "VM not yet implemented"] set_on_function,
        #[ignore = "VM not yet implemented"] set_on_nil,
        #[ignore = "VM not yet implemented"] set_on_num,
        #[ignore = "VM not yet implemented"] set_on_string,
        #[ignore = "VM not yet implemented"] undefined,
    ]);
}

mod for_loop {
    rlox::lox_tests!("for", [
        #[ignore = "VM not yet implemented"] class_in_body,
        #[ignore = "VM not yet implemented"] closure_in_body,
        #[ignore = "VM not yet implemented"] fun_in_body,
        #[ignore = "VM not yet implemented"] return_closure,
        #[ignore = "VM not yet implemented"] return_inside,
        #[ignore = "VM not yet implemented"] scope,
        #[ignore = "VM not yet implemented"] statement_condition,
        #[ignore = "VM not yet implemented"] statement_increment,
        #[ignore = "VM not yet implemented"] statement_initializer,
        #[ignore = "VM not yet implemented"] syntax,
        #[ignore = "VM not yet implemented"] var_in_body,
    ]);
}

mod function {
    rlox::lox_tests!("function", [
        #[ignore = "VM not yet implemented"] body_must_be_block,
        #[ignore = "VM not yet implemented"] empty_body,
        #[ignore = "VM not yet implemented"] extra_arguments,
        #[ignore = "VM not yet implemented"] local_mutual_recursion,
        #[ignore = "VM not yet implemented"] local_recursion,
        #[ignore = "VM not yet implemented"] missing_arguments,
        #[ignore = "VM not yet implemented"] missing_comma_in_parameters,
        #[ignore = "VM not yet implemented"] mutual_recursion,
        #[ignore = "VM not yet implemented"] nested_call_with_arguments,
        #[ignore = "VM not yet implemented"] parameters,
        #[ignore = "VM not yet implemented"] print,
        #[ignore = "VM not yet implemented"] recursion,
        #[ignore = "VM not yet implemented"] too_many_arguments,
        #[ignore = "VM not yet implemented"] too_many_parameters,
    ]);
}

mod if_stmt {
    rlox::lox_tests!("if", [
        #[ignore = "VM not yet implemented"] class_in_else,
        #[ignore = "VM not yet implemented"] class_in_then,
        #[ignore = "VM not yet implemented"] dangling_else,
        #[ignore = "VM not yet implemented"] r#else,
        #[ignore = "VM not yet implemented"] fun_in_else,
        #[ignore = "VM not yet implemented"] fun_in_then,
        #[ignore = "VM not yet implemented"] r#if,
        #[ignore = "VM not yet implemented"] truth,
        #[ignore = "VM not yet implemented"] var_in_else,
        #[ignore = "VM not yet implemented"] var_in_then,
    ]);
}

mod inheritance {
    rlox::lox_tests!("inheritance", [
        #[ignore = "VM not yet implemented"] constructor,
        #[ignore = "VM not yet implemented"] inherit_from_function,
        #[ignore = "VM not yet implemented"] inherit_from_nil,
        #[ignore = "VM not yet implemented"] inherit_from_number,
        #[ignore = "VM not yet implemented"] inherit_methods,
        #[ignore = "VM not yet implemented"] parenthesized_superclass,
        #[ignore = "VM not yet implemented"] set_fields_from_base_class,
    ]);
}

mod limit {
    rlox::lox_tests!("limit", [
        #[ignore = "VM not yet implemented"] loop_too_large,
        #[ignore = "VM not yet implemented"] no_reuse_constants,
        #[ignore = "VM not yet implemented"] stack_overflow,
        #[ignore = "VM not yet implemented"] too_many_constants,
        #[ignore = "VM not yet implemented"] too_many_locals,
        #[ignore = "VM not yet implemented"] too_many_upvalues,
    ]);
}

mod logical_operator {
    rlox::lox_tests!("logical_operator", [
        #[ignore = "VM not yet implemented"] and,
        #[ignore = "VM not yet implemented"] and_truth,
        #[ignore = "VM not yet implemented"] or,
        #[ignore = "VM not yet implemented"] or_truth,
    ]);
}

mod method {
    rlox::lox_tests!("method", [
        #[ignore = "VM not yet implemented"] arity,
        #[ignore = "VM not yet implemented"] empty_block,
        #[ignore = "VM not yet implemented"] extra_arguments,
        #[ignore = "VM not yet implemented"] missing_arguments,
        #[ignore = "VM not yet implemented"] not_found,
        #[ignore = "VM not yet implemented"] print_bound_method,
        #[ignore = "VM not yet implemented"] refer_to_name,
        #[ignore = "VM not yet implemented"] too_many_arguments,
        #[ignore = "VM not yet implemented"] too_many_parameters,
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
        #[ignore = "VM not yet implemented"] equals_class,
        #[ignore = "VM not yet implemented"] equals_method,
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
        #[ignore = "VM not yet implemented"] not,
        #[ignore = "VM not yet implemented"] not_class,
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
    rlox::lox_tests!("return", [
        #[ignore = "VM not yet implemented"] after_else,
        #[ignore = "VM not yet implemented"] after_if,
        #[ignore = "VM not yet implemented"] after_while,
        #[ignore = "VM not yet implemented"] at_top_level,
        #[ignore = "VM not yet implemented"] in_function,
        #[ignore = "VM not yet implemented"] in_method,
        #[ignore = "VM not yet implemented"] return_nil_if_no_value,
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
        #[ignore = "VM not yet implemented"] bound_method,
        #[ignore = "VM not yet implemented"] call_other_method,
        #[ignore = "VM not yet implemented"] call_same_method,
        #[ignore = "VM not yet implemented"] closure,
        #[ignore = "VM not yet implemented"] constructor,
        #[ignore = "VM not yet implemented"] extra_arguments,
        #[ignore = "VM not yet implemented"] indirectly_inherited,
        #[ignore = "VM not yet implemented"] missing_arguments,
        #[ignore = "VM not yet implemented"] no_superclass_bind,
        #[ignore = "VM not yet implemented"] no_superclass_call,
        #[ignore = "VM not yet implemented"] no_superclass_method,
        #[ignore = "VM not yet implemented"] parenthesized,
        #[ignore = "VM not yet implemented"] reassign_superclass,
        #[ignore = "VM not yet implemented"] super_at_top_level,
        #[ignore = "VM not yet implemented"] super_in_closure_in_inherited_method,
        #[ignore = "VM not yet implemented"] super_in_inherited_method,
        #[ignore = "VM not yet implemented"] super_in_top_level_function,
        #[ignore = "VM not yet implemented"] super_without_dot,
        #[ignore = "VM not yet implemented"] super_without_name,
        #[ignore = "VM not yet implemented"] this_in_superclass_method,
    ]);
}

mod this_ {
    rlox::lox_tests!("this", [
        #[ignore = "VM not yet implemented"] closure,
        #[ignore = "VM not yet implemented"] nested_class,
        #[ignore = "VM not yet implemented"] nested_closure,
        #[ignore = "VM not yet implemented"] this_at_top_level,
        #[ignore = "VM not yet implemented"] this_in_method,
        #[ignore = "VM not yet implemented"] this_in_top_level_function,
    ]);
}

mod variable {
    rlox::lox_tests!("variable", [
        #[ignore = "VM not yet implemented"] collide_with_parameter,
        #[ignore = "VM not yet implemented"] duplicate_local,
        #[ignore = "VM not yet implemented"] duplicate_parameter,
        #[ignore = "VM not yet implemented"] early_bound,
        #[ignore = "VM not yet implemented"] in_middle_of_block,
        #[ignore = "VM not yet implemented"] in_nested_block,
        #[ignore = "VM not yet implemented"] local_from_method,
        redeclare_global,
        redefine_global,
        #[ignore = "VM not yet implemented"] scope_reuse_in_different_blocks,
        #[ignore = "VM not yet implemented"] shadow_and_local,
        #[ignore = "VM not yet implemented"] shadow_global,
        #[ignore = "VM not yet implemented"] shadow_local,
        undefined_global,
        #[ignore = "VM not yet implemented"] undefined_local,
        uninitialized,
        #[ignore = "VM not yet implemented"] unreached_undefined,
        use_false_as_var,
        use_global_in_initializer,
        #[ignore = "VM not yet implemented"] use_local_in_initializer,
        use_nil_as_var,
        use_this_as_var,
    ]);
}

mod while_loop {
    rlox::lox_tests!("while", [
        #[ignore = "VM not yet implemented"] class_in_body,
        #[ignore = "VM not yet implemented"] closure_in_body,
        #[ignore = "VM not yet implemented"] fun_in_body,
        #[ignore = "VM not yet implemented"] return_closure,
        #[ignore = "VM not yet implemented"] return_inside,
        #[ignore = "VM not yet implemented"] syntax,
        #[ignore = "VM not yet implemented"] var_in_body,
    ]);
}
