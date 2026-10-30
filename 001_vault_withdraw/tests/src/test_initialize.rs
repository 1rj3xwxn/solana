use anchor_client::{Client, Cluster, CommitmentConfig};
use solana_keypair::read_keypair_file;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let counter =
        Pubkey::find_program_address(&[vault_withdraw::constants::COUNTER_SEED], &program_id).0;

    let initialize_tx = program
        .request()
        .accounts(vault_withdraw::accounts::Initialize {
            payer: payer.pubkey(),
            counter,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::Initialize {})
        .send()
        .expect("");

    println!("Initialize transaction signature {}", initialize_tx);

    let increment_tx = program
        .request()
        .accounts(vault_withdraw::accounts::Increment {
            counter,
            authority: payer.pubkey(),
        })
        .args(vault_withdraw::instruction::Increment {})
        .send()
        .expect("");

    println!("Increment transaction signature {}", increment_tx);
}

/// 先向该用户的金库转入 2_000_000 lamports，再调用 `withdraw`。
///
/// 成功时金库余额正好减少 1_000_000。没有参数，失败时测试断言会 panic。
#[test]
fn test_withdraw() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let (vault, bump) = Pubkey::find_program_address(&[b"vault", user.as_ref()], &program_id);
    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", user.as_ref()], &program_id);
    println!("vault = {vault}, bump = {bump}");

    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .send();

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &vault, 2_600_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    println!("读取转出前的金库余额");
    let before = program.rpc().get_balance(&vault).expect("balance before");
    println!("balance before = {before}");

    println!("调用 withdraw");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::Withdraw {
            user,
            vault,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::Withdraw {})
        .send()
        .expect("withdraw");
    println!("withdraw signature = {withdraw_tx}");

    println!("读取转出后的金库余额");
    let after = program.rpc().get_balance(&vault).expect("balance after");
    println!("balance after = {after}");

    println!("断言余额正好减少 1_000_000");
    assert_eq!(before - after, 1_000_000);
    println!("assert ok: {before} - {after} = 1_000_000");
}

#[test]
fn test_pay() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let (nest, bump) = Pubkey::find_program_address(&[b"nest", user.as_ref()], &program_id);
    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", user.as_ref()], &program_id);
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .send();

    println!("nest = {nest}, bump = {bump}");

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_050_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    println!("读取转出前的金库余额");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    println!("balance before = {before}");

    println!("调用 withdraw");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::Pay {
            user,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::Pay {})
        .send()
        .expect("withdraw");
    println!("withdraw signature = {withdraw_tx}");

    println!("读取转出后的金库余额");
    let after = program.rpc().get_balance(&nest).expect("balance after");
    println!("balance after = {after}");

    println!("断言余额正好减少 500_000");
    assert_eq!(before - after, 500_000);
    println!("assert ok: {before} - {after} = 500_000");
}

#[test]
fn test_pay_add_nest_state() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let user_new_program_id = Pubkey::try_from(user_new_address).unwrap();
     let user_new_program = client.program(user_new_program_id).unwrap();

    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", &user_new_address.as_ref()], &program_id);
    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", &user_new_address.as_ref()], &program_id);

     let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            999_050_000,
        ))
        .send()
        .expect("fund vault");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("fund vault");

    program
        .request()
        .accounts(vault_withdraw::accounts::SetState {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SetState { state: true })
        .signer(&user_new_keypair)
        .send()
        .expect("close nest");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &nest,
            2_050_000,
        ))
        // .signer(&user_new_keypair)
        .send()
        .expect("fund vault");

    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::Pay {
            user: user_new_address,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::Pay {})
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance after");
    assert_eq!(before - after, 0);
}

#[test]
fn test_send_out() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let (nest, bump) = Pubkey::find_program_address(&[b"nest", user.as_ref()], &program_id);
    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", user.as_ref()], &program_id);
    println!("nest = {nest}, bump = {bump}");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .send();

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_000_001,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    println!("读取转出前的金库余额");
    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_500_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    println!("调用 withdraw");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let recipient_before = program
        .rpc()
        .get_balance(&recipient)
        .expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 500_000 })
        .send()
        .expect("withdraw");
    println!("withdraw signature = {withdraw_tx}");

    println!("读取转出后的金库余额");
    let after = program.rpc().get_balance(&nest).expect("balance after");
    let recipient_after = program
        .rpc()
        .get_balance(&recipient)
        .expect("balance after");
    println!("balance after = {after}");

    println!("断言余额正好减少 500_000");
    println!("assert ok: {before} - {after} = 500_000");
    assert_eq!(before - after, 500_000);
    println!("assert ok: {recipient_after} - {recipient_before} = 500_000");
    assert_eq!(recipient_after - recipient_before, 500_000);
}

