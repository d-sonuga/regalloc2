use crate::cfg::CFGInfo;
use crate::checker::CheckerError;
use crate::ion::Stats;
use crate::ssa::validate_ssa;
use crate::OperandConstraint::{self, *};
use crate::OperandKind::{self, *};
use crate::{
    checker, run, Algorithm, Allocation, Block, Function, Inst, InstRange, MachineEnv, Operand,
    OperandPos, Output, PReg, PRegSet, ProgPoint, RegClass, RegallocOptions, VReg,
};
use alloc::vec;
use alloc::vec::Vec;

use std::println;

#[test]
fn test_debug_locations1() {
    let mach_env = mach_env(10);
    let mut options = RegallocOptions::default();
    options.validate_ssa = true;
    options.algorithm = Algorithm::Fastalloc;
    let mut f = RealFunction::new(vec![BlockBuildInfo {
        insts: vec![
            /* 0. */ inst(vec![op(Def, 0, FixedReg(p(0)))]),
            /* 1. */
            inst(vec![
                op(Def, 1, FixedReg(p(0))),
                op(Use, 0, FixedReg(p(0))),
                op(Use, 0, Reg),
            ]),
            /* 2. */
            inst(vec![
                op(Def, 2, FixedReg(p(8))),
                op(Use, 0, FixedReg(p(2))),
                op(Use, 1, FixedReg(p(0))),
            ]),
            /* 3. */ inst(vec![op(Def, 3, FixedReg(p(9))), op(Use, 0, FixedReg(p(9)))]),
            ret(),
        ],
    }]);
    f.debug_value_labels = vec![
        (v(0), i(0), i(4), 32),
        (v(2), i(2), i(4), 70),
        (v(2), i(2), i(4), 71),
        (v(3), i(3), i(4), 34),
    ];
    let result = run(&f, &mach_env, &options).unwrap();
    assert_eq!(
        result.debug_locations,
        vec![
            (
                32,
                ProgPoint::after(i(0)),
                ProgPoint::after(i(3)),
                alloc(p(9))
            ),
            (
                34,
                ProgPoint::after(i(3)),
                ProgPoint::before(i(4)),
                alloc(p(9))
            ),
            (
                70,
                ProgPoint::after(i(2)),
                ProgPoint::before(i(3)),
                alloc(p(8))
            ),
            (
                71,
                ProgPoint::after(i(2)),
                ProgPoint::before(i(3)),
                alloc(p(8))
            ),
        ]
    );
}

#[test]
fn test_debug_locations2() {
    let mach_env = mach_env(2);
    let mut options = RegallocOptions::default();
    options.validate_ssa = true;
    options.algorithm = Algorithm::Fastalloc;
    let mut f = RealFunction::new(vec![BlockBuildInfo {
        insts: vec![
            /* 0. */ inst(vec![op(Def, 2, FixedReg(p(0)))]),
            /* 1. */ inst(vec![op(Def, 0, FixedReg(p(0)))]),
            /* 2. */ inst(vec![op(Def, 1, FixedReg(p(1)))]),
            /* 3. */ inst(vec![op(Use, 0, FixedReg(p(0))), op(Use, 0, FixedReg(p(1)))]),
            /* 4. */ inst(vec![op(Use, 1, FixedReg(p(1)))]),
            ret(),
        ],
    }]);
    f.debug_value_labels = vec![
        (v(0), i(1), i(4), 10),
        (v(1), i(0), i(1), 11),
        (v(1), i(2), i(3), 23),
    ];
    let result = run(&f, &mach_env, &options).unwrap();
    assert_eq!(result.debug_locations.len(), 2);
    assert_eq!(
        result.debug_locations[0],
        (
            10,
            ProgPoint::after(i(1)),
            ProgPoint::after(i(3)),
            alloc(p(0))
        )
    );
    assert_eq!(result.debug_locations[1].0, 23);
    assert_eq!(result.debug_locations[1].1, ProgPoint::after(i(2)));
    assert_eq!(result.debug_locations[1].2, ProgPoint::after(i(4)));
    assert!(matches!(result.debug_locations[1].3.as_stack(), Some(_)));
}

