# frozen_string_literal: true

require 'set'
require 'pathname'
require 'forwardable'
require 'json'

require 'inkoc/version'
require 'inkoc/inspect'
require 'inkoc/source_file'
require 'inkoc/source_location'
require 'inkoc/token'
require 'inkoc/lexer'
require 'inkoc/parser'
require 'inkoc/type_scope'
require 'inkoc/equality'
require 'inkoc/module_paths_cache'
require 'inkoc/visitor_methods'
require 'inkoc/type_pass'
require 'inkoc/ast/type_operations'
require 'inkoc/ast/predicates'
require 'inkoc/ast/type_cast'
require 'inkoc/ast/body'
require 'inkoc/ast/send'
require 'inkoc/ast/block_type'
require 'inkoc/ast/lambda_type'
require 'inkoc/ast/constant'
require 'inkoc/ast/type_name'
require 'inkoc/ast/global'
require 'inkoc/ast/keyword_argument'
require 'inkoc/ast/string'
require 'inkoc/ast/template_string'
require 'inkoc/ast/integer'
require 'inkoc/ast/float'
require 'inkoc/ast/identifier'
require 'inkoc/ast/block'
require 'inkoc/ast/lambda'
require 'inkoc/ast/method'
require 'inkoc/ast/extern_method'
require 'inkoc/ast/define_argument'
require 'inkoc/ast/define_type_parameter'
require 'inkoc/ast/define_variable'
require 'inkoc/ast/define_attribute'
require 'inkoc/ast/attribute'
require 'inkoc/ast/object'
require 'inkoc/ast/trait_implementation'
require 'inkoc/ast/reopen_object'
require 'inkoc/ast/trait'
require 'inkoc/ast/return'
require 'inkoc/ast/yield'
require 'inkoc/ast/throw'
require 'inkoc/ast/reassign_variable'
require 'inkoc/ast/self'
require 'inkoc/ast/try'
require 'inkoc/ast/import'
require 'inkoc/ast/import_symbol'
require 'inkoc/ast/glob_import'
require 'inkoc/ast/compiler_option'
require 'inkoc/ast/method_requirement'
require 'inkoc/ast/new_instance'
require 'inkoc/ast/assign_attribute'
require 'inkoc/ast/match'
require 'inkoc/ast/match_type'
require 'inkoc/ast/match_expression'
require 'inkoc/ast/match_else'
require 'inkoc/config'
require 'inkoc/prototype_id'
require 'inkoc/constant_resolver'
require 'inkoc/state'
require 'inkoc/diagnostics'
require 'inkoc/diagnostic'
require 'inkoc/formatter/pretty'
require 'inkoc/formatter/json'
require 'inkoc/pass/path_to_source'
require 'inkoc/pass/source_to_ast'
require 'inkoc/pass/insert_implicit_imports'
require 'inkoc/pass/track_module'
require 'inkoc/pass/add_implicit_import_symbols'
require 'inkoc/pass/compile_imported_modules'
require 'inkoc/pass/desugar_object'
require 'inkoc/pass/desugar_method'
require 'inkoc/pass/define_import_types'
require 'inkoc/pass/define_this_module_type'
require 'inkoc/pass/define_type'
require 'inkoc/pass/define_type_signatures'
require 'inkoc/pass/validate_throw'
require 'inkoc/pass/implement_traits'
require 'inkoc/pass/generate_tir'
require 'inkoc/pass/define_module_type'
require 'inkoc/pass/code_generation'
require 'inkoc/pass/collect_imports'
require 'inkoc/pass/tail_call_elimination'
require 'inkoc/pass/setup_symbol_tables'
require 'inkoc/compiler'
require 'inkoc/tir/catch_entry'
require 'inkoc/tir/catch_table'
require 'inkoc/tir/code_object'
require 'inkoc/tir/basic_block'
require 'inkoc/tir/instruction/predicates'
require 'inkoc/tir/instruction/binary'
require 'inkoc/tir/instruction/ternary'
require 'inkoc/tir/instruction/quaternary'
require 'inkoc/tir/instruction/quinary'
require 'inkoc/tir/instruction/copy_blocks'
require 'inkoc/tir/instruction/unary'
require 'inkoc/tir/instruction/simple'
require 'inkoc/tir/instruction/nullary'
require 'inkoc/tir/instruction/panic'
require 'inkoc/tir/instruction/exit'
require 'inkoc/tir/instruction/array_set'
require 'inkoc/tir/instruction/get_global'
require 'inkoc/tir/instruction/get_local'
require 'inkoc/tir/instruction/get_parent_local'
require 'inkoc/tir/instruction/set_parent_local'
require 'inkoc/tir/instruction/goto_next_block_if_true'
require 'inkoc/tir/instruction/goto_next_block_if_false'
require 'inkoc/tir/instruction/goto_block_if_true'
require 'inkoc/tir/instruction/goto_block'
require 'inkoc/tir/instruction/skip_next_block'
require 'inkoc/tir/instruction/local_exists'
require 'inkoc/tir/instruction/return'
require 'inkoc/tir/instruction/throw'
require 'inkoc/tir/instruction/run_block'
require 'inkoc/tir/instruction/external_function_call'
require 'inkoc/tir/instruction/run_block_with_receiver'
require 'inkoc/tir/instruction/generator_allocate'
require 'inkoc/tir/instruction/tail_call'
require 'inkoc/tir/instruction/allocate_array'
require 'inkoc/tir/instruction/string_concat'
require 'inkoc/tir/instruction/set_attribute'
require 'inkoc/tir/instruction/set_block'
require 'inkoc/tir/instruction/set_global'
require 'inkoc/tir/instruction/set_local'
require 'inkoc/tir/instruction/allocate'
require 'inkoc/tir/instruction/allocate_permanent'
require 'inkoc/tir/instruction/try'
require 'inkoc/tir/instruction/set_literal'
require 'inkoc/tir/instruction/process_suspend_current'
require 'inkoc/tir/instruction/process_terminate_current'
require 'inkoc/tir/module'
require 'inkoc/tir/qualified_name'
require 'inkoc/tir/virtual_register'
require 'inkoc/tir/virtual_registers'
require 'inkoc/type_system/type'
require 'inkoc/type_system/type_with_prototype'
require 'inkoc/type_system/type_with_attributes'
require 'inkoc/type_system/generic_type'
require 'inkoc/type_system/generic_type_with_instances'
require 'inkoc/type_system/without_empty_type_parameters'
require 'inkoc/type_system/new_instance'
require 'inkoc/type_system/type_name'
require 'inkoc/type_system/type_parameter'
require 'inkoc/type_system/rigid_type_parameter'
require 'inkoc/type_system/type_parameter_table'
require 'inkoc/type_system/type_parameter_instances'
require 'inkoc/type_system/object'
require 'inkoc/type_system/trait'
require 'inkoc/type_system/block'
require 'inkoc/type_system/any'
require 'inkoc/type_system/self_type'
require 'inkoc/type_system/error'
require 'inkoc/type_system/never'
require 'inkoc/type_system/database'
require 'inkoc/symbol'
require 'inkoc/symbol_table'
require 'inkoc/codegen/compiled_code'
require 'inkoc/codegen/instruction'
require 'inkoc/codegen/literals'
require 'inkoc/codegen/serializer'
require 'inkoc/codegen/catch_entry'
require 'inkoc/codegen/module'
