import * as anchor from "@project-serum/anchor";
import { Program, translateAddress } from "@project-serum/anchor";
import { Solchat } from "../target/types/solchat";
import {PublicKey, Keypair, sendAndConfirmTransaction } from "@solana/web3.js";
import { expect } from "chai";

describe("solchat", () => {
  const contactAKeypair = Keypair.generate();
  const contactBKeypair = Keypair.generate();
  const conversationStartMessage = "Let's chat!";
  const groupName = "Group1";
  const groupNonce = 0;

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
  let [groupPda, groupPdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("group"), 
      contactAKeypair.publicKey.toBuffer(),
      new anchor.BN(groupNonce).toArrayLike(Buffer, 'be', 2),
    ], program.programId);
  let [contactAGroupPda, contactAGroupPdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("group_contact"),
      groupPda.toBuffer(),
      contactAKeypair.publicKey.toBuffer(),
    ], program.programId);
  let [contactBGroupPda, contactBGroupPdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("group_contact"),
      groupPda.toBuffer(),
      contactBKeypair.publicKey.toBuffer(),
    ], program.programId);

  const contacts = [contactAPda, contactBPda];
  contacts.sort();

  let [directConversationPda, directConversationPdaBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("direct_conversation"), 
        contacts[0].toBuffer(),
        contacts[1].toBuffer(),             
      ], program.programId);


  before((done) => {
    console.log('funding contact accounts');
    const airDropA = provider.connection
      .requestAirdrop(contactAKeypair.publicKey, 20000000)
      .catch(err=>done(err));

    const airDropB =  provider.connection
      .requestAirdrop(contactBKeypair.publicKey, 20000000)
      .catch(err=>done(err));

    Promise
      .all([airDropA, airDropB])
      .then(async(signatures)=>{
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
        contact1: contacts[0],
        contact2: contacts[1],
      })
      .transaction();
    
    const responseA = await anchor.web3.sendAndConfirmTransaction(provider.connection, txA, [contactAKeypair]);
    const conversation = await program.account.directConversation.fetch(directConversationPda);
    expect(conversation.bump).is.equal(directConversationPdaBump);
    expect(conversation.contact1).is.eql(contacts[0]);
    expect(conversation.contact2).is.eql(contacts[1]);
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
        contact1: contacts[0],
        contact2: contacts[1],
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
      contact1: contacts[0],
      contact2: contacts[1],
    })
    .transaction();
  
    const responseB = await anchor.web3.sendAndConfirmTransaction(provider.connection, txB, [contactBKeypair]);
    const conversationB = await program.account.directConversation.fetch(directConversationPda);
    expect(conversationB.messages[2]).is.equal(contactBMessage);   
  });

  it("Create Group", async () => {
    const groupData = "";
    const tx = await program.methods
      .createGroup(groupNonce, groupName, groupData)
      .accounts({
        signer: contactAKeypair.publicKey,
        contact: contactAPda,
        group: groupPda,
        signerGroupContact: contactAGroupPda,
      })
      .transaction();
    
    const txSignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [contactAKeypair]);
    const group = await program.account.group.fetch(groupPda);
    expect(group.bump).is.equal(groupPdaBump);
    expect(group.nonce).is.equal(groupNonce);
    expect(group.owner).is.eql(contactAKeypair.publicKey);
    expect(group.name).is.equal(groupName);
    expect(group.data).is.equal(groupData);

    const groupContact = await program.account.groupContact.fetch(contactAGroupPda);
    expect(groupContact.bump).is.equal(contactAGroupPdaBump);
    expect(groupContact.group).is.eql(groupPda);
    expect(groupContact.contact).is.eql(contactAPda);
    expect(groupContact.role).is.equal(1);
  });


  it("Add Group Contact", async () => {
    const tx = await program.methods
      .addGroupContact(0)
      .accounts({
        signer: contactAKeypair.publicKey,
        group: groupPda,
        signerGroupContact: contactAGroupPda,
        groupContact: contactBGroupPda,
        contact: contactBPda
      })
      .transaction();
    
    const txSignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [contactAKeypair]);

    const groupContact = await program.account.groupContact.fetch(contactBGroupPda);
    expect(groupContact.bump).is.equal(contactBGroupPdaBump);
    expect(groupContact.group).is.eql(groupPda);
    expect(groupContact.contact).is.eql(contactBPda);
    expect(groupContact.role).is.equal(0);
  });

});