#[test]
fn test_fuzzer() {
    let mach_env = mach_env(10);
    let f = RealFunction::new(vec![
        BlockBuildInfo {
            insts: vec![branch(
                vec![op(Def, 0, FixedReg(p(0)))],
                vec![Block::new(1)],
            )],
        },
        BlockBuildInfo {
            insts: vec![inst(vec![op(Use, 0, FixedReg(p(0)))]), ret()],
        },
    ]);
    validate_ssa(&f, &CFGInfo::new(&f).unwrap()).unwrap();
    let output = Output {
        num_spillslots: 0,
        edits: Vec::new(),
        allocs: vec![Allocation::reg(p(0)), Allocation::reg(p(0))],
        inst_alloc_offsets: vec![0, 1, 2],
        debug_locations: vec![],
        stats: Stats::default(),
    };
    let mut checker = checker::Checker::new(&f, &mach_env);
    checker.prepare(&output);
    let res = checker.run();
    use std::println;
    println!("{res:?}");
    assert!(res.is_ok());
}

#[test]
fn test_fuzzer2() {
    let mach_env = mach_env(10);
    let f = RealFunction::new(vec![
        BlockBuildInfo {
            insts: vec![
                inst(vec![op(Def, 0, FixedReg(p(0)))]),
                branch(vec![op(Use, 0, FixedReg(p(1)))], vec![Block::new(1)]),
            ],
        },
        BlockBuildInfo {
            insts: vec![inst(vec![op(Use, 0, FixedReg(p(0)))]), ret()],
        },
    ]);
    validate_ssa(&f, &CFGInfo::new(&f).unwrap()).unwrap();
    let output = Output {
        num_spillslots: 0,
        edits: Vec::new(),
        allocs: vec![
            Allocation::reg(p(0)),
            Allocation::reg(p(9)),
            Allocation::reg(p(0)),
        ],
        inst_alloc_offsets: vec![0, 1, 2, 3],
        debug_locations: vec![],
        stats: Stats::default(),
    };
    let mut checker = checker::Checker::new(&f, &mach_env);
    checker.prepare(&output);
    let res = checker.run();
    use std::println;
    println!("{res:?}");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().errors,
        vec![CheckerError::UnknownValueInAllocation {
            inst: Inst(1),
            op: op(Use, 0, FixedReg(p(1))),
            alloc: Allocation::reg(p(9))
        }]
    );
}

impl RealFunction {
    fn new(blocks: Vec<BlockBuildInfo>) -> Self {
        let mut f = Self::default();
        let mut max_vreg_num_seen = 0;
        for block in blocks.iter() {
            let mut real_block = RealBlock {
                params: vec![],
                preds: vec![],
                succs: vec![],
            };
            let start_inst_idx = f.insts.len();
            for inst in block.insts.iter() {
                f.insts.push(RealInst {
                    inst: Inst::new(f.insts.len()),
                    kind: inst.kind.clone(),
                });
                let start_op_idx = f.operands.len();
                for op in inst.operands.iter() {
                    max_vreg_num_seen = max_vreg_num_seen.max(op.vreg().vreg());
                    f.operands.push(*op);
                }
                f.operand_ranges.push((start_op_idx, f.operands.len()));
            }
            if !block.insts.is_empty() {
                if let RealInstKind::Branch(ref succs) = f.insts.last().unwrap().kind {
                    real_block.succs.extend(succs.iter().copied())
                }
            }
            f.inst_ranges.push((start_inst_idx, f.insts.len()));
            f.blocks.push(real_block);
        }
        f.num_vregs = max_vreg_num_seen + 1;
        for idx in 0..f.blocks.len() {
            let succs = f.blocks[idx].succs.clone();
            for succ_idx in succs.iter().copied() {
                f.blocks[succ_idx.index()].preds.push(Block::new(idx));
            }
        }
        f
    }
}

fn mach_env(no_of_regs: usize) -> MachineEnv {
    MachineEnv {
        preferred_regs_by_class: [
            (0..no_of_regs)
                .map(|no| PReg::new(no, RegClass::Int))
                .collect(),
            vec![],
            vec![],
        ],
        non_preferred_regs_by_class: [vec![], vec![], vec![]],
        scratch_by_class: [None, None, None],
        fixed_stack_slots: vec![],
    }
}

fn op(kind: OperandKind, vreg_num: usize, constraint: OperandConstraint) -> Operand {
    Operand::new(
        VReg::new(vreg_num, RegClass::Int),
        constraint,
        kind,
        match kind {
            Use => OperandPos::Early,
            Def => OperandPos::Late,
        },
    )
}

