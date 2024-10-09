pub const IPUSH64: u8 = 0; // IPUSH [value]
pub const IPUSH8: u8 = IPUSH64 + 1; // IPUSH [value]
pub const SPUSH: u8 = IPUSH8 + 1; // SPUSH #[value]
pub const BPUSH: u8 = SPUSH + 1; // BPUSH [length], [bytes...]
pub const DROPN: u8 = BPUSH + 1; // DROP [value] 
pub const CHG: u8 = DROPN + 1; // CHG #[value], #[value]
pub const SWAP: u8 = CHG + 1; // ###

pub const BHASH: u8 = SWAP + 1; // ###
pub const BLEN: u8 = BHASH + 1; // ###

pub const MKSLICE: u8 = BLEN + 1; // MKSLICE
pub const IREAD64: u8 = MKSLICE + 1; // U64READ
pub const IREAD8: u8 = IREAD64 + 1; // U8READ
pub const BREAD: u8 = IREAD8 + 1;
pub const SLLEN: u8 = BREAD + 1;

pub const MKBUILDER: u8 = SLLEN + 1; // MKBUILDER
pub const IWRITE64: u8 = MKBUILDER + 1; // U64WRITE
pub const IWRITE8: u8 = IWRITE64 + 1; // U8WRITE
pub const BWRITE: u8 = IWRITE8 + 1; //
pub const BUILD: u8 = BWRITE + 1; // BUILD
pub const BLLEN: u8 = BUILD + 1; // BUILD

pub const ADD: u8 = BLLEN + 1; // ADD
pub const SUB: u8 = ADD + 1; // SUB
pub const MUL: u8 = SUB + 1; // MUL
pub const DIV: u8 = MUL + 1; // DIV
pub const MOD: u8 = DIV + 1; // MOD
pub const INC: u8 = MOD + 1; // MOD ###

pub const CMB: u8 = INC + 1; // >
pub const CML: u8 = CMB + 1; // <
pub const CMBE: u8 = CML + 1; // >=
pub const CMLE: u8 = CMBE + 1; // <=
pub const CME: u8 = CMLE + 1; // ==
pub const CMNE: u8 = CME + 1; // !=

pub const JMP: u8 = CMNE + 1; // JMP &[ip]
pub const JMT: u8 = JMP + 1; // JMT &[ip]
pub const JMF: u8 = JMT + 1; // JMF &[ip]
pub const RJMP: u8 = JMF + 1; // Relative JMP &[ip]
pub const RJMT: u8 = RJMP + 1; // Relative JMT &[ip]
pub const RJMF: u8 = RJMT + 1; // Relative JMF &[ip]
pub const CALL: u8 = RJMF + 1; // CALL &[ip]
pub const RET: u8 = CALL + 1;

pub const HALT: u8 = RET + 1; // HALT

pub const LDATA: u8 = HALT + 1;
pub const SDATA: u8 = LDATA + 1;
pub const MESSAGE: u8 = SDATA + 1;
pub const SEND: u8 = MESSAGE + 1;
