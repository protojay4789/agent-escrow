use {
    anchor_lang::{InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_register_agent() {
    let program_id = colosseum_programs::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/colosseum_programs.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Derive agent PDA from [b"agent", authority]
    let (agent_pda, _bump) = anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
        &[b"agent", payer.pubkey().as_ref()],
        &program_id,
    );

    let accounts = colosseum_programs::accounts::RegisterAgent {
        agent: agent_pda,
        authority: payer.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    let ix = anchor_lang::solana_program::instruction::Instruction::new_with_bytes(
        program_id,
        &colosseum_programs::instruction::RegisterAgent {
            name: "TestBot".to_string(),
            capabilities: vec!["code-review".to_string(), "audit".to_string()],
            stake_amount: 100_000_000,
        }
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "register_agent failed: {:?}", res.err());
}
