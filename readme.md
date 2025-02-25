# Anchor Token Swap Program

## Overview

This `token-swap` project is rewritten using [Anchor](https://www.anchor-lang.com/docs), forked from [solana-program-library/token-swap](https://github.com/solana-labs/solana-program-library/tree/master/token-swap). 

## Advantages of using anchor

1. **Unpack & Pack**: 

    Using macros like `#[account]` and `#[derive(AnchorSerialize, AnchorDeserialize)]` can simply implement the `unpack` and `pack` methods, which can reduce chunky of boliplate code.

2. **Configuable Constraints**: 

   The configurable validations are a great enhancement compared to the original code using lots of comparasion. Below is an example:  
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

  Javascript is the default client language, unfortunately, it's power is not well utilized in some partss the original code. Below are the notable changes (tracked here: [**#2**](https://github.com/yanCode/anchor-token-swap/issues/2)):
  1. **Misuse of async javascript**  
      - It's a rather common misconception that main typescript client executes the transaction one after another (*sequentially*). In fact, that can be perfectly submitted in batches. You know the reason why solana is faster is because it supports parallel processing. 

      - `sleep` shouldn't be used, as javascript is event-driven, there is always an async way to be informed when a transaction is done, using `sleep` to wait for a fixed-period of time can make the execution much ineffective, because the actual time of transaction execution can be vary for different traffic.

2. **Fix a bug on`account validation`** 

    I detected an business logic issue, details are here: [**#3**](https://github.com/yanCode/anchor-token-swap/issues/3)

3. **Pre-commit hooks** 

  This project uses `pre-commit` to format the code before each commits [**#4**](https://github.com/yanCode/anchor-token-swap/issues/4).

## How to run
- build: `anchor build`
- unit test & proptest: `cargo test`
- integration test: `anchor test` (make sure `yarn install` has been executed to ensure all javascript/typescript dependencies are installed)
- run two above tests in one go: `yarn test`

## Todo
- [ ] Add tests on curves including `constant_price` and `offset`
- [ ] Add tests on both `spl-token` and `spl-token-2022`, all the mints can use each of the token programs.
- [ ] Add tests related to `bpf-upgradeable`

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
