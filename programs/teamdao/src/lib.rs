use anchor_lang::prelude::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod teamdao {
    use super::*;
    // create the team
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
    // join a team
    pub fn join_team(ctx: Context<JoinTeam>, name: String, _id: u64, new_member: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        team.captain = ctx.accounts.signer.key();
        team.name = name;
        team.members.push(new_member);
        msg!("player {} is successfully joined to the team {}", new_member , team.name );
        Ok(())
    }
    // leave from a team
    pub fn leave_team(ctx: Context<LeaveTeam>, _name: String, _id: u64, leaving_member: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        require!(team.members.contains(&leaving_member),Err::MemberNotFound);
                
        team.members.retain(|&member| member != leaving_member);
        msg!("player {} is successfully leaved from the team {}", leaving_member , team.name );
        
        Ok(())
    }
    // change the captain of the team
    pub fn change_captain(ctx: Context<ChangeCaptain>, _name: String, _id: u64, new_captain: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        team.captain = *ctx.accounts.signer.key;
        require!(team.members.contains(&new_captain),Err::MemberNotFound);
        
            team.captain = new_captain;
            msg!("player {} is now the captain of the team {}", new_captain , team.name );
        
        Ok(())
    }
    // remove a member of the team
    pub fn remove_from_team(ctx: Context<RemoveFromTeam>, _name: String, _id: u64) -> Result<()> {
        let team: &mut Account<Team> = &mut ctx.accounts.team;
        let signer = *ctx.accounts.signer.key;
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

    // join a tournament
    pub fn join_tournament(ctx: Context<JoinTournament>, _team: String, _id: u64) -> Result<()> {
        let team = &mut ctx.accounts.team;

        require!(team.members.len() == 5,Err::NotEligibleToJoinTournament);

        if team.vote_result && team.dist_yes > 2 {
            team.join_tournament = true;
        } else {
            team.join_tournament = false;
        }
        Ok(())
    }

    // leave from a tournament
    pub fn leave_from_tournament(ctx: Context<LeaveFromTournament>, _team: String, _id: u64, vote: Vote) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.active_tournament != Pubkey::default() && team.members.contains(ctx.accounts.signer.key) && !team.voted.contains(ctx.accounts.signer.key) {
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
            return err!(Err::NotEligibleToLeaveVoting)
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

    // vote in a tournament
    pub fn vote_for_tournament(ctx: Context<VoteForTournament>, _team: String, _id: u64, address: Pubkey, vote: Vote) -> Result<()> {
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
            return err!(Err::NotEligibleToVote)
        }

        if team.yes > 2 {
            team.active_tournament = address;
            team.yes = 0;
            team.voted = vec![];
            team.result = true;
        }

        msg!("{} is voted for the tournament", team.name);
        Ok(())
    }

    // -------------------- tournament proposal section --------------------------

    // distribution of rewards proposal
    pub fn set_proposal(ctx: Context<SetProposal>, _team: String, _id: u64, per: Vec<u8>) -> Result<()> {
        let team = &mut ctx.accounts.team;
        let total: u8 = per.iter().sum();
        if total == 100 && team.active_tournament != Pubkey::default() && team.captain == *ctx.accounts.signer.key {
            team.distribution = per;
        } else {
            return err!(Err::CannotProvideProposalConditions)
        }

        msg!("{} is successfully proposed the percentage of {:?}", team.name, team.distribution);
        Ok(())
    }

    // distribution of the rewards
    pub fn reward_distribution(ctx: Context<RewardDistribution>, _team: String, _id: u64, vote: Vote) -> Result<()> {
        let team = &mut ctx.accounts.team;

        if team.active_tournament != Pubkey::default() && team.members.contains(ctx.accounts.signer.key) && !team.voted.contains(ctx.accounts.signer.key) {
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
            } else {
                return err!(Err::CannotProvideRewardDistributionConditions)
            }
        
        } 
        Ok(())
    }

    // claim the rewards
    pub fn set_rewards(ctx: Context<SetRewards>, _team: String, _id: u64, reward: u64) -> Result<()> {
        let team = &mut ctx.accounts.team;
        if team.members.contains(ctx.accounts.to.key) {
            let from = ctx.accounts.from.to_account_info();
            let to = ctx.accounts.to.to_account_info();
            **from.try_borrow_mut_lamports()? -= reward;
            **to.try_borrow_mut_lamports()? += reward;
        } else {
            return err!(Err::MemberNotFoundInTeam)
        }
        Ok(())
    }

}

    // ---------------- instructions -----------------

// derive macro for creating the team 
#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = Team::LEN, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// derive macro for joinging team
#[derive(Accounts)]
#[instruction(name: String, id: u64)]
pub struct JoinTeam<'info> {
    #[account(mut, seeds=[name.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// derive macro for leaving the team
#[derive(Accounts)]
#[instruction(name: String, id: u64)]
pub struct LeaveTeam<'info> {
    #[account(mut, seeds=[name.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// derive macro for changing the captain
#[derive(Accounts)]
#[instruction(name: String, id: u64)]
pub struct ChangeCaptain<'info> {
    #[account(mut, seeds=[name.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// derive macro for removing from team
#[derive(Accounts)]
#[instruction(name: String, id: u64)]
pub struct RemoveFromTeam<'info> {
    #[account(mut, seeds=[name.as_bytes(), &id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

    // ----------------- tournament instructions ------------------

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct JoinTournament<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct VoteForTournament<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct LeaveFromTournament<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

    // ----------------- reward proposal instructions --------------

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct SetProposal<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct RewardDistribution<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_team: String, _id: u64)]
pub struct SetRewards<'info> {
    #[account(mut, seeds=[_team.as_bytes(), &_id.to_ne_bytes()], bump = team.bump)]
    pub team: Account<'info, Team>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub from: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub to: AccountInfo<'info>,
    #[account()]

    pub user: Signer<'info>,
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
    pub yes: u8,
    pub no: u8,
    pub leave_yes_vote: u8,
    pub leave_no_vote: u8,
    pub leave_voted_members: Vec<Pubkey>,
    pub result: bool,
    pub distribution: Vec<u8>,
    pub dist_yes: u8,
    pub dist_of_voted: Vec<Pubkey>,
    pub dist_result: bool,
}

impl Team {
    // bump values are in the same order of Team struct except the first one is discriminator
    const LEN: usize = 16+32+8+32+5*32+1+1+32+5*32+1+1+1+1+5*32+1+1*5+1+5*32+1;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Vote {
    Yes,
    No,
}

#[error_code]
pub enum Err {
    InvalidBump,
    MemberNotFound,
    NotEligibleToJoinTournament,
    NotEligibleToLeaveVoting,
    NotEligibleToVote,
    CannotProvideProposalConditions,
    CannotProvideRewardDistributionConditions,
    MemberNotFoundInTeam,
}
