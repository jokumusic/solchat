use anchor_lang::prelude::*;

declare_id!("FjQr3txvbowVHZud1Ec7QZTYGkmSihvLgyrBmF8FkM5");


const DIRECT_CONVERSATION_SEED_BYTES: &[u8] = b"direct_conversation";
const CONTACT_SEED_BYTES: &[u8] = b"contact";
const GROUP_SEED_BYTES: &[u8] = b"group";
const GROUP_CONTACT_SEED_BYTES: &[u8] = b"group_contact";
const MESSAGE_MAX_LEN: usize = 1024;

#[program]
pub mod solchat {
    use super::*;

    pub fn create_contact(ctx: Context<CreateContact>, name: String, data: String, receiver: Option<Pubkey>) -> Result<()> {
        
        if name.len() > CONTACT_NAME_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }
        
        let contact = &mut ctx.accounts.contact;
        contact.bump = *ctx.bumps.get("contact").unwrap();
        contact.creator = ctx.accounts.creator.key();        
        contact.receiver = receiver;
        contact.name = name;
        contact.data = data;

        Ok(())
    }

    pub fn update_contact(ctx: Context<UpdateContact>, name: String, data: String, receiver: Option<Pubkey>) -> Result<()> {

        if name.len() > CONTACT_NAME_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }
        
        let contact = &mut ctx.accounts.contact;   
        contact.receiver = receiver;
        contact.name = name;
        contact.data = data;

        Ok(())
    }


    pub fn start_direct_conversation(ctx: Context<StartDirectConversation>, message: String) -> Result<()> {

        if message.len() > MESSAGE_MAX_LEN {
            return Err(ErrorCode::MessageIsTooLong.into());
        }

        let conversation = &mut ctx.accounts.conversation;
        conversation.bump = *ctx.bumps.get("conversation").unwrap();
        conversation.contact1 = ctx.accounts.contact1.key();
        conversation.contact2 = ctx.accounts.contact2.key();
        conversation.messages_size = MESSAGE_MAX_LEN as u64;
        conversation.messages.push(message);

        Ok(())
    }

    pub fn send_direct_message(ctx: Context<SendDirectMessage>, message: String) -> Result<()> {
       
        if message.len() > MESSAGE_MAX_LEN {
            return Err(ErrorCode::MessageIsTooLong.into());
        }
        
        let conversation = &mut ctx.accounts.conversation;
        conversation.messages_size += message.len() as u64;
        conversation.messages.push(message);
        
        Ok(())
    }

    pub fn create_group(ctx: Context<CreateGroup>, nonce: u16, name: String, data: String) -> Result<()> {
        
        if name.len() > GROUP_NAME_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }
        
        let group = &mut ctx.accounts.group;
        group.bump = *ctx.bumps.get("group").unwrap();
        group.nonce = nonce;
        group.creator = ctx.accounts.signer.key();
        group.name = name;
        group.data = data;

        let signer_group_contact = &mut ctx.accounts.signer_group_contact;
        signer_group_contact.bump = *ctx.bumps.get("signer_group_contact").unwrap();
        signer_group_contact.group = group.key();
        signer_group_contact.contact = ctx.accounts.contact.key();
        signer_group_contact.group_contact_role = GroupContactRole::ADMIN;
        signer_group_contact.group_contact_preference = GroupContactPreference::SUBSCRIBE;

        Ok(())
    }

    pub fn edit_group(ctx: Context<EditGroup>, name: String, data: String) -> Result<()> {
        let signer_group_contact = &ctx.accounts.signer_group_contact;
        if signer_group_contact.group_contact_role != GroupContactRole::ADMIN {
            return Err(ErrorCode::NotAuthorized.into());
        }

        if name.len() > GROUP_NAME_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }

        let group = &mut ctx.accounts.group;
        group.name = name;
        group.data = data;

        Ok(())
    }

    pub fn create_group_contact(ctx: Context<CreateGroupContact>) -> Result<()> {
        let group_contact = &mut ctx.accounts.group_contact;
        group_contact.bump = *ctx.bumps.get("group_contact").unwrap();
        group_contact.group = ctx.accounts.group.key();
        group_contact.contact = ctx.accounts.contact.key();
        group_contact.group_contact_role = GroupContactRole::IGNORE;
        group_contact.group_contact_preference = GroupContactPreference::IGNORE;

        Ok(())
    }

    pub fn set_group_contact_role(ctx: Context<SetGroupContactRole>, role: u8) -> Result<()> {
        ctx.accounts.group_contact.group_contact_role = role;
        Ok(())
    }

    pub fn set_group_contact_preference(ctx: Context<SetGroupContactPreference>, preference: u8) -> Result<()> {
        ctx.accounts.signer_group_contact.group_contact_preference = preference;
        Ok(())
    }


}

