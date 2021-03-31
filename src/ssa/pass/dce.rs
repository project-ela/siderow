use std::collections::{HashMap, HashSet};

use crate::ssa::{Function, InstructionId, Module};

pub fn apply(module: &mut Module) {
    DeadCodeElimination::new().apply(module);
}

struct DeadCodeElimination {}

impl DeadCodeElimination {
    fn new() -> Self {
        Self {}
    }

    fn apply(&mut self, module: &mut Module) {
        for (_, function) in module.functions.iter_mut() {
            self.apply_function(function);
        }
    }

    fn apply_function(&mut self, function: &mut Function) {
        let mut ids_to_eliminate = HashSet::new();
        let mut new_users_map = HashMap::new();
        for (_, block) in function.blocks.iter().rev() {
            for inst_id in block.instructions.iter().rev() {
                let inst = function.inst(*inst_id).unwrap();
                let new_users: HashSet<InstructionId> =
                    inst.users.difference(&ids_to_eliminate).copied().collect();

                let can_be_eliminated = new_users.len() == 0;
                if can_be_eliminated {
                    ids_to_eliminate.insert(*inst_id);
                } else {
                    new_users_map.insert(*inst_id, new_users);
                }
            }
        }

        for (_, block) in function.blocks.iter_mut() {
            block
                .instructions
                .retain(|inst_id| !ids_to_eliminate.contains(inst_id));
        }
        for (inst_id, new_users) in new_users_map.into_iter() {
            let inst = function.inst_mut(inst_id).unwrap();
            let _ = std::mem::replace(&mut inst.users, new_users);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeadCodeElimination;
    use crate::ssa::{Block, Function, FunctionBuilder, Type, Value};

    #[test]
    fn dce_1() {
        let mut func_main = Function::new("main", Type::I32, vec![]);
        let mut builder = FunctionBuilder::new(&mut func_main);
        let block_0 = builder.add_block();
        let block_1 = builder.add_block();

        builder.set_block(block_0);
        let one = Value::new_i32(1);
        let v0 = builder.add(one, one);
        let v1 = builder.add(v0, v0);
        let _v2 = builder.add(v0, v0);
        builder.br(block_1);

        builder.set_block(block_1);
        let _v3 = builder.add(v0, v0);
        builder.ret(v1);

        // ---

        DeadCodeElimination::new().apply_function(&mut func_main);
        assert_eq!(inst_indices(func_main.block(block_0).unwrap()), vec![0, 1]);
        assert_eq!(inst_indices(func_main.block(block_1).unwrap()), vec![]);
    }

    fn inst_indices(block: &Block) -> Vec<usize> {
        block
            .instructions
            .iter()
            .map(|inst_id| inst_id.index())
            .collect()
    }
}