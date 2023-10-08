pub mod decode;

mod mov {
    #[derive(Debug)]
    pub enum Mov {
        RM,
        ImmToReg,
        ImmToRegOrMem,
        MemToAcc,
        AccToMem,
    }
}

mod add {
    #[derive(Debug)]
    pub enum Add {
        RM,
        ImmToRegOrMem,
        ImmToAcc,
    }
}