#[test]
fn test_send_out_failed() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    println!("nest = {nest}, bump = {bump}");

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_400_002,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_001,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    println!("读取转出前的金库余额");
    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_400_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    println!("调用 withdraw");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");

    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 500_000 })
        .send();
    println!("send_out result = {withdraw_tx:?}");
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    assert_eq!(after, before);
    let error_text = format!("{withdraw_tx:?}");
    assert!(error_text.contains("NotEnoughSigners"), "{error_text}");
    assert!(!error_text.contains("ConstraintSeeds"));
}

#[test]
fn test_send_out_seeds_error() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) = Pubkey::find_program_address(&[b"nest_error", user.as_ref()], &program_id);
    println!("nest = {nest}, bump = {bump}");

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_030_002,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_001,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    println!("读取转出前的金库余额");
    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_300_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    let (nest_state, bump) =
        Pubkey::find_program_address(&[b"nest_state", user.as_ref()], &program_id);
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .send();

    println!("调用 withdraw");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 500_000 })
        .send();
    println!("send_out result = {withdraw_tx:?}");
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    let error_text = format!("{withdraw_tx:?}");
    assert!(error_text.contains("ConstraintSeeds"), "{error_text}");
    assert_eq!(after, before)
}

#[test]
fn test_send_out_zero_amount() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用新用户推导金库地址");
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, _bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    println!("nest = {nest}");

    println!("普通转账到金库和新用户");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_200_002,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_001,
        ))
        .send()
        .expect("fund user");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    println!("普通转账到收款人");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_200_000,
        ))
        .send()
        .expect("fund recipient");
    println!("fund signature = {fund_tx}");

    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");

    println!("调用 withdraw");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 0 })
        .signer(&user_new_keypair)
        .send();
    println!("send_out result = {withdraw_tx:?}");
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    let error_text = format!("{withdraw_tx:?}");
    assert!(error_text.contains("金额不能为0"), "{error_text}");
    assert_eq!(after, before)
}

#[test]
fn test_send_out_when_balance_insufficient() {
    println!("读取程序 ID");
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    println!("program_id = {program_id}");

    println!("读取 ANCHOR_WALLET");
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    println!("wallet = {anchor_wallet}");

    println!("读取付款人密钥");
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    println!("payer = {}", payer.pubkey());

    println!("连接本地网络");
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    println!("cluster = localnet");

    println!("解析程序 ID");
    let program_id = Pubkey::try_from(program_id).unwrap();
    println!("program_id pubkey = {program_id}");

    println!("创建程序客户端");
    let program = client.program(program_id).unwrap();
    println!("program client ready");

    println!("用户就是这个付款人");
    let user = payer.pubkey();
    println!("user = {user}");

    println!("用 vault 和用户公钥推导金库地址");
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    println!("nest = {nest}, bump = {bump}");

    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 20_000_100,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_001,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    println!("读取转出前的金库余额");
    println!("普通转账 2_000_000 lamports 到金库");
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_010_000,
        ))
        .send()
        .expect("fund vault");
    println!("fund signature = {fund_tx}");
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    println!("调用 withdraw");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut {
            amount: 100_000_000_000,
        })
        .signer(&user_new_keypair)
        .send();
    println!("send_out result = {withdraw_tx:?}");
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    let error_text = format!("{withdraw_tx:?}");
    assert!(
        error_text.contains("Transfer: insufficient lamports"),
        "{error_text}"
    );
    assert_eq!(after, before)
}

#[test]
fn test_send_out_when_balance_below_rent_exempt() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 2_900_001,
        ))
        .send()
        .expect("fund vault");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");

    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 2_900_000 })
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_err());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    let error_text = format!("{withdraw_tx:?}");
    assert!(!error_text.contains("金额不能为0"), "{error_text}");
    assert_eq!(after, before)
}