#[derive(Accounts)]
#[instruction(name:String, data:String)]
pub struct CreateContact<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + CONTACT_SIZE + name.len() + data.len(),
        seeds = [CONTACT_SEED_BYTES, creator.key().as_ref()],
        bump
    )]
    pub contact: Account<'info, Contact>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
#[instruction(name:String, data:String)]
pub struct UpdateContact<'info> {
    #[account(
        mut,
        has_one = creator,        
        seeds = [CONTACT_SEED_BYTES, creator.key().as_ref()],
        bump = contact.bump,
        realloc = 8 + CONTACT_SIZE + data.len(),
        realloc::payer = creator,
        realloc::zero = false
    )]
    pub contact: Account<'info, Contact>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(message: String)]
pub struct StartDirectConversation<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + DIRECT_CONVERSATION_SIZE + MESSAGE_MAX_LEN, //anchor allocates the initial vec. check the number of vecs and multiply by 4 for each vec struct(3 usize) + 1 string pointer(1 usize)
        seeds = [DIRECT_CONVERSATION_SEED_BYTES, contact1.key().as_ref(), contact2.key().as_ref()],
        bump
    )]
    pub conversation: Account<'info, DirectConversation>,

    #[account(
        mut,
        constraint = (payer.key() == contact1.creator.key() || payer.key() == contact2.creator.key())
    )]
    pub payer: Signer<'info>,

    #[account()]
        //owner=id())]
    pub contact1: Account<'info, Contact>,

    #[account()]
    //(owner=id())]
    pub contact2: Account<'info, Contact>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(message: String)]
pub struct SendDirectMessage<'info> {

    #[account(mut, 
        has_one=contact1,
        has_one=contact2,
        realloc = 8 
            + DIRECT_CONVERSATION_SIZE
            + (conversation.messages.len() * 4) //number of items * 3(vec struct) * 1(string pointer)
            + (conversation.messages_size as usize) //number of characters. I think this is wrong because of usize miscalculation and is going to screw the pooch
            + (4 + message.len()), //current vec being added + number of characters in message
        realloc::payer = payer,
        realloc::zero = false,
        seeds = [DIRECT_CONVERSATION_SEED_BYTES, contact1.key().as_ref(), contact2.key().as_ref()],
        bump=conversation.bump
    )]
    pub conversation: Account<'info, DirectConversation>,

    pub contact1: Account<'info, Contact>,
    pub contact2: Account<'info, Contact>,
    
    #[account(
        mut,
        constraint = payer.key() == contact1.creator.key() || payer.key() == contact2.creator.key()
    )]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u16, name:String, data:String)]
pub struct CreateGroup<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, signer.key().as_ref()],
        bump = contact.bump,
        constraint = contact.creator == signer.key()
    )]
    pub contact: Account<'info, Contact>,

    #[account(
        init,
        payer = signer,
        space = 8 + GROUP_SIZE + data.len(),
        seeds = [GROUP_SEED_BYTES, signer.key().as_ref(), &nonce.to_be_bytes()],
        bump
    )]
    pub group: Account<'info, Group>,

    #[account(
        init,
        payer = signer,
        space = 8 + GROUP_CONTACT_SIZE,
        seeds = [GROUP_CONTACT_SEED_BYTES, group.key().as_ref(), contact.key().as_ref()],
        bump
    )]
    pub signer_group_contact: Account<'info, GroupContact>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(name: String, data: String)]
