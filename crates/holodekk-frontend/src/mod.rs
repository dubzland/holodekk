pub mod instructions;

use std::fmt;

use crate::builder::instructions::*;
use dockerfile_parser::Instruction as DockerfileInstruction;

pub enum Instruction {
  Base(BaseInstruction),
  Add(AddInstruction),
  Cmd(CmdInstruction),
  Unknown(UnknownInstruction),
  UnknownMisc(UnknownMiscInstruction),
  // Arg(ArgInstruction),
  // Label(LabelInstruction),
  // Run(RunInstruction),
  // Entrypoint(EntrypointInstruction),
  // Copy(CopyInstruction),
  // Env(EnvInstruction),
  // Misc(MiscInstruction)
}

impl From<&dockerfile_parser::Instruction> for Instruction {
    fn from(ins: &dockerfile_parser::Instruction) -> Self {
        match ins {
            DockerfileInstruction::From(from) => {
                Instruction::Base(BaseInstruction {
                    image: from.image_parsed.image.to_owned(),
                })
            },
            DockerfileInstruction::Cmd(cmd) => {
                Instruction::Cmd(CmdInstruction {
                    content: cmd.expr.into_exec().unwrap(),
                })
            },
            DockerfileInstruction::Misc(misc) => {
                match misc.instruction.content.as_str() {
                    "ADD" => {
                        let arguments = misc.arguments.to_string();
                        let mut chunks = arguments.split_whitespace();
                        let target = chunks.next_back().unwrap().to_string();
                        Instruction::Add(AddInstruction {
                            sources: chunks.map(|s| s.to_string() ).collect(),
                            target
                        })
                    },
                    &_ => {
                        Instruction::UnknownMisc(UnknownMiscInstruction {
                            tag: misc.instruction.content.to_string(),
                            content: format!("{:?}", misc),
                        })
                    }
                }
            }
            _ => {
                println!("=======================================");
                println!("NOT IMPLEMENTED");
                println!("{:?}", ins);
                println!("=======================================");
                Instruction::Unknown(UnknownInstruction {content: format!("{:?}", ins)})
            }
        }

    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Base(ins) => write!(f, "{}", ins),
            Instruction::Add(ins) => write!(f, "{}", ins),
            Instruction::Unknown(ins) => write!(f, "{}", ins),
            Instruction::UnknownMisc(ins) => write!(f, "{}", ins),
        }
    }
}
