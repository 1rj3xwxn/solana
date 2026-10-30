use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the counter authority can update this counter")]
    Unauthorized,
    #[msg("Counter has reached the maximum value")]
    CounterOverflow,
    #[msg("金额不能为0")]
    ZeroAmount,
    #[msg("账户已经关闭")]
    AccountClosed,
}