fn alloc(preg: PReg) -> Allocation {
    Allocation::reg(preg)
}

fn v(vreg_num: usize) -> VReg {
    VReg::new(vreg_num, RegClass::Int)
}

fn i(inst: usize) -> Inst {
    Inst::new(inst)
}

fn p(hw_enc: usize) -> PReg {
    PReg::new(hw_enc, RegClass::Int)
}

struct RealInstInfo {
    operands: Vec<Operand>,
    kind: RealInstKind,
}

fn inst(operands: Vec<Operand>) -> RealInstInfo {
    RealInstInfo {
        operands,
        kind: RealInstKind::Normal,
    }
}

fn branch(operands: Vec<Operand>, block: Vec<Block>) -> RealInstInfo {
    RealInstInfo {
        operands,
        kind: RealInstKind::Branch(block),
    }
}

fn ret() -> RealInstInfo {
    RealInstInfo {
        operands: vec![],
        kind: RealInstKind::Ret,
    }
}

struct BlockBuildInfo {
    insts: Vec<RealInstInfo>,
}

#[derive(Default)]
struct RealFunction {
    blocks: Vec<RealBlock>,
    insts: Vec<RealInst>,
    operands: Vec<Operand>,
    operand_ranges: Vec<(usize, usize)>,
    inst_ranges: Vec<(usize, usize)>,
    num_vregs: usize,
    debug_value_labels: Vec<(VReg, Inst, Inst, u32)>,
}

struct RealBlock {
    params: Vec<VReg>,
    preds: Vec<Block>,
    succs: Vec<Block>,
}

struct RealInst {
    inst: Inst,
    kind: RealInstKind,
}

impl RealInst {
    fn is_branch(&self) -> bool {
        match self.kind {
            RealInstKind::Branch(_) => true,
            _ => false,
        }
    }

    fn is_ret(&self) -> bool {
        match self.kind {
            RealInstKind::Ret => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
enum RealInstKind {
    Normal,
    Branch(Vec<Block>),
    Ret,
}

impl Function for RealFunction {
    fn num_insts(&self) -> usize {
        self.insts.len()
    }

    fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    fn block_insns(&self, block: crate::Block) -> crate::InstRange {
        let (start, end) = self.inst_ranges[block.index()];
        if start != end {
            InstRange::new(
                self.insts[start].inst,
                Inst::new(self.insts[end - 1].inst.index() + 1),
            )
        } else {
            InstRange::new(Inst::new(0), Inst::new(0))
        }
    }

    fn allow_multiple_vreg_defs(&self) -> bool {
        false
    }

    fn block_params(&self, block: crate::Block) -> &[VReg] {
        &self.blocks[block.index()].params
    }

    fn block_preds(&self, block: crate::Block) -> &[crate::Block] {
        &self.blocks[block.index()].preds
    }

    fn block_succs(&self, block: Block) -> &[Block] {
        &self.blocks[block.index()].succs
    }

    fn debug_value_labels(&self) -> &[(VReg, Inst, Inst, u32)] {
        &self.debug_value_labels
    }

    fn entry_block(&self) -> Block {
        Block::new(0)
    }

    fn inst_clobbers(&self, _insn: Inst) -> crate::PRegSet {
        PRegSet::empty()
    }

    fn inst_operands(&self, insn: Inst) -> &[Operand] {
        let (start, end) = self.operand_ranges[insn.index()];
        &self.operands[start..end]
    }

    fn is_branch(&self, insn: Inst) -> bool {
        self.insts[insn.index()].is_branch()
    }

    fn is_ret(&self, insn: Inst) -> bool {
        self.insts[insn.index()].is_ret()
    }

    fn multi_spillslot_named_by_last_slot(&self) -> bool {
        false
    }

    fn num_vregs(&self) -> usize {
        self.num_vregs
    }

    fn spillslot_size(&self, regclass: crate::RegClass) -> usize {
        match regclass {
            RegClass::Int => 2,
            RegClass::Float => 4,
            RegClass::Vector => 8,
        }
    }

    fn branch_blockparams(&self, _block: Block, _insn: Inst, _succ_idx: usize) -> &[VReg] {
        &[]
    }
}
