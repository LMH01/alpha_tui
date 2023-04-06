fn main() {
    println!("Hello, world!");
    
    let instructions = vec![Instruction::PushStack(2),
        Instruction::PopStack(),
        Instruction::PrintRegister()];
    let mut runner = Runner::new(instructions);
    runner.run();
}

/// All registers that can contain data
enum RegisterLabel {
    A,
    B,
    C,
    D,
    E,
    F,
}

/// Representation of a single register, containing data 
struct Register {
    data: Option<i32>,
    label: RegisterLabel,
}

impl Register {
    /// Creates a new register
    fn new(label: RegisterLabel) -> Self {
        Self {
            data: None,
            label,
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
    registers: Vec<Register>,
    stack: Vec<i32>,
}

impl RuntimeArgs {
    fn new() -> Self {
        let registers = vec![
            Register::new(RegisterLabel::A),
            Register::new(RegisterLabel::B),
            Register::new(RegisterLabel::C),
            Register::new(RegisterLabel::D),
            Register::new(RegisterLabel::E),
            Register::new(RegisterLabel::F),
        ];
        Self {
            registers,
            stack: Vec::new(),
        }
    }
}

enum Instruction {
    // push value to stack 
    PushStack(i32),
    // pop in register 
    PopStack(),
    // Prints the current contnet of the register to console
    PrintRegister(),
}

impl Instruction {
    fn run(&self, runtime_args: &mut RuntimeArgs) {
        match self {
            Self::PopStack() => {
                runtime_args.register = runtime_args.stack.pop().unwrap_or(0);
            },
            Self::PushStack(x) => {
                runtime_args.stack.push(*x);
            },
            Self::PrintRegister() => {
                println!("{}", runtime_args.register);
            },
        }
    }
}