#[test]
fn test_send_out_when_balance_equal_rent_exempt() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, _bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 890_881,
        ))
        .send()
        .expect("fund vault");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 1 })
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_ok());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    assert_eq!(after + 1, before)
}

#[test]
fn test_send_out_when_balance_equal_zero() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 890_881,
        ))
        .send()
        .expect("fund vault");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 890_881 })
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_ok());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    assert_eq!(after, 0);
    assert_eq!(before, 890_881);
}

#[test]
fn test_send_out_when_balance_equal_zero2() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest", user_new_address.as_ref()], &program_id);
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &nest, 890_881,
        ))
        .send()
        .expect("fund vault");

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);
    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 890_881 })
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_ok());
    let after = program.rpc().get_balance(&nest).expect("balance before");
    assert_eq!(after, 0);
    assert_eq!(before, 890_881);
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user: user_new_address,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 890_881 })
        .signer(&user_new_keypair)
        .send();
    let error_text = format!("{withdraw_tx:?}");
    println!("{error_text}");
    assert!(withdraw_tx.is_err());
}

#[test]
fn test_init_nest() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);

    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    let fund_tx = program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");

    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send();
    let error_text = format!("{withdraw_tx:?}");
    println!("{error_text}");
    assert!(withdraw_tx.is_ok());

    let nest_state = program
        .account::<vault_withdraw::NestState>(nest)
        .expect("read nest state");
    assert!(!nest_state.closed);
}

#[test]
fn test_set_state() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let user = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user_new_address = user_new_keypair.pubkey();
    let (nest, bump) =
        Pubkey::find_program_address(&[b"nest_state", user_new_address.as_ref()], &program_id);

    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user,
            &user_new_address,
            2_000_011,
        ))
        .send()
        .expect("fund vault");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &user, &recipient, 2_800_000,
        ))
        .send()
        .expect("fund vault");

    program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user: user_new_address,
            nest,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send();
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SetState {
            user: user_new_address,
            nest,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SetState { state: true })
        .signer(&user_new_keypair)
        .send();
    println!("{}", format!("{withdraw_tx:#?}").replace("\\n", "\n"));
    assert!(withdraw_tx.is_ok());

    let nest_state = program
        .account::<vault_withdraw::NestState>(nest)
        .expect("read nest state");
    assert!(nest_state.closed);
}

/// 关闭后 `send_out` 应在转账前失败，存 SOL 的 `nest` 余额不变。
#[test]
fn test_send_out_when_closed() {
    let program_id = "5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let payer_address = payer.pubkey();
    let user_new_keypair = solana_keypair::Keypair::new();
    let user = user_new_keypair.pubkey();
    let (nest, _bump) = Pubkey::find_program_address(&[b"nest", user.as_ref()], &program_id);
    let (nest_state, _bump) =
        Pubkey::find_program_address(&[b"nest_state", user.as_ref()], &program_id);

    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &payer_address,
            &user,
            2_000_011,
        ))
        .send()
        .expect("fund user");
    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &payer_address,
            &nest,
            2_000_000,
        ))
        .send()
        .expect("fund nest");

    let recipient_keypair = solana_keypair::Keypair::new();
    let recipient = recipient_keypair.pubkey();
    program
        .request()
        .instruction(solana_system_interface::instruction::transfer(
            &payer_address,
            &recipient,
            2_000_000,
        ))
        .send()
        .expect("fund recipient");

    program
        .request()
        .accounts(vault_withdraw::accounts::InitNest {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::InitState {})
        .signer(&user_new_keypair)
        .send()
        .expect("init nest state");
    program
        .request()
        .accounts(vault_withdraw::accounts::SetState {
            user,
            nest: nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SetState { state: true })
        .signer(&user_new_keypair)
        .send()
        .expect("close nest");

    let before = program.rpc().get_balance(&nest).expect("balance before");
    let withdraw_tx = program
        .request()
        .accounts(vault_withdraw::accounts::SendOut {
            user,
            recipient,
            nest,
            nest_state,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(vault_withdraw::instruction::SendOut { amount: 500_000 })
        .signer(&user_new_keypair)
        .send();
    assert!(withdraw_tx.is_err());
    let error_text = format!("{withdraw_tx:?}");
    assert!(error_text.contains("账户已经关闭"), "{error_text}");
    let after = program.rpc().get_balance(&nest).expect("balance after");
    assert_eq!(after, before);
}
