#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpCode {
    /// Return to caller, cleanup GC. \
    /// Usage: `exit`
    Exit,
    /// Move contents of `rY` into `rX`. \
    /// Usage: `rX <- reg rY`
    RegisterMove,
    /// Jump over functon, define label `label.a`. \
    /// Usage:
    /// ```plain
    /// func label.a
    ///     ...
    /// end
    /// ```
    Func,
    /// Jump to `label.a`. \
    /// Usage: `jump label.a`
    LabelJump,
    /// Jump to `label.a`. \
    /// Argument in `rA` is moved to `r1`, `rB` to `r2`, `rC` to `r3`, and so on. \
    /// Once the function is done, all registers are restored. The return value is put into `rX`. \
    /// Usage: `rX <- call label.a rA? rB? rC...`
    LabelCall,
    /// Store address of `label.a` in `rX`. \
    /// Usage: `rX <- addr label.a`
    LabelAddress,
    /// Jump to address stored in `rX`. Usually this is obtained from [LabelAddress](OpCode::LabelAddress). \
    /// Usage: `djump rX`
    DynamicJump,
    /// Jump to the address stored in `rX`. Usually this is obtained from [LabelAddress](OpCode::LabelAddress). \
    /// Argument in `rA` is moved to `r1`, `rB` to `r2`, `rC` to `r3`, and so on. \
    /// Once the function is done, all registers are restored. The return value is put into `rX`. \
    /// Usage: `rX <- dcall rY rA? rB? rC?...`
    DynamicCall,
    /// Store the value stored in `rY` in the `rX` from [LabelCall](OpCode::LabelCall) or [DynamicCall](OpCode::DynamicCall). \
    /// Usage: `ret rY`
    Return,
    /// Store `N` in `rX`. \
    /// Usage: `rX <- int N`
    Integer,
    /// Store result of operation `-rY` into `rX`. \
    /// Usage: `rX <- neg rY`
    Neg,
    /// Store result of operation `rY + rZ` into `rX`. \
    /// Usage: `rX <- add rY rZ`
    Add,
    /// Store result of operation `rY - rZ` into `rX`. \
    /// Usage: `rX <- sub rY rZ`
    Sub,
    /// Store result of operation `rY * rZ` into `rX`. \
    /// Usage: `rX <- mul rY rZ`
    Mul,
    /// Store result of operation `rY / rZ` into `rX`. \
    /// Usage: `rX <- div rY rZ`
    Div,
    /// Store result of operation `rY % rZ` into `rX`. \
    /// Usage: `rX <- mod rY rZ`
    Mod,
    /// Jump to `label.a` if the contents of `rX` is zero, otherwise jump to `label.b`. \
    /// Usage: `bb rX label.a label.b`
    BranchBoolean,
    /// Jump to `label.t` if the contents of `rX` is equal to the contents of `rY`, otherwise jump to `label.f`. \
    /// Usage: `beq rX rY label.f label.t`
    BranchEqual,
    /// Jump to `label.t` if the contents of `rX` is less than the contents of `rY`, otherwise jump to `label.f`. \
    /// Usage: `blt rX rY label.f label.t`
    BranchLessThan,
    /// Store an array with the ascii data representing `"text-1"` into `rX`. \
    /// Usage: `rX <- str :text-1`
    String,
    /// Store an empty array of length `rY` into `rX`. \
    /// Usage: `rX <- arr rY`
    Array,
    /// Store `rZ` into `rX` at index `rY`. \
    /// Usage: `set rX rY rZ`
    SetArrayIndex,
    /// Store into `rX` the element at index `rZ` of `rY`. \
    /// Usage: `rX <- get rY rZ`
    GetArrayIndex,
    /// Store into `rX` the length of the array in `rY`. \
    /// Usage: `rX <- len rY`
    ArrayLength,
    /// Store `0` into `rX` if the data in `rY` is an integer. \
    /// Store `1` into `rX` if the data in `rY` is an array. \
    /// Usage: `rX <- type rY`
    ObjectType,
    /// Print the character stored in `rX` to stdout. \
    /// Usage: `putchar rX`
    PutChar,
}