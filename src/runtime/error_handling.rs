use miette::Diagnostic;
use thiserror::Error;

use crate::base::Operation;

/// Errors that can occur when a runtime is constructed from a `RuntimeBuilder`.
#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum RuntimeBuildError { // TODO Make error messages consistent by starting them with a captial letter and by better explaining the reason, make them consistent with the runtime errors
    #[error("runtime arguments missing")]
    #[diagnostic(code("runtime_build_error::runtime_args_missing"))]
    RuntimeArgsMissing,
    #[error("instructions missing")]
    #[diagnostic(code("runtime_build_error::instructions_missing"))]
    InstructionsMissing,
    /// Indicates that a label is used in an instruction that does not exist in the control flow.
    /// This would lead to a runtime error.
    #[error("label '{0}' undefined")]
    #[diagnostic(code("runtime_build_error::label_undefined"), help("Make sure that you include the label somewhere before an instruction.\nExample: '{0}: a0 := 5'"))]
    LabelUndefined(String),
    #[error("memory cell '{0}' is missing")]
    #[diagnostic(code("runtime_build_error::memory_cell_missing"), help("Make sure to include the memory cell '{0}' in the available memory cells.\nExample: alpha_tui -i FILE -m {0}"))]
    MemoryCellMissing(String),
    #[error("accumulator with id '{0}' is missing")]
    #[diagnostic(
        code("runtime_build_error::accumulator_missing"),
        help("Make sure to have the number of available accumulators set to at least {0}+1")
    )]
    AccumulatorMissing(String),
    
    #[error("Gamma accumulator is used in the program but is disabled")]
    #[diagnostic(
        code("runtime_build_error::gamma_disabled"),
        help("You can't use the gamma accumulator when it is disabled, to enable it you can either enable automatic memory detection\nby removing the \"--disable-memory-detection\" flag or you can explicitly enable it by using the \"--enable-gamma-accumulator\" flag.") // TODO Add flag to disable gamma accumulator and update this message
    )]
    GammaDisabled,
}

#[derive(Debug)]
pub enum AddLabelError {
    InstructionsNotSet,
    IndexOutOfBounds,
}

