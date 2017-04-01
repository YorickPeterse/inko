//! VM instruction handlers for executing bytecode files and code objects.
use vm::action::Action;
use vm::instruction::Instruction;
use vm::instructions::result::InstructionResult;
use vm::machine::Machine;

use block::Block;
use binding::Binding;
use compiled_code::RcCompiledCode;
use execution_context::ExecutionContext;
use object_value;
use process::RcProcess;

/// Executes a Block object.
///
/// This instruction takes the following arguments:
///
/// 1. The register to store the return value in.
/// 2. The register containing the Block object to run.
///
/// Any extra arguments passed are passed as arguments to the CompiledCode
/// object.
pub fn run_block(_: &Machine,
                 process: &RcProcess,
                 _: &RcCompiledCode,
                 instruction: &Instruction)
                 -> InstructionResult {
    process.advance_line(instruction.line);

    let register = instruction.arg(0)?;
    let block_ptr = process.get_register(instruction.arg(1)?)?;
    let block_val = block_ptr.block_value()?;

    let arg_offset = 2;
    let arg_count = instruction.arguments.len() - arg_offset;
    let tot_args = block_val.arguments();
    let req_args = block_val.required_arguments();

    if arg_count > tot_args {
        return Err(format!("{} accepts up to {} arguments, but {} arguments \
                            were given",
                           block_val.name(),
                           tot_args,
                           arg_count));
    }

    if arg_count < req_args {
        return Err(format!("{} requires {} arguments, but {} arguments were \
                            given",
                           block_val.name(),
                           req_args,
                           arg_count));
    }

    let context = ExecutionContext::with_binding(block_val.binding.clone(),
                                                 block_val.code.clone(),
                                                 Some(register));

    {
        // Add the arguments to the binding. Since arguments are the first
        // locals we can just push them in-order.
        let mut locals = context.binding.locals_mut();

        for index in arg_offset..(arg_offset + arg_count) {
            let register = instruction.arg(index)?;

            locals.push(process.get_register(register)?);
        }
    }

    process.push_context(context);

    Ok(Action::EnterContext)
}

/// Executes a Block object with a rest argument.
///
/// This instruction takes the following arguments:
///
/// 1. The register to store the return value in.
/// 2. The register containing the Block object to run.
///
/// Any extra arguments passed are passed as arguments to the CompiledCode
/// object. If excessive arguments are given they are packed into the block's
/// rest argument.
pub fn run_block_with_rest(_: &Machine,
                           _: &RcProcess,
                           _: &RcCompiledCode,
                           _: &Instruction)
                           -> InstructionResult {
    // TODO: implement
    //let register = instruction.arg(0)?;
    //let block_ptr = process.get_register(instruction.arg(1)?)?;
    //let block_val = block_ptr.block_value()?;
    //let has_rest = block_val.has_rest_argument();

    // Unpack the last argument if it's a rest argument
    //if rest_arg {
    //if let Some(last_arg) = arguments.pop() {
    //for value in last_arg.array_value()? {
    //arguments.push(value.clone());
    //}
    //}
    //}

    // If the code object defines a rest argument we'll pack any excessive
    // arguments into a single array.
    //if block_val.has_rest_argument() && arguments.len() > tot_args {
    //let rest_count = arguments.len() - tot_args;
    //let mut rest = Vec::new();

    //for obj in arguments[arguments.len() - rest_count..].iter() {
    //rest.push(obj.clone());
    //}

    //arguments.truncate(tot_args);

    //let rest_array = process.allocate(object_value::array(rest),
    //machine.state.array_prototype.clone());

    //arguments.push(rest_array);
    //} else if block_val.has_rest_argument() && arguments.len() == 0 {
    //let rest_array = process.allocate(object_value::array(Vec::new()),
    //machine.state.array_prototype.clone());

    //arguments.push(rest_array);
    //}

    Ok(Action::EnterContext)
}

/// Parses a bytecode file and stores the resulting Block in the register.
///
/// This instruction requires 2 arguments:
///
/// 1. The register to store the resulting Block in.
/// 2. The register containing the file path to open, as a string.
///
/// This instruction will panic if the file does not exist or when the bytecode
/// is invalid.
pub fn parse_file(machine: &Machine,
                  process: &RcProcess,
                  _: &RcCompiledCode,
                  instruction: &Instruction)
                  -> InstructionResult {
    let register = instruction.arg(0)?;
    let path_ptr = process.get_register(instruction.arg(1)?)?;
    let path_str = path_ptr.string_value()?;

    let code = write_lock!(machine.file_registry).get_or_set(path_str)
        .map_err(|err| err.message())?;

    let block = Block::new(code.clone(), Binding::new());

    let block_ptr = process.allocate(object_value::block(block),
                                    machine.state.block_prototype);

    process.set_register(register, block_ptr);

    Ok(Action::None)
}

