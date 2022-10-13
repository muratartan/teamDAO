use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod teamdao {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, id:u64) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.id = id;
        
        msg!("team {} is successfully created, captain is {}", team.name,team.captain );
        Ok(())
    }

    pub fn join_team(ctx: Context<JoinTeam>, name: String, id: u64, new_member: Pubkey) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.members.push(new_member);
        msg!("player {} is successfully joined to the team {}", new_member , team.name );
        Ok(())
    }

    pub fn leave_team(ctx: Context<LeaveTeam>, name: String, id: u64, leaving_member: Pubkey) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        if team.members.contains(&leaving_member) {
            team.members.retain(|&member| member != leaving_member);
            msg!("player {} is successfully leaved from the team {}", leaving_member , team.name );
        } else {
            Err::MemberNotFound
        };
        Ok(())
    }

    fn change_captain(ctx: Context<ChangeCaptain>, team: String, id: u64, new_captain: Pubkey) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        team.captain = *ctx.accounts.signer.key;
        if team.members.contains(&new_captain) {
            team.captain = new_captain;
            msg!("player {} is now the captain of the team {}", new_captain , team.name );
        } else {
            Err::MemberNotFound
        };
        Ok(())
    }

    fn remove_from_team(ctx: Context<RemoveFromTeam>, team: String, id: u64) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        let signer == *ctx.accounts.signer.key;
        if team.members.len() == 1 {
            Err::TeamNumberError
        } else {
            team.members.retain(|&i| i != signer);
        }
        msg!("{} is successfully removed from the team {}", signer,team.name);
        Ok(())
    }

    //----------------------- tournament section -------------------------------

    pub fn join_tournament(ctx: Context<JoinTournament>, team: String, id: u64) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.members.len() < 5 || team.tournament != Pubkey::default {
            Err::NotEligibleToJoinTournament
        }

        if team.vote_result && team.yes_votes > 2 {
            team.join_tournament = true;
        } else {
            teamjoin_tournament = false;
        }
        Ok(())
    }

}











#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = 8 + 8 + 2 + 32 + 1 + 64 )]
    pub team: Account<'info, Team>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

pub struct ChangeTeam<'info> {
    #[account(init, payer = signer, space = space = 8 + 8 + 2 + 32 + 1 + 64)]
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