#[derive(Debug, Error, Diagnostic, Clone, PartialEq)]
#[error("runtime error in line {line_number}")]
pub struct RuntimeError {
    #[diagnostic_source]
    pub reason: RuntimeErrorType,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum RuntimeErrorType {
    #[error("Attempt to use value of accumulator with index '{0}' while value is not initialized")]
    #[diagnostic(
        code("runtime_error::accumulator_uninitialized"),
        help("Try assigning a value before accessing it.\nExample: a{0} := 5")
    )]
    AccumulatorUninitialized(usize),

    #[error("Attempt to use accumulator with index '{0}' while it does not exist")]
    #[diagnostic(
        code("runtime_error::accumulator_does_not_exist"),
        help("Make sure to tell the program to use enough accumulators by using the '-a' flag")
    )]
    AccumulatorDoesNotExist(usize),

    #[error("Attempt to use value of accumulator gamma while value is not initialized")]
    #[diagnostic(
        code("runtime_error::gamma_uninitialized"),
        help("Try assigning a value before accessing it.\nExample: y := 5")
    )]
    GammaUninitialized,

    #[error("Attempt to use accumulator gamma while it does not exist")]
    #[diagnostic(
        code("runtime_error::gamma_does_not_exist"),
        help("Make sure to tell the program to use the gamma accumulator by using the TODO flag") // TODO implement flag that enables gamma accumulator
    )]
    GammaDoesNotExist,

    #[error("Attempt to use value of memory cell named '{0}' while value is not initialized")]
    #[diagnostic(
        code("runtime_error::memory_cell_uninitialized"),
        help("Try assigning a value before accessing it.\nExample p({0}) := 5")
    )]
    MemoryCellUninitialized(String),

    #[error("Attempt to use value of memory cell named '{0}' that does not exist")]
    #[diagnostic(
        code("runtime_error::memory_cell_uninitialized"),
        help("Make sure to tell the program to use this memory cell by using the '-m' flag")
    )]
    MemoryCellDoesNotExist(String),

    #[error("Attempt to use value of index memory cell with index '{0}' while value is not initialized")]
    #[diagnostic(
        code("runtime_error::index_memory_cell_uninitialized"),
        help("Try assigning a value before accessing it.\nExample p({0}) := 5")
    )]
    IndexMemoryCellUninitialized(usize),

    #[error("Attempt to push value of a0 onto stack while a0 is not initialized")]
    #[diagnostic(
        code("runtime_error::push_fail"),
        help("Try assigning a value before accessing it.\nExample: a0 := 5")
    )]
    PushFail,

    #[error("Attempt to pop value from stack while stack is empty")]
    #[diagnostic(
        code("runtime_error::stack_empty"),
        help("Make sure to only use pop when you know that the stack contains at least one value")
    )]
    PopFail,

    #[error("Attempt to perform operation '{0}' on stack while stack does not contain two values")]
    #[diagnostic(
        code("runtime_error::stack_op_fail"),
        help("Make sure to only use a stack operation (stack{0}) when you know that the stack contains at least two values")
    )]
    StackOpFail(Operation),

    #[error("Stack Overflow")]
    #[diagnostic(
        code("runtime_error::stack_overflow_error"),
        help("This error is usually caused by an infinite recursion. Make sure that all of your recursive functions return properly.")
    )]
    StackOverflowError,

    #[error("Attempt to jump to label '{0}' that does not exist")]
    #[diagnostic(
        code("runtime_error::label_missing"),
        help("Try to create the label.\nExample: '{0}: a0 := 5' or '{0}:'")
    )]
    LabelMissing(String),

    //#[error("Attempt to divide by zero")]
    //#[diagnostic(
    //    code("runtime_error::attempt_to_divide_by_zero"),
    //    help("Division by zero is undefined in mathematics")
    //)]
    //AttemptToDivideByZero(),
    #[error("Illegal calculation")]
    #[diagnostic(code(runtime_error::illegal_calculation))]
    IllegalCalculation {
        #[diagnostic_source]
        cause: CalcError,
    },

    #[error("Attempt to access index memory cell with negative index, '{0}'")]
    #[diagnostic(
        code("runtime_error::index_memory_cell_negative_index"),
        help("Make sure that the value with which you try to access the index memory cell is positive")
    )]
    IndexMemoryCellNegativeIndex(i32),
}

#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum CalcError {
    #[error("Attempt to divide by zero")]
    #[diagnostic(
        code("calc_error::attempt_to_divide_by_zero"),
        help("Division by zero is undefined in mathematics")
    )]
    AttemptToDivideByZero(),

    #[error("Attempt to {0} with overflow")]
    #[diagnostic(
        code("calc_error::attempt_to_overflow"),
        help("{1} would have resulted in an overflow leading to a wrong value.\nMake sure the integer never leaves the following range: [{},{}]", i32::MIN, i32::MAX)
    )]
    AttemptToOverflow(String, String),
}

#[cfg(test)]
mod tests {
    use crate::{
        base::Operation,
        instructions::{Instruction, TargetType, Value},
        runtime::{
            builder::RuntimeBuilder,
            error_handling::{CalcError, RuntimeBuildError, RuntimeErrorType},
            ControlFlow, RuntimeArgs,
        },
    };

