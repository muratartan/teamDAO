use anchor_lang::prelude::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod teamdao {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, id:u64) -> Result<()> {
        let team = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.id = id;
        team.members.push(*ctx.accounts.signer.key);
        team.bump = *ctx.bumps.get("team").ok_or(Err::InvalidBump)?;
        
        msg!("team {} is successfully created, captain is {}", team.name,team.captain );
        Ok(())
    }  

    pub fn join_team(ctx: Context<JoinTeam>, name: String, _id: u64, new_member: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.members.push(new_member);
        msg!("player {} is successfully joined to the team {}", new_member , team.name );
        Ok(())
    }

    pub fn leave_team(ctx: Context<LeaveTeam>, name: String, _id: u64, leaving_member: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        if team.members.contains(&leaving_member) {
            team.members.retain(|&member| member != leaving_member);
            msg!("player {} is successfully leaved from the team {}", leaving_member , team.name );
        } else {
            Err::MemberNotFound
        };
        Ok(())
    }

    fn change_captain(ctx: Context<ChangeCaptain>, team: String, _id: u64, new_captain: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        team.captain = *ctx.accounts.signer.key;
        if team.members.contains(&new_captain) {
            team.captain = new_captain;
            msg!("player {} is now the captain of the team {}", new_captain , team.name );
        } else {
            Err::MemberNotFound
        };
        Ok(())
    }

    fn remove_from_team(ctx: Context<RemoveFromTeam>, id: u64) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        let signer == *ctx.accounts.signer.key;
        if team.members.len() == 1 {
            team.captain = Pubkey::default();
            team.members = vec![];
            team.id = 0;
        } else {
            team.members.retain(|&i| i != signer);
        }
        msg!("{} is successfully removed from the team {}", signer,team.name);
        Ok(())
    }

    //----------------------- tournament section -------------------------------

    pub fn join_tournament(ctx: Context<JoinTournament>, _team: String, _id: u64) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.members.len() < 5 || team.tournament == Pubkey::default {
            Err::NotEligibleToJoinTournament
        }

        if team.voting_result && team.dist_yes > 2 {
            team.join_tournament = true;
        } else {
            team.join_tournament = false;
        }
        Ok(())
    }

    pub fn leave_from_tournament(ctx: Context<LeaveTournament>, team: String, id: u64, vote: Vote) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.active_tournament != Pubkey::default() && team.members.contains(ctx.accounts.signer.key) && !team.voted_players.contains(ctx.accounts.signer.key) {
            match vote {
                Vote::Yes => {
                    team.leave_voted_members.push(*ctx.accounts.signer.key);
                    team.leave_yes_vote += 1;
                }
                Vote::No => {
                    team.voted.push(*ctx.accounts.signer.key);
                    team.leave_no_vote += 1;
                }
           } 
        } else {
            Err::NotEligibleToLeaveVoting
        }

        if team.leave_yes_vote > 2 {
            team.active_tournament = Pubkey::default();
            team.leave_yes_vote = 0;
            team.leave_no_vote = 0;
            team.leave_voted_members = vec![];
            team.voted = vec![];
        }

        msg!("{} is successfully leaved from the tournament", team.name);
        Ok(())

    }

    pub fn vote_for_tournament(ctx: Context<Vote>, team: String, id: u64, address: Pubkey, vote: Vote) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.active_tournament == Pubkey::default() && team.members.contains(ctx.accounts.signer.key) && !team.voted.contains(ctx.accounts.signer.key) {
           match vote {
                Vote::Yes => {
                    team.voted.push(*ctx.accounts.signer.key);
                    team.yes += 1;
                }
                Vote::No => {
                    team.voted.push(*ctx.accounts.signer.key);
                    team.no += 1;
                }
           } 
        } else {
            Err::NotEligibleToVote
        }

        if team.yes > 2 {
            team.active_tournament = tournament_add;
            team.yes = 0;
            team.voted = vec![];
            team.result = true;
        }

        msg!("{} is voted for the tournament", team.name);
        Ok(())
    }

    // -------------------- tournament proposal section --------------------------

    pub fn set_proposal(ctx: Context<SetProposal>, _team: String, _id: u64, per: Vec<u8>) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if per.iter().sum == 100 && team.active_tournament != Pubkey::default() && team.captain == *ctx.accounts.signer.key {
            team.distribution = per;
        } else {
            Err::CannotProvideProposalConditions
        }

        msg!("{} is successfully proposed the percentage of {:?}", team.name, team.distribution);
        Ok(())
    }

    pub fn reward_distribution(ctx: Context<RewardDistribution>, team: String, id: u64, vote: Vote) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.active_tournament != Pubkey::default() && team.members.contains(ctx.accounts.signer.key) && !team.voted_players.contains(ctx.accounts.signer.key) {
            match vote {
                Vote::Yes => {
                    team.dist_of_voted.push(*ctx.accounts.signer.key);
                    team.dist_yes += 1;
                }
                Vote::No => {
                    team.voted.push(*ctx.accounts.signer.key);
                }
            }

            if team.dist_of_voted.len() > 2 && team.dist_yes > 2 {
                team.dist_result = true;
            }
            if team.dist_of_voted.len() > 2 && team.dist_yes < 3 {
                team.dist_result = false;
            }
        } else {
            Err::CannotProvideRewardDistributionConditions
        }
    }

    pub fn set_rewards(ctx: Context<SetRewards>, _team: String, _id: u64, reward: u64) -> Result<()> {
        let team = &mut ctx.accounts.team;
        if team.members.contains(ctx.accounts.to.key) {
            let from = ctx.accounts.from.to_account_info();
            let to = ctx.accounts.to.to_account_info();
            **from.try_borrow_mut_lamports()? -= reward;
            **to.try_borrow_mut_lamports()? += reward;
        } else {
            Err::MemberNotFoundInTeam
        }
        Ok(())
    }

}

    // ---------------- instructions -----------------

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = Team::LEN, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(team: String, id: u64)]
pub struct JoinTeam<'info> {
    #[account(mut, seeds=[team.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(team: String, id: u64)]
pub struct LeaveTeam<'info> {
    #[account(mut, seeds=[team.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(team: String, id: u64)]
pub struct TransferCaptain<'info> {
    #[account(mut, seeds=[team.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(team: String, id: u64)]
pub struct RemoveFromTeam<'info> {
    #[account(mut, seeds=[team.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}




    // ---------------- base structs ---------------

#[account]
pub struct Team {
    pub name: String,
    pub id: u64,
    pub captain: Pubkey,
    pub members: Vec<Pubkey>,
    pub bump: u8, 
    pub join_tournament: bool,
    pub active_tournament: Pubkey,
    pub voted: Vec<Pubkey>,
    pub vote_result: bool,
    pub yes: u8
    pub no: u8,
    pub leave_yes_vote: u8,
    pub leave_no_vote: u8,
    pub leave_voted_members: Vec<Pubkey>,
    pub result: bool,
    pub distribution: Vec<u8>
    pub dist_yes: u8,
    pub dist_of_voted: Vec<Pubkey>,
    pub dist_result: bool,
}

impl Team {
    const LEN: usize = 16+32+8+32+5*32+1+1+32+5*32+1+1+1+1+5*32+1+1*5+1+5*32+1;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Vote {
    Yes,
    No,
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