/// Sets the target register to true if the given file path has been parsed.
///
/// This instruction requires two arguments:
///
/// 1. The register to store the resulting boolean in.
/// 2. The register containing the file path to check.
///
/// The result of this instruction is true or false.
pub fn file_parsed(machine: &Machine,
                   process: &RcProcess,
                   _: &RcCompiledCode,
                   instruction: &Instruction)
                   -> InstructionResult {
    let register = instruction.arg(0)?;
    let path_ptr = process.get_register(instruction.arg(1)?)?;
    let path_str = path_ptr.string_value()?;

    let ptr = if read_lock!(machine.file_registry).contains_path(path_str) {
        machine.state.true_object
    } else {
        machine.state.false_object
    };

    process.set_register(register, ptr);

    Ok(Action::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binding::Binding;
    use object_value;
    use vm::instructions::test::*;
    use vm::instruction::InstructionType;

    mod run_block {
        use super::*;

        #[test]
        fn test_without_arguments() {
            let (machine, code, process) = setup();

            let block = Block::new(code.clone(), Binding::new());

            let block_ptr =
                process.allocate_without_prototype(object_value::block(block));

            process.set_register(0, block_ptr);
            process.set_register(1, machine.state.false_object);

            let instruction = new_instruction(InstructionType::RunBlock,
                                              vec![2, 0, 1]);

            let result = run_block(&machine, &process, &code, &instruction);

            assert!(result.is_ok());

            assert!(process.context().parent.is_some());
            assert!(process.binding().locals().is_empty());
        }

        #[test]
        fn test_with_too_many_arguments() {
            let (machine, code, process) = setup();

            let block = Block::new(code.clone(), Binding::new());

            let block_ptr =
                process.allocate_without_prototype(object_value::block(block));

            process.set_register(0, block_ptr);
            process.set_register(1, machine.state.true_object);
            process.set_register(2, machine.state.false_object);

            let instruction = new_instruction(InstructionType::RunBlock,
                                              vec![3, 0, 1, 2]);

            let result = run_block(&machine, &process, &code, &instruction);

            assert!(result.is_err());
        }

        #[test]
        fn test_with_not_enough_arguments() {
            let (machine, code, process) = setup();

            arc_mut(&code).arguments = 2;
            arc_mut(&code).required_arguments = 2;

            let block = Block::new(code.clone(), Binding::new());

            let block_ptr =
                process.allocate_without_prototype(object_value::block(block));

            process.set_register(0, block_ptr);
            process.set_register(1, machine.state.true_object);
            process.set_register(2, machine.state.false_object);

            let instruction = new_instruction(InstructionType::RunBlock,
                                              vec![3, 0, 2, 1]);

            let result = run_block(&machine, &process, &code, &instruction);

            assert!(result.is_err());
        }

        #[test]
        fn test_with_enough_arguments() {
            let (machine, code, process) = setup();

            arc_mut(&code).arguments = 2;

            let block = Block::new(code.clone(), Binding::new());

            let block_ptr =
                process.allocate_without_prototype(object_value::block(block));

            process.set_register(0, block_ptr);
            process.set_register(1, machine.state.true_object);
            process.set_register(2, machine.state.false_object);
            process.set_register(3, machine.state.false_object);

            let instruction = new_instruction(InstructionType::RunBlock,
                                              vec![4, 0, 3, 1, 2]);

            let result = run_block(&machine, &process, &code, &instruction);

            assert!(result.is_ok());

            assert_eq!(process.binding().locals().len(), 2);

            assert!(process.binding().get_local(0).unwrap() ==
                    machine.state.true_object);

            assert!(process.binding().get_local(1).unwrap() ==
                    machine.state.false_object);
        }

        #[test]
        fn test_with_rest_argument() {
            let (machine, code, process) = setup();

            arc_mut(&code).arguments = 2;
            arc_mut(&code).rest_argument = true;

            let block = Block::new(code.clone(), Binding::new());

            let block_ptr =
                process.allocate_without_prototype(object_value::block(block));

            process.set_register(0, block_ptr);
            process.set_register(1, machine.state.true_object);

            let args =
                process.allocate_without_prototype(object_value::array(vec![machine.state.true_object,
                                                                       machine.state.false_object]));

            process.set_register(2, args);

            let instruction = new_instruction(InstructionType::RunBlock,
                                              vec![5, 0, 1, 2]);

            let result = run_block(&machine, &process, &code, &instruction);

            assert!(result.is_ok());

            assert_eq!(process.binding().locals().len(), 2);

            assert!(process.binding().get_local(0).unwrap() ==
                    machine.state.true_object);

            assert!(process.binding().get_local(1).unwrap() ==
                    machine.state.false_object);
        }
    }
}