    #[test]
    fn test_rbe_runtime_args_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_instructions(vec![Instruction::Push]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::RuntimeArgsMissing));
    }

    #[test]
    fn test_rbe_instructions_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_runtime_args(RuntimeArgs::new_debug(&[""]));
        assert_eq!(rt.build(), Err(RuntimeBuildError::InstructionsMissing));
    }

    #[test]
    fn test_rbe_label_undefined_error() {
        let mut rt = RuntimeBuilder::new_debug(&[]);
        rt.set_instructions(vec![Instruction::Goto("loop".to_string())]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::LabelUndefined("loop".to_string()))
        );
    }

    #[test]
    fn test_rbe_memory_cell_missing() {
        let mut rt = RuntimeBuilder::new_debug(&[]);
        rt.set_instructions(vec![Instruction::Assign(
            TargetType::MemoryCell("h1".to_string()),
            Value::Constant(10),
        )]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::MemoryCellMissing("h1".to_string()))
        );
    }

    #[test]
    fn test_rbe_accumulator_missing() {
        let mut rt = RuntimeBuilder::new_debug(&[]);
        rt.set_runtime_args(RuntimeArgs::new_empty());
        rt.set_instructions(vec![Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(10),
        )]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::AccumulatorMissing("0".to_string()))
        );
    }

    #[test]
    fn test_re_accumulator_uninitialized() {
        let mut ra = RuntimeArgs::new(1, vec!["h1".to_string()], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Assign(
                TargetType::MemoryCell("h1".to_string()),
                Value::Accumulator(0)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::AccumulatorUninitialized(0))
        );
    }

    #[test]
    fn test_re_accumulator_does_not_exist() {
        let mut ra = RuntimeArgs::new(0, vec!["h1".to_string()], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Assign(
                TargetType::MemoryCell("h1".to_string()),
                Value::Accumulator(0)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::AccumulatorDoesNotExist(0))
        );
    }

    #[test]
    fn test_re_memory_cell_uninitialized() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Assign(
                TargetType::Accumulator(0),
                Value::MemoryCell("a".to_string())
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::MemoryCellUninitialized("a".to_string()))
        );
    }

    #[test]
    fn test_re_memory_cell_does_not_exist() {
        let mut ra = RuntimeArgs::new(1, vec![], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Assign(
                TargetType::Accumulator(0),
                Value::MemoryCell("a".to_string())
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::MemoryCellDoesNotExist("a".to_string()))
        );
    }

    #[test]
    fn test_re_push_fail() {
        let mut ra = RuntimeArgs::new(1, vec![], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Push.run(&mut ra, &mut cf),
            Err(RuntimeErrorType::PushFail)
        );
    }

    #[test]
    fn test_re_pop_fail() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Pop.run(&mut ra, &mut cf),
            Err(RuntimeErrorType::PopFail)
        );
    }

    #[test]
    fn test_re_stack_overflow() {
        let ra = RuntimeArgs::new(1, vec!["a".to_string()], None, true);
        let mut rb = RuntimeBuilder::new();
        rb.set_runtime_args(ra);
        let instructions = vec!["loop: call loop"];
        rb.build_instructions(&instructions, "test").unwrap();
        let mut rt = rb.build().unwrap();
        assert_eq!(
            rt.run().unwrap_err().reason,
            RuntimeErrorType::StackOverflowError
        );
    }

    #[test]
    fn test_re_label_missing() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()], None, true);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Goto("loop".to_string()).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::LabelMissing("loop".to_string()))
        );
    }

    #[test]
    fn test_ce_me_attempt_to_divide_by_zero() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(0);
        ra.accumulators.get_mut(&1).unwrap().data = Some(0);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Div,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToDivideByZero()
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_add() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(i32::MAX);
        ra.accumulators.get_mut(&1).unwrap().data = Some(1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Add,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToOverflow("add".to_string(), "Addition".to_string())
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_sub() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(i32::MIN);
        ra.accumulators.get_mut(&1).unwrap().data = Some(1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Sub,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToOverflow(
                    "subtract".to_string(),
                    "Subtraction".to_string()
                )
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_div() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(i32::MIN);
        ra.accumulators.get_mut(&1).unwrap().data = Some(-1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Div,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToOverflow("divide".to_string(), "Division".to_string())
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_mul() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(i32::MAX);
        ra.accumulators.get_mut(&1).unwrap().data = Some(2);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Mul,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToOverflow(
                    "multiply".to_string(),
                    "Multiplication".to_string()
                )
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_mod() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(i32::MIN);
        ra.accumulators.get_mut(&1).unwrap().data = Some(-1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Mod,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToOverflow("mod".to_string(), "Modulo".to_string())
            })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_divide_by_zero_mod() {
        let mut ra = RuntimeArgs::new(2, vec![], None, true);
        ra.accumulators.get_mut(&0).unwrap().data = Some(10);
        ra.accumulators.get_mut(&1).unwrap().data = Some(0);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(0),
                Operation::Mod,
                Value::Accumulator(1)
            )
            .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation {
                cause: CalcError::AttemptToDivideByZero()
            })
        );
    }
}
