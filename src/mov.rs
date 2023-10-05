#[derive(Debug)]
pub enum Mov {
    RM,
    ImmToReg,
    ImmToMem,
    MemToAcc,
    AccToMem,
}
