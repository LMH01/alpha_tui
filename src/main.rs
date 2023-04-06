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
        Instruction::AssignMemoryCellValueFromAccumulator("c".to_string(),1),
        Instruction::AssignAccumulatorValueFromMemoryCell(3, "c".to_string()),
        Instruction::AssignAccumulatorValueFromAccumulator(0, 2),
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
    fn new(label: String) -> Self {
        Self {
            label,
            data: None,
        }
    }
}

struct Runner {
    runtime_args: RuntimeArgs,
    instructions: Vec<Instruction>,
}

impl Runner {
    fn new(instructions: Vec<Instruction>) -> Self {
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

struct RuntimeArgs {
    /// Current values stored in accumulators
    accumulators: Vec<Accumulator>,
    /// All registers that are used to store data
    memory_cells: HashMap<String, MemoryCell>,
    /// The stack of the runner
    stack: Vec<i32>,
}

impl RuntimeArgs {
    fn new() -> Self {
        let mut accumulators = Vec::new();
        for i in 0..ACCUMULATORS {
            accumulators.push(Accumulator::new(i));
        }
        if ACCUMULATORS <= 0 {
            accumulators.push(Accumulator::new(0));
        }
        let mut memory_cells: HashMap<String, MemoryCell> = HashMap::new();
        for i in MEMORY_CELL_LABELS {
            memory_cells.insert(i.to_string(), MemoryCell::new(i.to_string()));
        }
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }
}


enum Instruction {
    /// push alpha_0 to stack 
    Push(),
    /// pop in alpha_0
    Pop(),
    /// Assigns param1 to accumulator with index param0.
    ///
    /// Errors when accumulator does not exist.
    AssignAccumulatorValue(usize, i32),
    /// Assigns value of accumulator with index param1 to accumulator with index param0.
    AssignAccumulatorValueFromAccumulator(usize, usize),
    /// Assigns value of memory cell with label param1 to accumulator with index param0.
    /// 
    /// Errors when memory cell or accumulator does not exist.
    AssignAccumulatorValueFromMemoryCell(usize, String),
    /// Assings param1 to memory cell with label param0.
    ///
    /// Errors when memory cell does not exist.
    AssignMemoryCellValue(String, i32),
    // TODO change String to &str
    /// Assigns value of accumulator with index param1 to memory cell with label param1.
    ///
    /// Errors when memory cell or accumulator does not exist.
    AssignMemoryCellValueFromAccumulator(String, usize),
    /// Prints the current contnets of the accumulators to console
    PrintAccumulators(),
    /// Prints the current contents of the memory cells
    PrintMemoryCells(),
    /// Prints the stack
    PrintStack(),
}

impl Instruction {
    /// Runs the instruction, retuns Err(String) when instruction could not be ran.
    /// Err contains the reason why running the instruction failed.
    fn run(&self, runtime_args: &mut RuntimeArgs) -> Result<(), String> {
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
                if runtime_args.accumulators.get(*target).is_some() && runtime_args.accumulators.get(*target).unwrap().data.is_some() {
                    if runtime_args.accumulators.get(*src).is_some() && runtime_args.accumulators.get(*src).unwrap().data.is_some() {
                        let replacement = runtime_args.accumulators.get(*src).unwrap();
                        runtime_args.accumulators[*target].data = replacement.data;
                    } else {
                        return Err(format!("Accumulator with index {} does not exist!", src));
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
        let mut args = RuntimeArgs::new();
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
        let mut args = RuntimeArgs::new();
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
        let mut args = RuntimeArgs::new();
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args).is_err());
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0).run(&mut args).is_err());
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell() {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.memory_cells.insert("a".to_string(), MemoryCell::new("a".to_string()));
        Instruction::AssignMemoryCellValue("a".to_string(), 10).run(&mut args).unwrap();
        Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string()).run(&mut args).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }
    
    #[test]
    fn test_assign_accumulator_value_from_memory_cell_error() {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string()).run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a".to_string(), MemoryCell::new("a".to_string()));
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a".to_string()).run(&mut args);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }
    
    #[test]
    fn test_assign_memory_cell_value() {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.memory_cells.insert("a".to_string(), MemoryCell::new("a()(".to_string()));
        args.memory_cells.insert("b".to_string(), MemoryCell::new("b".to_string()));
        Instruction::AssignMemoryCellValue("a".to_string(), 2).run(&mut args).unwrap();
        Instruction::AssignMemoryCellValue("b".to_string(), 20).run(&mut args).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_error() {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        assert!(Instruction::AssignMemoryCellValue("c".to_string(), 10).run(&mut args).is_err());
    }
}
