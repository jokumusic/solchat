import * as anchor from "@project-serum/anchor";
import { Program, translateAddress } from "@project-serum/anchor";
import { Solchat } from "../target/types/solchat";
import {PublicKey, Keypair, sendAndConfirmTransaction } from "@solana/web3.js";
import { expect } from "chai";

describe("solchat", () => {
  const contactAKeypair = Keypair.generate();
  const contactBKeypair = Keypair.generate();
  const conversationStartMessage = "Let's chat!";

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  console.log('contact A pubkey: ', contactAKeypair.publicKey.toBase58());
  console.log('contact B pubkey: ', contactBKeypair.publicKey.toBase58());

  const program = anchor.workspace.Solchat as Program<Solchat>;

  let [contactAPda, contactAPdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("contact"), 
      contactAKeypair.publicKey.toBuffer(),             
    ], program.programId);
  let [contactBPda, contactBPdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("contact"), 
      contactBKeypair.publicKey.toBuffer(),             
    ], program.programId);
  let [directConversationPda, directConversationPdaBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("direct_conversation"), 
        contactAPda.toBuffer(),
        contactBPda.toBuffer(),             
      ], program.programId);


  before((done) => {
    console.log('funding contact accounts');
    const airDropA = provider.connection
      .requestAirdrop(contactAKeypair.publicKey, 20_000_000)
      .catch(err=>done(err));

    const airDropB =  provider.connection
      .requestAirdrop(contactBKeypair.publicKey, 20_000_000)
      .catch(err=>done(err));

    Promise.all([airDropA, airDropB])
      .then(async (signatures)=>{
        const responseA = await provider.connection.confirmTransaction(signatures[0],'confirmed');
        const responseB = await provider.connection.confirmTransaction(signatures[1],'confirmed');
        done();
      })
      .catch(err=>done(err));
  });


  it("Create Contacts", async () => {
    const contactAName = "Contact A";
    const contactBName = "Contact B";    
    const data = JSON.stringify({prop1:1, prop2:""});

    const txA = await program.methods
      .createContact(contactAName, data, contactAKeypair.publicKey)
      .accounts({
        creator: contactAKeypair.publicKey,
        contact: contactAPda,
      })
      .transaction();
    
    const responseA = await anchor.web3.sendAndConfirmTransaction(provider.connection, txA, [contactAKeypair]);
    const contactA = await program.account.contact.fetch(contactAPda);
    expect(contactA.bump).is.equal(contactAPdaBump);
    expect(contactA.creator).is.eql(contactAKeypair.publicKey);
    expect(contactA.receiver).is.eql(contactAKeypair.publicKey);
    expect(contactA.name).is.equal(contactAName);
    expect(contactA.data).is.equal(data);


    const txB = await program.methods
      .createContact(contactBName, data, contactBKeypair.publicKey)
      .accounts({
        creator: contactBKeypair.publicKey,
        contact: contactBPda,
      })
      .transaction();

    
    const responseB = await anchor.web3.sendAndConfirmTransaction(provider.connection, txB, [contactBKeypair]);
    const contactB = await program.account.contact.fetch(contactBPda);
    expect(contactB.bump).is.equal(contactBPdaBump);
    expect(contactB.creator).is.eql(contactBKeypair.publicKey);
    expect(contactB.receiver).is.eql(contactBKeypair.publicKey);
    expect(contactB.name).is.equal(contactBName);
    expect(contactB.data).is.equal(data);
  });


  it("Update Contact", async () => {
    const contactAName = "Contact A - updated";  
    const data = JSON.stringify({prop1:2, prop2:"updated"});

    const txA = await program.methods
      .updateContact(contactAName, data, contactAKeypair.publicKey)
      .accounts({
        creator: contactAKeypair.publicKey,
        contact: contactAPda,
      })
      .transaction();
    
    const responseA = await anchor.web3.sendAndConfirmTransaction(provider.connection, txA, [contactAKeypair]);
    const contactA = await program.account.contact.fetch(contactAPda);
    expect(contactA.bump).is.equal(contactAPdaBump);
    expect(contactA.creator).is.eql(contactAKeypair.publicKey);
    expect(contactA.receiver).is.eql(contactAKeypair.publicKey);
    expect(contactA.name).is.equal(contactAName);
    expect(contactA.data).is.equal(data);
  });

  
  it("Start Direct Conversation", async () => {
    const txA = await program.methods
      .startDirectConversation(conversationStartMessage)
      .accounts({
        conversation: directConversationPda,
        payer: contactAKeypair.publicKey,
        from: contactAPda,
        to: contactBPda,
      })
      .transaction();
    
    const responseA = await anchor.web3.sendAndConfirmTransaction(provider.connection, txA, [contactAKeypair]);
    const conversation = await program.account.directConversation.fetch(directConversationPda);
    expect(conversation.bump).is.equal(directConversationPdaBump);
    expect(conversation.contactA).is.eql(contactAPda);
    expect(conversation.contactB).is.eql(contactBPda);
    expect(conversation.messages[0]).is.equal(conversationStartMessage);
  });


  it("Send Direct Messages", async () => {
    const contactAMessage = "I'm contact A!";
    const contactBMessage  = "I'm contact B!";    

    const txA = await program.methods
      .sendDirectMessage(contactAMessage)
      .accounts({
        conversation: directConversationPda,
        payer: contactAKeypair.publicKey,
        contactA: contactAPda,
        contactB: contactBPda,
      })
      .transaction();
    
    const responseA = await anchor.web3.sendAndConfirmTransaction(provider.connection, txA, [contactAKeypair]);
    const conversationA = await program.account.directConversation.fetch(directConversationPda);
    expect(conversationA.messages[1]).is.equal(contactAMessage);

    const txB = await program.methods
    .sendDirectMessage(contactBMessage)
    .accounts({
      conversation: directConversationPda,
      payer: contactBKeypair.publicKey,
      contactA: contactAPda,
      contactB: contactBPda,
    })
    .transaction();
  
    const responseB = await anchor.web3.sendAndConfirmTransaction(provider.connection, txB, [contactBKeypair]);
    const conversationB = await program.account.directConversation.fetch(directConversationPda);
    expect(conversationB.messages[2]).is.equal(contactBMessage);   
  });

});
