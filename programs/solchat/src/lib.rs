use anchor_lang::prelude::*;

declare_id!("AhqDVkiKVxijhJy3vU9hXFYjcwxaHAkyXsViMa4mEJc7");


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

    pub fn start_direct_conversation(ctx: Context<StartDirectConversation>, message: String) -> Result<()> {

        if message.len() > MESSAGE_MAX_LEN {
            return Err(ErrorCode::MessageIsTooLong.into());
        }

        let conversation = &mut ctx.accounts.conversation;
        conversation.bump = *ctx.bumps.get("conversation").unwrap();
        conversation.contact_a = ctx.accounts.from.key();
        conversation.contact_b = ctx.accounts.to.key();
        conversation.messages_size = message.len() as u64;
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
#[instruction(message: String)]
pub struct StartDirectConversation<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + DIRECT_CONVERSATION_SIZE + message.len(),
        seeds = [DIRECT_CONVERSATION_SEED_BYTES, from.key().as_ref(), to.key().as_ref()],
        bump
    )]
    pub conversation: Account<'info, DirectConversation>,

    #[account(mut,
    address = from.creator.key())]
    pub payer: Signer<'info>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, payer.key().as_ref()],
        bump = from.bump,
        constraint = from.creator.key() == payer.key()
    )]
    pub from: Account<'info, Contact>,

    #[account(
        seeds = [CONTACT_SEED_BYTES, to.creator.key().as_ref()],
        bump = to.bump
    )]
    pub to: Account<'info, Contact>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(message: String)]
pub struct SendDirectMessage<'info> {
    #[account(mut, 
        has_one=contact_a,
        has_one=contact_b,
        realloc = 8 + DIRECT_CONVERSATION_SIZE + (conversation.messages_size as usize) + message.len(),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub conversation: Account<'info, DirectConversation>,

    pub contact_a: Account<'info, Contact>,
    pub contact_b: Account<'info, Contact>,
    
    #[account(mut,
    constraint = payer.key() == contact_a.creator.key() || payer.key() == contact_b.creator.key())]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


const CONTACT_NAME_LEN: usize = 4+100;
const CONTACT_SIZE: usize = 1 + 32 + (1+32) + CONTACT_NAME_LEN + 4;
#[account]
pub struct Contact 
{
    pub bump: u8, //1;
    pub creator: Pubkey, //32;
    pub receiver: Option<Pubkey>, //1+32;
    pub name: String, //4+100;
    pub data: String, //4+size;
}


const DIRECT_CONVERSATION_SIZE: usize = 1 + 32 + 32 + 64 + 4;
#[account]
pub struct DirectConversation {
    pub bump: u8, //1;
    pub contact_a: Pubkey, //32;
    pub contact_b: Pubkey, //32;
    pub messages_size: u64, //8;
    pub messages: Vec<String>, //4+size;
}

#[error_code]
pub enum ErrorCode {
    #[msg("name is more than 100 characters")]
    NameIsTooLong,
    #[msg("message is more than 1024 characters")]
    MessageIsTooLong
}
