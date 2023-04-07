use crate::{runtime::{RuntimeArgs, ControlFlow}, base::Comparison};

pub enum Instruction<'a> {
    /// push **alpha_0** to stack 
    Push(),
    /// pop in **alpha_0**
    Pop(),
    /// Assigns **param1** to accumulator with index **param0**.
    ///
    /// Errors when accumulator does not exist.
    AssignAccumulatorValue(usize, i32),
    /// Assigns value of accumulator with index **param1** to accumulator with index **param0**.
    AssignAccumulatorValueFromAccumulator(usize, usize),
    /// Assigns value of memory cell with label **param1** to accumulator with index **param0**.
    /// 
    /// Errors when memory cell or accumulator does not exist.
    AssignAccumulatorValueFromMemoryCell(usize, &'a str),
    /// Assings **param1** to memory cell with label **param0**.
    ///
    /// Errors when memory cell does not exist.
    AssignMemoryCellValue(&'a str, i32),
    // TODO change String to &str
    /// Assigns value of accumulator with index **param1** to memory cell with label **param0**.
    ///
    /// Errors when memory cell or accumulator does not exist.
    AssignMemoryCellValueFromAccumulator(&'a str, usize),
    /// Assings value of memory cell with label **param1** to memory cell with label **param0**.
    AssingMemoryCellValueFromMemoryCell(&'a str, &'a str),
    /// Sets next instruction to instruction with label **param1**.
    /// 
    /// See [ControlFlow](../runtime/struct.ControlFlow.html) for further information.
    Goto(&'a str),
    /// See [goto_if_accumulator](fn.goto_if_accumulator.html)
    GotoIfAccumulator(Comparison, &'a str, usize, usize),
    /// See [goto_if_constant](fn.goto_if_constant.html)
    GotoIfConstant(Comparison, &'a str, usize, i32),
    /// See [goto_if_memory_cell](fn.goto_if_memory_cell.html)
    GotoIfMemoryCell(Comparison, &'a str, usize, &'a str),
    /// Prints the current contnets of the accumulators to console
    PrintAccumulators(),
    /// Prints the current contents of the memory cells
    PrintMemoryCells(),
    /// Prints the stack
    PrintStack(),
}

impl<'a> Instruction<'a> {
    /// Runs the instruction, retuns Err(String) when instruction could not be ran.
    /// Err contains the reason why running the instruction failed.
    pub fn run(&self, runtime_args: &mut RuntimeArgs<'a>, control_flow: &mut ControlFlow<'a>) -> Result<(), String> {
        match self {
            Self::Push() => {
                runtime_args.stack.push(runtime_args.accumulators[0].data.unwrap_or(0));
            },
            Self::Pop() => {
                runtime_args.accumulators[0].data = Some(runtime_args.stack.pop().unwrap_or(0));
            },
            Self::AssignAccumulatorValue(a,x) => {
                if let Some(y) = runtime_args.accumulators.get_mut(*a) {
                    y.data = Some(*x);
                } else {
                    return Err(format!("Accumulator with index {} does not exist!", a));
                }
            },
            Self::AssignAccumulatorValueFromAccumulator(target, src) => {
                if runtime_args.accumulators.get(*target).is_some() {
                    if runtime_args.accumulators.get(*src).is_some() && runtime_args.accumulators.get(*src).unwrap().data.is_some() {
                        let replacement = runtime_args.accumulators.get(*src).unwrap();
                        runtime_args.accumulators[*target].data = replacement.data;
                    } else {
                        return Err(format!("Accumulator with index {} does not exist or does not contain data!", src));
                    }
                } else {
                    return Err(format!("Accumulator with index {} does not exist!", target));
                }
            }
            Self::AssignAccumulatorValueFromMemoryCell(a, cell_label) => {
                if let Some(ac) = runtime_args.accumulators.get_mut(*a) {
                    if let Some(cell) = runtime_args.memory_cells.get_mut(cell_label) {
                        ac.data = cell.data;
                    } else {
                        return Err(format!("Memory cell labeled {} does not exist!", cell_label));
                    }
                } else {
                    return Err(format!("Accumulator with index {} does not exist!", a));
                }
            },
            Self::AssignMemoryCellValue(cell_label, value) => {
                if let Some(cell) = runtime_args.memory_cells.get_mut(cell_label) {
                    cell.data = Some(*value);
                } else {
                    return Err(format!("Memory cell labeled {} does not exist!", cell_label));
                }
            }
            Self::AssignMemoryCellValueFromAccumulator(cell_label, a) => {
                if let Some(cell) = runtime_args.memory_cells.get_mut(cell_label) {
                    if let Some(ac) = runtime_args.accumulators.get_mut(*a) {
                        cell.data = ac.data;
                    } else {
                        return Err(format!("Accumulator with index {} does not exist!", a));
                    }
                } else {
                    return Err(format!("Memory cell labeled {} does not exist!", cell_label));
                }
            },
            Self::AssingMemoryCellValueFromMemoryCell(target, src) => {
                if runtime_args.memory_cells.get(*target).is_some() {
                    if runtime_args.memory_cells.get(*src).is_some() && runtime_args.memory_cells.get(*src).unwrap().data.is_some() {
                        let replacement = runtime_args.memory_cells.get(*src).unwrap();
                        runtime_args.memory_cells.get_mut(*target).unwrap().data = replacement.data;
                    } else {
                        return Err(format!("Memory cell labeled {} does not exist or does not contain data!", src));
                    }
                } else {
                    return Err(format!("Memory cell labeled {} does not exist!", target));
                }
            },
            Self::Goto(label) => {
                if let Some(index) = control_flow.instruction_labels.get(label) {
                    control_flow.next_instruction_index = *index;
                } else {
                    return Err(format!("Unable to go to label {}: No instruction found for that label!", label));
                }
            }
            Self::GotoIfAccumulator(comparison, label, a_idx_a, a_idx_b) => {
                goto_if_accumulator(runtime_args, control_flow, comparison, label, a_idx_a, a_idx_b)?
            }
            Self::GotoIfConstant(comparison, label, a_idx, c) => {
                goto_if_constant(runtime_args, control_flow, comparison, label, a_idx, c)?;
            }
            Self::GotoIfMemoryCell(comparison, label, a_idx, mcl) => {
                goto_if_memory_cell(runtime_args, control_flow, comparison, label, a_idx, mcl)?;
            }
            Self::PrintAccumulators() => {
                println!("--- Accumulators ---");
                for (index, i) in runtime_args.accumulators.iter().enumerate() {
                    println!("{} - {:?}", index, i.data);
                }
                println!("--------------------");
            },
            Self::PrintMemoryCells() => {
                // TODO Make print sorted (Alpabetically by label name)
                println!("--- Memory Cells ---");
                for (k, v) in &runtime_args.memory_cells {
                    println!("{} - {:?}", k, v.data);
                }
                println!("--------------------");
            },
            Self::PrintStack() => {
                println!("------ Stack -------");
                for (index, i) in runtime_args.stack.iter().enumerate() {
                    println!("{} - {:?}", index, i);
                }
                println!("--------------------");
            }
        }
        Ok(())
    }
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx_a** and accumulator **a_idx_b** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx_a' - The index of accumulator a whichs value should be compared
/// - 'a_idx_b' - The index of accumulator b whichs value should be compared
fn goto_if_accumulator<'a>(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &'a str, a_idx_a: &usize, a_idx_b: &usize) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx** and constant **c** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx' - The index of accumulator a whichs value should be compared
/// - 'c' - Constant that should be compared with **a_idx_a**
fn goto_if_constant<'a>(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &'a str, a_idx: &usize, c: &i32) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    if comparison.cmp(a, *c) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx** and memory cell with label **mcl** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx' - The index of accumulator a whichs value should be compared
/// - 'mcl' - The label of the memory cell behind wich the value can be found that should be compared
fn goto_if_memory_cell<'a>(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &'a str, a_idx: &usize, mcl: &'a str) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, mcl)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Tests if the accumulator with **index** exists and contains a value.
/// 
/// Ok(i32) contains the accumulator value.
/// 
/// Err(String) contains error message.
fn assert_accumulator_contains_value(runtime_args: &mut RuntimeArgs, index: &usize) -> Result<i32, String> {
    if let Some(value) = runtime_args.accumulators.get(*index) {
        if value.data.is_some() {
            Ok(runtime_args.accumulators.get(*index).unwrap().data.unwrap())
        } else {
            Err(format!("Accumulator with index {} does not contain data!", index))
        }
    } else {
        Err(format!("Accumulator with index {} does not exist!", index))
    }
}

/// Tests if the memory cell with **label** exists and contains a value.
/// 
/// Ok(i32) contains the memory cell value.
/// 
/// Err(String) contains error message.
fn assert_memory_cell_contains_value(runtime_args: &mut RuntimeArgs, label: &str) -> Result<i32, String> {
    if let Some(value) = runtime_args.memory_cells.get(label) {
        if value.data.is_some() {
            Ok(runtime_args.memory_cells.get(label).unwrap().data.unwrap())
        } else {
            Err(format!("Memory cell with label {} does not contain data!", label))
        }
    } else {
        Err(format!("Memory cell with label {} does not exist!", label))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{runtime::{ControlFlow, RuntimeArgs}, instructions::Instruction, base::{Accumulator, MemoryCell, Comparison}};

    
    #[test]
    fn test_stack() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args, &mut control_flow).unwrap();
        Instruction::Push().run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(0, 10).run(&mut args, &mut control_flow).unwrap();
        Instruction::Push().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.stack, vec![5, 10]);
        Instruction::Pop().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
        Instruction::Pop().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 5);
        assert_eq!(args.stack.len(), 0);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(1, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(2, 12).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(1, 2).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[1].data.unwrap(), 12);
        assert_eq!(args.accumulators[2].data.unwrap(), 12);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0).run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 10).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }
    
    #[test]
    fn test_assign_accumulator_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a").run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }
    
    #[test]
    fn test_assign_memory_cell_value() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 2).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValue("b", 20).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignMemoryCellValue("c", 10).run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 1).run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).is_err());
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.memory_cells.get_mut("b").unwrap().data = Some(10);
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_goto() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 5);
        Instruction::Goto("loop").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 5);
    }

    #[test]
    fn test_goto_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::Goto("loop").run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_goto_if_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(1, 30).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfAccumulator(Comparison::Less, "loop", 0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfAccumulator(Comparison::Equal, "loop", 0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfAccumulator(Comparison::Less, "none", 0, 1).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfAccumulator(Comparison::Equal, "none", 0, 1).run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_goto_if_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfConstant(Comparison::Less, "loop", 0, 40).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfConstant(Comparison::Equal, "loop", 0, 40).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfConstant(Comparison::Less, "none", 0, 40).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfConstant(Comparison::Equal, "none", 0, 40).run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_goto_if_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValue("a", 50).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfMemoryCell(Comparison::Less, "loop", 0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfMemoryCell(Comparison::Equal, "loop", 0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfMemoryCell(Comparison::Less, "none", 0, "a").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfMemoryCell(Comparison::Equal, "none", 0, "a").run(&mut args, &mut control_flow).is_ok());
    }

    /// Sets up runtime args in a conistent way because the default implementation for memory cells and accumulators is configgurable.
    fn setup_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.accumulators = vec![Accumulator::new(0), Accumulator::new(1), Accumulator::new(2)];
        args
    }

    /// Sets up runtime args where no memory cells or accumulators are set.
    fn setup_empty_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.accumulators = Vec::new();
        args.memory_cells = HashMap::new();
        args
    }
}