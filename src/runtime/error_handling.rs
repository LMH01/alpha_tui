use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur when a runtime is constructed from a RuntimeBuilder.
#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum RuntimeBuildError {
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
}

#[derive(Debug)]
pub enum AddLabelError {
    InstructionsNotSet,
    IndexOutOfBounds,
}

#[derive(Debug, Error, Diagnostic, Clone)]
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

    #[error("Attempt to jump to label '{0}' that does not exist")]
    #[diagnostic(
        code("runtime_error::label_missing"),
        help("Try to create the label before an instruction.\nExample: {0}: a0 := 5")
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
        cause: CalcError
    }
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
        instructions::Instruction,
        runtime::{
            builder::RuntimeBuilder,
            error_handling::{RuntimeBuildError, RuntimeErrorType, CalcError},
            ControlFlow, RuntimeArgs,
        }, base::Operation,
    };

    #[test]
    fn test_rbe_runtime_args_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_instructions(vec![Instruction::Push()]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::RuntimeArgsMissing));
    }

    #[test]
    fn test_rbe_instructions_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_runtime_args(RuntimeArgs::new_debug(&vec![""]));
        assert_eq!(rt.build(), Err(RuntimeBuildError::InstructionsMissing));
    }

    #[test]
    fn test_rbe_label_undefined_error() {
        let mut rt = RuntimeBuilder::new_debug(&vec![]);
        rt.set_instructions(vec![Instruction::Goto("loop".to_string())]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::LabelUndefined("loop".to_string()))
        );
    }

    #[test]
    fn test_rbe_memory_cell_missing() {
        let mut rt = RuntimeBuilder::new_debug(&vec![]);
        rt.set_instructions(vec![Instruction::AssignMemoryCellValue(
            "h1".to_string(),
            10,
        )]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::MemoryCellMissing("h1".to_string()))
        );
    }

    #[test]
    fn test_rbe_accumulator_missing() {
        let mut rt = RuntimeBuilder::new();
        rt.set_runtime_args(RuntimeArgs::new_empty());
        rt.set_instructions(vec![Instruction::AssignAccumulatorValue(0, 10)]);
        assert_eq!(
            rt.build(),
            Err(RuntimeBuildError::AccumulatorMissing("0".to_string()))
        );
    }

    #[test]
    fn test_re_accumulator_uninitialized() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
                .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::AccumulatorUninitialized(0))
        );
    }

    #[test]
    fn test_re_accumulator_does_not_exist() {
        let mut ra = RuntimeArgs::new(0, vec!["a".to_string()]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
                .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::AccumulatorDoesNotExist(0))
        );
    }

    #[test]
    fn test_re_memory_cell_uninitialized() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
                .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::MemoryCellUninitialized("a".to_string()))
        );
    }

    #[test]
    fn test_re_memory_cell_does_not_exist() {
        let mut ra = RuntimeArgs::new(1, vec![]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
                .run(&mut ra, &mut cf),
            Err(RuntimeErrorType::MemoryCellDoesNotExist("a".to_string()))
        );
    }

    #[test]
    fn test_re_push_fail() {
        let mut ra = RuntimeArgs::new(1, vec![]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Push().run(&mut ra, &mut cf),
            Err(RuntimeErrorType::PushFail)
        );
    }

    #[test]
    fn test_re_pop_fail() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Pop().run(&mut ra, &mut cf),
            Err(RuntimeErrorType::PopFail)
        );
    }

    #[test]
    fn test_re_label_missing() {
        let mut ra = RuntimeArgs::new(1, vec!["a".to_string()]);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::Goto("loop".to_string()).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::LabelMissing("loop".to_string()))
        );
    }

    #[test]
    fn test_ce_me_attempt_to_divide_by_zero() {
        let mut ra = RuntimeArgs::new(2, vec![]);
        ra.accumulators[0].data = Some(0);
        ra.accumulators[1].data = Some(0);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Division, 0, 1).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToDivideByZero() })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_add() {
        let mut ra = RuntimeArgs::new(2, vec![]);
        ra.accumulators[0].data = Some(i32::MAX);
        ra.accumulators[1].data = Some(1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("add".to_string(), "Addition".to_string()) })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_sub() {
        let mut ra = RuntimeArgs::new(2, vec![]);
        ra.accumulators[0].data = Some(i32::MIN);
        ra.accumulators[1].data = Some(1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Minus, 0, 1).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("subtract".to_string(), "Subtraction".to_string()) })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_div() {
        let mut ra = RuntimeArgs::new(2, vec![]);
        ra.accumulators[0].data = Some(i32::MIN);
        ra.accumulators[1].data = Some(-1);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Division, 0, 1).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("divide".to_string(), "Division".to_string()) })
        );
    }

    #[test]
    fn test_re_ce_attempt_to_overflow_mult() {
        let mut ra = RuntimeArgs::new(2, vec![]);
        ra.accumulators[0].data = Some(i32::MAX);
        ra.accumulators[1].data = Some(2);
        let mut cf = ControlFlow::new();
        assert_eq!(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Multiplication, 0, 1).run(&mut ra, &mut cf),
            Err(RuntimeErrorType::IllegalCalculation { cause: CalcError::AttemptToOverflow("multiply".to_string(), "Multiplication".to_string()) })
        );
    }
}