pub struct EditGroup<'info> {
    
    #[account(
        mut,
        realloc = 8 + GROUP_SIZE + data.len(),
        realloc::payer = signer,
        realloc::zero = false,
        seeds = [GROUP_SEED_BYTES, group.creator.as_ref(), &group.nonce.to_be_bytes()],
        bump = group.bump
    )]
    pub group: Account<'info, Group>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, signer.key().as_ref()],
        bump = signer_contact.bump,
        constraint = signer_contact.creator == signer.key()
    )]
    pub signer_contact: Account<'info, Contact>,

    #[account(
        seeds = [GROUP_CONTACT_SEED_BYTES, group.key().as_ref(), signer_contact.key().as_ref()],
        bump = signer_group_contact.bump,
    )]
    pub signer_group_contact: Account<'info, GroupContact>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CreateGroupContact<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, signer.key().as_ref()],
        bump = signer_contact.bump,
    )]
    pub signer_contact: Box<Account<'info, Contact>>,

    #[account(
        seeds = [GROUP_SEED_BYTES, group.creator.as_ref(), &group.nonce.to_be_bytes()],
        bump=group.bump
    )]
    pub group: Box<Account<'info, Group>>,

    #[account(
        seeds = [GROUP_CONTACT_SEED_BYTES, group.key().as_ref(), signer_group_contact.contact.as_ref()],
        bump= signer_group_contact.bump,
        constraint = signer_group_contact.group == group.key() && signer_group_contact.group_contact_role == GroupContactRole::ADMIN
    )]
    pub signer_group_contact: Box<Account<'info, GroupContact>>,


    #[account(
        init,
        payer = signer,
        space = 8 + GROUP_CONTACT_SIZE,
        seeds = [GROUP_CONTACT_SEED_BYTES, group.key().as_ref(), contact.key().as_ref()],
        bump
    )]
    pub group_contact: Account<'info, GroupContact>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, contact.creator.as_ref()],
        bump = contact.bump,
    )]
    pub contact: Box<Account<'info, Contact>>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct SetGroupContactRole<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, signer.key().as_ref()],
        bump = signer_contact.bump,
    )]
    pub signer_contact: Account<'info, Contact>,

    #[account(
        seeds = [GROUP_CONTACT_SEED_BYTES, signer_group_contact.group.as_ref(), signer_contact.key().as_ref()],
        bump= signer_group_contact.bump,
        constraint = signer_group_contact.group_contact_role == GroupContactRole::ADMIN,
    )]
    pub signer_group_contact: Account<'info, GroupContact>,

    #[account(
        mut,
        seeds = [GROUP_CONTACT_SEED_BYTES, group_contact.group.as_ref(), group_contact.contact.as_ref()],
        bump = group_contact.bump,
        constraint = group_contact.group == signer_group_contact.group
    )]
    pub group_contact: Account<'info, GroupContact>,
}

#[derive(Accounts)]
pub struct SetGroupContactPreference<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, signer.key().as_ref()],
        bump = signer_contact.bump,
    )]
    pub signer_contact: Account<'info, Contact>,

    #[account(
        mut,
        seeds = [GROUP_CONTACT_SEED_BYTES, signer_group_contact.group.as_ref(), signer_contact.key().as_ref()],
        bump= signer_group_contact.bump,
        constraint = signer_group_contact.contact == signer_contact.key()
    )]
    pub signer_group_contact: Account<'info, GroupContact>,
}




const CONTACT_NAME_LEN: usize = 100;
const CONTACT_SIZE: usize = 1 + 32 + (1+32) + (4+CONTACT_NAME_LEN) + 4;
#[account]
pub struct Contact 
{
    pub bump: u8, //1;
    pub creator: Pubkey, //32;
    pub receiver: Option<Pubkey>, //1+32;
    pub name: String, //4+100;
    pub data: String, //4+size;
}


const DIRECT_CONVERSATION_SIZE: usize = 1 + 32 + 32 + 8;
#[account]
pub struct DirectConversation {
    pub bump: u8, //1;
    pub contact1: Pubkey, //32;
    pub contact2: Pubkey, //32;
    pub messages_size: u64, //8;
    pub messages: Vec<String>, //vec is initialized by anchor. check size at creation time
}

const GROUP_NAME_LEN: usize = 100;
const GROUP_SIZE: usize = 1 + 2 + 32 + (4+GROUP_NAME_LEN) + 4;
#[account]
pub struct Group {
    pub bump: u8, //1;
    pub nonce: u16, //2;
    pub creator: Pubkey, //32;
    pub name: String, //4+100;
    pub data: String, //4+size;
}

const GROUP_CONTACT_SIZE: usize = 1 + 32 + 32 + 1 + 1;
#[account]
pub struct GroupContact {
    pub bump: u8, //1;
    pub group: Pubkey, //32;
    pub contact: Pubkey, //32;
    pub group_contact_role: u8, //1; GroupContactRole
    pub group_contact_preference: u8, //1; GroupContactPreference
}



#[error_code]
pub enum ErrorCode {
    #[msg("name is more than 100 characters")]
    NameIsTooLong,
    #[msg("message is more than 1024 characters")]
    MessageIsTooLong,
    #[msg("Not Authorized")]
    NotAuthorized,
}


struct GroupContactRole;
impl GroupContactRole {
    const IGNORE: u8 = 0;
    //const READ: u8 = 1;
    //const WRITE: u8 = 2;
    const ADMIN: u8 = 4;
}

struct GroupContactPreference;
impl GroupContactPreference {
    const IGNORE: u8 = 0;
    const SUBSCRIBE: u8 = 1;
}
