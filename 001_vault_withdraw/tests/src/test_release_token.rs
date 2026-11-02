use anchor_client::{Client, Cluster, CommitmentConfig};
use solana_keypair::{read_keypair_file, Keypair};
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_token_interface::state::{Account as TokenAccountState, Mint as MintState};

/// 读取一只普通代币账户里的代币数量。
///
/// `data` 是该账户的原始字节。
/// 返回 `amount`。字节对不上代币账户布局时 panic。
fn token_amount(data: &[u8]) -> u64 {
    TokenAccountState::unpack(data).expect("token account").amount
}

/// 创建 Mint、两只代币账户，铸入金库后再调用 `release_token`。
///
/// 金库代币账户的 authority 是 `pot`。用户代币账户的 authority 是付款人。
/// 成功时两边的数量差都等于传入的 `amount`。
#[test]
fn test_release_token() {
    // 本程序地址。客户端用它找到要调用的程序。
    let program_id = Pubkey::try_from("5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp").unwrap();
    // 付款人密钥来自本地钱包文件。这个客户端之后会自动用它签名。
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();
    // 本题的 user 就是付款人。他的公钥会写进 pot 的种子，也会成为 user_token 的 authority。
    let user = payer.pubkey();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(program_id).unwrap();
    // Token 程序的固定地址。Mint 和两只代币账户都由它拥有。
    let token_program = spl_token_interface::ID;

    // pot 只是一个地址，按 b"pot" 和 user 公钥推出来。它不存放代币，稍后只负责签名。
    let (pot, _) =
        Pubkey::find_program_address(&[vault_withdraw::POT_SEED, user.as_ref()], &program_id);
    // 下面三把密钥都是新账户。链上还没有它们，要先创建再初始化。
    let mint = Keypair::new();
    let pot_token = Keypair::new();
    let user_token = Keypair::new();

    // 带数据的账户必须留下免租额。Mint 和代币账户的数据长度不同，所以租金不同。
    let mint_rent = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(MintState::LEN)
        .expect("mint rent");
    let token_rent = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(TokenAccountState::LEN)
        .expect("token rent");

    // 创建 Mint：系统程序先开户，所有者设为 Token 程序；再由 Token 程序写入小数位和铸币权限。
    // 铸币权限是 user。新账户必须用自己的密钥签名，所以这里加上 .signer(&mint)。
    program
        .request()
        .instruction(solana_system_interface::instruction::create_account(
            &user,
            &mint.pubkey(),
            mint_rent,
            MintState::LEN as u64,
            &token_program,
        ))
        .instruction(
            spl_token_interface::instruction::initialize_mint2(
                &token_program,
                &mint.pubkey(),
                &user,
                None,
                6,
            )
            .expect("initialize mint"),
        )
        .signer(&mint)
        .send()
        .expect("create mint");

    // 金库代币账户。authority 写成 pot，所以以后转走这里的币，必须由 pot 签名。
    program
        .request()
        .instruction(solana_system_interface::instruction::create_account(
            &user,
            &pot_token.pubkey(),
            token_rent,
            TokenAccountState::LEN as u64,
            &token_program,
        ))
        .instruction(
            spl_token_interface::instruction::initialize_account3(
                &token_program,
                &pot_token.pubkey(),
                &mint.pubkey(),
                &pot,
            )
            .expect("initialize pot token"),
        )
        .signer(&pot_token)
        .send()
        .expect("create pot token");

    // 用户自己的代币账户。authority 是 user 的钱包，收到的币以后可以由他本人再转走。
    program
        .request()
        .instruction(solana_system_interface::instruction::create_account(
            &user,
            &user_token.pubkey(),
            token_rent,
            TokenAccountState::LEN as u64,
            &token_program,
        ))
        .instruction(
            spl_token_interface::instruction::initialize_account3(
                &token_program,
                &user_token.pubkey(),
                &mint.pubkey(),
                &user,
            )
            .expect("initialize user token"),
        )
        .signer(&user_token)
        .send()
        .expect("create user token");

    // 铸币权限是 user，而 user 就是这个客户端的付款人，所以这笔 mint_to 不用再附加签名。
    // 新币进入 pot_token，不进入 user_token。
    let minted = 1_000_000;
    program
        .request()
        .instruction(
            spl_token_interface::instruction::mint_to(
                &token_program,
                &mint.pubkey(),
                &pot_token.pubkey(),
                &user,
                &[],
                minted,
            )
            .expect("mint to"),
        )
        .send()
        .expect("mint tokens");

    let pot_before = token_amount(&program.rpc().get_account(&pot_token.pubkey()).unwrap().data);
    let user_before = token_amount(&program.rpc().get_account(&user_token.pubkey()).unwrap().data);
    let amount = 400_000;

    // 调用本程序。user 由客户端自动签名；pot 的种子签名发生在程序内部，测试里看不到那一步。
    // 币从 pot_token 转到 user_token。
    program
        .request()
        .accounts(vault_withdraw::accounts::ReleaseToken {
            user,
            pot,
            mint: mint.pubkey(),
            pot_token: pot_token.pubkey(),
            user_token: user_token.pubkey(),
            token_program,
        })
        .args(vault_withdraw::instruction::ReleaseToken { amount })
        .send()
        .expect("release token");

    let pot_after = token_amount(&program.rpc().get_account(&pot_token.pubkey()).unwrap().data);
    let user_after = token_amount(&program.rpc().get_account(&user_token.pubkey()).unwrap().data);

    // 转出数量由参数决定。金库少掉的和用户账户增加的必须都是这个数。
    assert_eq!(pot_before - pot_after, amount);
    assert_eq!(user_after - user_before, amount);
}
