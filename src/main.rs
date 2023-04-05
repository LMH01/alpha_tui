fn main() {
    println!("Hello, world!");
    
    let mut instructions = Vec::new();
    instructions.push(Instruction::PushStack(2));
    instructions.push(Instruction::PopStack());
    instructions.push(Instruction::PrintRegister());
    let mut runner = Runner::new(instructions);
    runner.run();
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
    register: i32,
    stack: Vec<i32>,
}

impl RuntimeArgs {
    fn new() -> Self {
        Self {
            register: 0,
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

