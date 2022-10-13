use anchor_lang::prelude::*;

declare_id!("ERRCx3eF4vpLDNcJERAM7MRVCE9NxJxDQs7qMDo6C7aj");

#[program]
pub mod dice_roll {
    use super::*;

    use anchor_lang::solana_program::{ program::invoke, system_instruction::transfer };
    pub fn setup(ctx: Context<Setup>, player: Pubkey, bet_amount: u64) -> Result<()> {
        let dice_account = &mut ctx.accounts.dice_account;

        dice_account.players = [ctx.accounts.user.key(), player];
        dice_account.bump = *ctx.bumps.get("dice_account").unwrap();
        dice_account.bet_amount = bet_amount;
        dice_account.game_state = String::from("");

        Ok(())
    }

    pub fn play(ctx: Context<Play>, player_choices: String) -> Result<()> {
        let dice_account = &mut ctx.accounts.dice_account;

        invoke(
            &transfer(
                ctx.accounts.user.to_account_info().key,
                dice_account.to_account_info().key,
                dice_account.bet_amount
            ),
            &[
                ctx.accounts.user.to_account_info(),
                dice_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ]
        )?;

        let mut num_vec: Vec<i32> = Vec::new();

        for item in player_choices.chars() {
            if item != ',' {
                num_vec.push((item as i32) - 0x30);
            }
        }
        let clock: Clock = Clock::get().unwrap();

        // take the current time 
        // take the mod 6 of it 
        // add 1 
        // we will get a number between 1 and 6
        let dice_result: i32 = ((clock.unix_timestamp % 6) + 1).try_into().unwrap();

        // win
        if num_vec.contains(&dice_result) {
            dice_account.game_state = format!("You won! Dice result: {}", dice_result);
            **dice_account.to_account_info().try_borrow_mut_lamports()? -= dice_account.bet_amount;
            **ctx.accounts.user.try_borrow_mut_lamports()? += 2 * dice_account.bet_amount;
        } else {
            dice_account.game_state = format!("You lose! Dice result: {}", dice_result);
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Setup<'info> {
    #[account(
        init,
        payer = user,
        space = DiceRoll::LEN,
        seeds = ["dice_account".as_bytes(), user.key().as_ref()],
        bump
    )]
    pub dice_account: Account<'info, DiceRoll>,
    #[account(mut)]
    pub user: Signer<'info>,
    //pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(
        mut, 
        seeds = ["dice_account".as_bytes(), user.key().as_ref()],
        bump = dice_account.bump
    )]
    pub dice_account: Account<'info, DiceRoll>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct DiceRoll {
    players: [Pubkey; 2],
    bet_amount: u64,
    game_state: String,
    bump: u8,
}

impl DiceRoll {
    const LEN: usize = 200;
}
