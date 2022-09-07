use anchor_lang::prelude::*;

declare_id!("FjQr3txvbowVHZud1Ec7QZTYGkmSihvLgyrBmF8FkM5");


const DIRECT_CONVERSATION_SEED_BYTES: &[u8] = b"direct_conversation";
const CONTACT_SEED_BYTES: &[u8] = b"contact";
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
        bump,
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
        //constraint = (payer.key() == contact1.creator.key() || payer.key() == contact2.creator.key())
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
        realloc::zero = false
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

#[error_code]
pub enum ErrorCode {
    #[msg("name is more than 100 characters")]
    NameIsTooLong,
    #[msg("message is more than 1024 characters")]
    MessageIsTooLong,
    #[msg("test error")]
    TestError,
}
