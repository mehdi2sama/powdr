#![deny(clippy::print_stdout)]

use powdr_ast::asm_analysis::{AnalysisASMFile, Item};
use powdr_number::FieldElement;
use romgen::generate_machine_rom;
mod common;
mod romgen;
mod vm_to_constrained;

/// Remove all ASM from the machine tree. Takes a tree of virtual or constrained machines and returns a tree of constrained machines
pub fn compile<T: FieldElement>(file: AnalysisASMFile<T>) -> AnalysisASMFile<T> {
    AnalysisASMFile {
        items: file
            .items
            .into_iter()
            .map(|(name, m)| {
                (
                    name,
                    match m {
                        Item::Machine(m) => {
                            let (m, rom) = generate_machine_rom(m);
                            Item::Machine(vm_to_constrained::convert_machine(m, rom))
                        }
                        Item::Expression(e) => Item::Expression(e),
                    },
                )
            })
            .collect(),
    }
}

pub mod utils {
    use powdr_ast::{
        asm_analysis::{
            AssignmentStatement, FunctionStatement, Instruction, InstructionDefinitionStatement,
            InstructionStatement, LabelStatement, RegisterDeclarationStatement, RegisterTy,
        },
        parsed::{
            asm::{AssignmentRegister, InstructionBody, MachineStatement, RegisterFlag},
            PilStatement,
        },
    };
    use powdr_number::FieldElement;
    use powdr_parser::{
        powdr::{
            FunctionStatementParser, InstructionBodyParser, InstructionDeclarationParser,
            InstructionParser, PilStatementParser, RegisterDeclarationParser,
        },
        ParserContext,
    };

    lazy_static::lazy_static! {
        static ref INSTRUCTION_DECLARATION_PARSER: InstructionDeclarationParser = InstructionDeclarationParser::new();
        static ref INSTRUCTION_PARSER: InstructionParser = InstructionParser::new();
        static ref INSTRUCTION_BODY_PARSER: InstructionBodyParser = InstructionBodyParser::new();
        static ref FUNCTION_STATEMENT_PARSER: FunctionStatementParser = FunctionStatementParser::new();
        static ref PIL_STATEMENT_PARSER: PilStatementParser = PilStatementParser::new();
        static ref REGISTER_DECLARATION_PARSER: RegisterDeclarationParser = RegisterDeclarationParser::new();

    }

    pub fn parse_instruction_definition<T: FieldElement>(
        input: &str,
    ) -> InstructionDefinitionStatement<T> {
        let ctx = ParserContext::new(None, input);
        match INSTRUCTION_DECLARATION_PARSER.parse(&ctx, input).unwrap() {
            MachineStatement::InstructionDeclaration(source, name, instruction) => {
                InstructionDefinitionStatement {
                    source,
                    name,
                    instruction: Instruction {
                        params: instruction.params,
                        body: instruction.body,
                    },
                }
            }
            _ => panic!(),
        }
    }

    pub fn parse_instruction<T: FieldElement>(input: &str) -> Instruction<T> {
        let ctx = ParserContext::new(None, input);
        let instr = INSTRUCTION_PARSER.parse(&ctx, input).unwrap();
        Instruction {
            params: instr.params,
            body: instr.body,
        }
    }

    pub fn parse_instruction_body<T: FieldElement>(input: &str) -> InstructionBody<T> {
        let ctx = ParserContext::new(None, input);
        INSTRUCTION_BODY_PARSER.parse(&ctx, input).unwrap()
    }

    pub fn parse_function_statement<T: FieldElement>(input: &str) -> FunctionStatement<T> {
        let ctx = ParserContext::new(None, input);
        match FUNCTION_STATEMENT_PARSER.parse(&ctx, input).unwrap() {
            powdr_ast::parsed::asm::FunctionStatement::Assignment(source, lhs, reg, rhs) => {
                AssignmentStatement {
                    source,
                    lhs_with_reg: {
                        let lhs_len = lhs.len();
                        lhs.into_iter()
                            .zip(reg.unwrap_or(vec![AssignmentRegister::Wildcard; lhs_len]))
                            .collect()
                    },
                    rhs,
                }
                .into()
            }
            powdr_ast::parsed::asm::FunctionStatement::Instruction(source, instruction, inputs) => {
                InstructionStatement {
                    source,
                    instruction,
                    inputs,
                }
                .into()
            }
            powdr_ast::parsed::asm::FunctionStatement::Label(source, name) => {
                LabelStatement { source, name }.into()
            }
            _ => unimplemented!(),
        }
    }

    pub fn parse_pil_statement<T: FieldElement>(input: &str) -> PilStatement<T> {
        let ctx = ParserContext::new(None, input);
        PIL_STATEMENT_PARSER.parse(&ctx, input).unwrap()
    }

    pub fn parse_register_declaration<T: FieldElement>(
        input: &str,
    ) -> RegisterDeclarationStatement {
        let ctx = ParserContext::new(None, input);
        match REGISTER_DECLARATION_PARSER.parse::<T>(&ctx, input).unwrap() {
            MachineStatement::RegisterDeclaration(source, name, flag) => {
                let ty = match flag {
                    Some(RegisterFlag::IsAssignment) => RegisterTy::Assignment,
                    Some(RegisterFlag::IsPC) => RegisterTy::Pc,
                    Some(RegisterFlag::IsReadOnly) => RegisterTy::ReadOnly,
                    None => RegisterTy::Write,
                };
                RegisterDeclarationStatement { source, name, ty }
            }
            _ => unreachable!(),
        }
    }
}
