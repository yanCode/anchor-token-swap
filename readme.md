# Token Lending Program

## Overview

This project is an anchor version of token-swap, forked from [solana-program-library/token-swap](https://github.com/solana-labs/solana-program-library/tree/master/token-swap). 

## Advantages of using anchor

1. **Unpack & Pack**: 

    Using macros like `#[account]` and `#[derive(AnchorSerialize, AnchorDeserialize)]` can simply implement the `unpack` and `pack` methods, which can reduce chunky of boliplate code.

2. **Configuable Constraints**: 
   ```rust
    #[account(
        token::token_program = token_swap.token_program_id,
        token::mint = pool_mint.key(),
        constraint = host_fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub host_fee_account: Option<InterfaceAccount<'info, TokenAccount>>, 
   ```
3. **Client friendly**:
It automatically generates client types & even methods for typescript client.
4. **Other features**:
   - `init` can combine create the account via system CPI and initialize the account in one step.
   - `#[derive(InitSpace)]` can measure the initial size of the account.
   - `seeds` Configurable PDA support
   - `CpiContext` Strongly typed for CPI calling.

## Noticeable Changes

  Comparing with the original code, below changes are made:
    Tracked issues are here: [**#2**](issues/2)
  1. **Misuse of async javascript**  
      - It's a rather common misconception that main typescript client executes the transaction one after another is completed. In fact, that can be perfectly submitted in batches. After all, solana is faster is because of it supports parallel processing. 

      - `sleep` shouldn't be used, as javascript is event-driven, there is always an asyc way to know certain transaction is done, using `sleep` to wait for a fixed-period of time can make the execution much ineffective.
2. **Fix a bug on`account validation`** 

    Details is here: [**#3**](issues/3)

3. **Pre-commit hooks** 

  This project uses `pre-commit` to format the code before each commits [**#4**](issues/4).


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
