use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod teamdao {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, player_number:u8) -> Result<()> {
        let team = &mut Account<Team> = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.players = 0;
        team.prize_owners = player_number

        msg!("team {} is successfully created, captain is {}", team.name,team.captain );
        Ok(())
    }

    pub fn change_team(ctx: Context<ChangeTeam>, name: String) -> Result<()> {
        let team = &mut Account<Team> = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        msg!("team {} is successfully changed, the new captain is {}", team.name,team.captain );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = signer,space = 8 + 8 + 2 + 32 + 1 + 64 )]
    pub team: Account<'info, Team>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}



pub struct Team {
    pub players: u8,
    pub captain: Pubkey,
    pub prize_owners: u8,
    pub prize_owners_id: Vec<u64>,
    pub prize_owners_vote: Vec<u64>,
}
