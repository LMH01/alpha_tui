use std::collections::HashMap;

/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: i32 = 4;
/// Used to set the available memory cells.
const MEMORY_CELL_LABELS: &'static [&'static str] = &["a", "b", "c", "d", "e", "f"];

fn main() {
    println!("Hello, world!");
    
    let instructions = vec![
        Instruction::AssignAccumulatorValue(0, 5),
        Instruction::AssignAccumulatorValue(1, 10),
        Instruction::AssignAccumulatorValue(2, 7),
        Instruction::Push(),
        Instruction::Pop(),
        Instruction::AssignMemoryCellValueFromAccumulator("c",1),
        Instruction::AssignAccumulatorValueFromMemoryCell(3, "c"),
        Instruction::AssignAccumulatorValueFromAccumulator(0, 2),
        Instruction::AssignMemoryCellValue("f", 17),
        Instruction::PrintAccumulators(),
        Instruction::PrintMemoryCells(),
        Instruction::PrintStack(),
    ];
    let mut runner = Runner::new(instructions);
    runner.run();
}

/// A single accumulator, represents "Akkumulator/Alpha" from SysInf lecture.
struct Accumulator {
    /// Used to identify accumulator
    id: i32,
    /// The data stored in the Accumulator
    data: Option<i32>,
}

impl Accumulator {
    /// Creates a new accumulator
    fn new(id: i32) -> Self {
        Self {
            id,
            data: None,
        }
    }
}

/// Representation of a single memory cell.
/// The term memory cell is equal to "Speicherzelle" in the SysInf lecture.
struct MemoryCell {
    label: String,
    data: Option<i32>,
}

impl MemoryCell {
    /// Creates a new register
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            data: None,
        }
    }
}

struct Runner<'a> {
    runtime_args: RuntimeArgs<'a>,
    instructions: Vec<Instruction<'a>>,
}

impl<'a> Runner<'a> {
    fn new(instructions: Vec<Instruction<'a>>) -> Self {
        Self {
            runtime_args: RuntimeArgs::new(),
            instructions,
        }
    }

    fn run(&mut self) {
        for instruction in &self.instructions {
            instruction.run(&mut self.runtime_args);
        }
    }
}

struct RuntimeArgs<'a> {
    /// Current values stored in accumulators
    accumulators: Vec<Accumulator>,
    /// All registers that are used to store data
    memory_cells: HashMap<&'a str, MemoryCell>,
    /// The stack of the runner
    stack: Vec<i32>,
}

impl<'a> RuntimeArgs<'a> {
    fn new() -> Self {
        let mut accumulators = Vec::new();
        for i in 0..ACCUMULATORS {
            accumulators.push(Accumulator::new(i));
        }
        if ACCUMULATORS <= 0 {
            accumulators.push(Accumulator::new(0));
        }
        let mut memory_cells: HashMap<&str, MemoryCell> = HashMap::new();
        for i in MEMORY_CELL_LABELS {
            memory_cells.insert(i, MemoryCell::new(i));
        }
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }
}


enum Instruction<'a> {
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
    fn run(&self, runtime_args: &mut RuntimeArgs<'a>) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{RuntimeArgs, Instruction, MemoryCell, Accumulator};

    
    #[test]
    fn test_stack() {
        let mut args = setup_runtime_args();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args).unwrap();
        Instruction::Push().run(&mut args).unwrap();
        Instruction::AssignAccumulatorValue(0, 10).run(&mut args).unwrap();
        Instruction::Push().run(&mut args).unwrap();
        assert_eq!(args.stack, vec![5, 10]);
        Instruction::Pop().run(&mut args).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
        Instruction::Pop().run(&mut args).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 5);
        assert_eq!(args.stack.len(), 0);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator() {
        let mut args = setup_runtime_args();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValue(1, 20).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValue(2, 12).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(1, 2).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args).unwrap();
        assert_eq!(args.accumulators[1].data.unwrap(), 12);
        assert_eq!(args.accumulators[2].data.unwrap(), 12);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args).is_err());
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0).run(&mut args).is_err());
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        Instruction::AssignMemoryCellValue("a", 10).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }
    
    #[test]
    fn test_assign_accumulator_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a").run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }
    
    #[test]
    fn test_assign_memory_cell_value() {
        let mut args = setup_runtime_args();
        Instruction::AssignMemoryCellValue("a", 2).run(&mut args).unwrap();
        Instruction::AssignMemoryCellValue("b", 20).run(&mut args).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_error() {
        let mut args = setup_empty_runtime_args();
        assert!(Instruction::AssignMemoryCellValue("c", 10).run(&mut args).is_err());
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator() {
        let mut args = setup_runtime_args();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args).unwrap();
        Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 1).run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        Instruction::AssignMemoryCellValue("a", 20).run(&mut args).unwrap();
        Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args).unwrap();
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args).is_err());
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.memory_cells.get_mut("b").unwrap().data = Some(10);
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args).is_ok());
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
