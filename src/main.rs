/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: i32 = 4;

fn main() {
    println!("Hello, world!");
    
    let instructions = vec![
        Instruction::AssignAccumulatorValue(0, 5),
        Instruction::AssignAccumulatorValue(1, 10),
        Instruction::Push(),
        Instruction::Pop(),
        Instruction::PrintAccumulators()];
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

/// List of memory cell labels to distinguish multiple memory cells.
enum MemoryCellLabel {
    A,
    B,
    C,
    D,
    E,
    F,
}

/// Representation of a single memory cell.
/// The term memory cell is equal to "Speicherzelle" in the SysInf lecture.
struct MemoryCell {
    label: MemoryCellLabel,
    data: Option<i32>,
}

impl MemoryCell {
    /// Creates a new register
    fn new(label: MemoryCellLabel) -> Self {
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
    memory_cells: Vec<MemoryCell>,
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
        let memory_cells = vec![
            MemoryCell::new(MemoryCellLabel::A),
            MemoryCell::new(MemoryCellLabel::B),
            MemoryCell::new(MemoryCellLabel::C),
            MemoryCell::new(MemoryCellLabel::D),
            MemoryCell::new(MemoryCellLabel::E),
            MemoryCell::new(MemoryCellLabel::F),
        ];
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }
}

enum Instruction {
    // push alpha_0 to stack 
    Push(),
    // pop in alpha_0
    Pop(),
    // Assigns param1 to accumulator with index param0.
    AssignAccumulatorValue(usize, i32),
    // Prints the current contnets of the accumulators to console
    PrintAccumulators(),
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
                    return Err(format!("Accumulator with index {} does not exist!", a).to_string());
                }
            }
            Self::PrintAccumulators() => {
                println!("--- Accumulators ---");
                for (index, i) in runtime_args.accumulators.iter().enumerate() {
                    println!("{} - {:?}", index, i.data);
                }
                println!("--------------------");
            },
        }
        Ok(())
    }
}

