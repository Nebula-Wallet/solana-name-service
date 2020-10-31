use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    entrypoint,
    entrypoint::ProgramResult,
    log,
    program_pack::{IsInitialized, Pack, Sealed},
};
use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::{str::from_utf8, str::FromStr};

static PAYMENT_ACCOUNT_ADDRESS: &'static str = "Gsun7cGFrSUm3N8TEBq7Uu9xz4c9cE4pKdbtETQiSgZX";
static REGISTRATION_FEE: u64 = 1_000_000_000;
const STORAGE_DATA_SIZE: usize = 64;
const MAX_DATA_SIZE: usize = 32;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Register {
    pub token_address: Pubkey,
    pub token_name: [u8; 32],
}
impl Sealed for Register {}
impl IsInitialized for Register {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Pack for Register {
    const LEN: usize = STORAGE_DATA_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, STORAGE_DATA_SIZE];
        let (token_address, token_name) = array_refs![src, 32, 32];
        let token_address = Pubkey::new_from_array(*token_address);
        Ok(Register {
            token_address,
            token_name: *token_name,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, STORAGE_DATA_SIZE];
        let (token_address_dst, token_name_dst) = mut_array_refs![dst, 32, 32];
        let &Register {
            ref token_address,
            token_name,
        } = self;
        *token_name_dst = token_name;
        *token_address_dst = token_address.to_bytes();
    }
}
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    log::sol_log("Rust program entrypoint");
    let data_to_save =
        from_utf8(instruction_data).map_err(|_| ProgramError::InvalidInstructionData)?;

    let accounts_iter = &mut accounts.iter();
    let payment_account = next_account_info(accounts_iter)?;

    // Validate payment account
    if payment_account.key.to_bytes()
        != Pubkey::from_str(PAYMENT_ACCOUNT_ADDRESS)
            .unwrap()
            .to_bytes()
    {
        log::sol_log("Invalid payment_account");
        return Err(ProgramError::InvalidAccountData);
    }
    let token = next_account_info(accounts_iter)?;
    let minter_of_token = next_account_info(accounts_iter)?;
    let test_token_data = token.try_borrow_data()?;
    let owner_of_token_from_data = &test_token_data[4..36];
    // Check if user is minter of token SPL-token standard
    if owner_of_token_from_data != minter_of_token.key.to_bytes() {
        log::sol_log("You are not minter of this token");
        return Err(ProgramError::InvalidAccountData);
    }
    // Check if minter sends transaction
    if minter_of_token.is_signer == false {
        log::sol_log("Transaction need to be send from minter account");
        return Err(ProgramError::InvalidAccountData);
    }
    let storage_account = next_account_info(accounts_iter)?;
    // Check if programs owns account where we store data
    if storage_account.owner != program_id {
        log::sol_log("storage_account must be owned by program");
        return Err(ProgramError::InvalidAccountData);
    }
    let mut storage_account_balance = storage_account.try_borrow_mut_lamports()?;
    if **storage_account_balance < REGISTRATION_FEE {
        log::sol_log("InsufficientFunds in storage_account");
        return Err(ProgramError::InsufficientFunds);
    }
    let mut payment_account_balance = payment_account.try_borrow_mut_lamports()?;
    // Transfer fee
    **storage_account_balance = storage_account_balance.wrapping_sub(REGISTRATION_FEE);
    **payment_account_balance = payment_account_balance.wrapping_add(REGISTRATION_FEE);
    let mut storage_account_data = storage_account.try_borrow_mut_data()?;
    // log::sol_log(&storage_account_data.len().to_string());
    // log::sol_log(&STORAGE_DATA_SIZE.to_string());

    // I should add special field but is good enough for now
    if storage_account_data[0] != 0 {
        log::sol_log("Account data is not empty.");
        return Err(ProgramError::InvalidAccountData);
    }
    if storage_account_data.len().lt(&STORAGE_DATA_SIZE) {
        log::sol_log("Account data field is insufficient");
        return Err(ProgramError::InvalidAccountData);
    }
    if data_to_save.len().gt(&MAX_DATA_SIZE) {
        log::sol_log("Account data field is insufficient");
        return Err(ProgramError::InvalidAccountData);
    }
    if instruction_data.len() != 32 {
        log::sol_log("Inavlid  instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }
    let token_name = array_ref![instruction_data, 0, 32];

    let data_to_store = Register {
        token_address: *token.key,
        token_name: *token_name,
    };
    // Store data
    data_to_store.pack_into_slice(&mut storage_account_data);
    Ok(())
}

// tests
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use solana_sdk::clock::Epoch;

    #[test]
    fn test_struct() {
        // mock program id
        let token_address =
            Pubkey::from_str("4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra").unwrap();
        let token_name: [u8; 32] = *b"some super random token name xxx";
        let data = [token_address.to_bytes(), token_name].concat();
        let register_data = Register {
            token_address,
            token_name,
        };
        let register_data_from_slice = Register::unpack_from_slice(&data).unwrap();
        assert_eq!(register_data_from_slice, register_data);
        assert_eq!(register_data_from_slice.token_address, token_address);
        assert_eq!(
            from_utf8(&register_data_from_slice.token_name).unwrap(),
            "some super random token name xxx"
        );
    }
    #[test]
    fn test_flow() {
        // mock program id

        let program_id = Pubkey::default();

        let mut empty_data = vec![0; 0];
        let mut empty_data2 = vec![0; 0];
        let mut balance_payment_account = 0;
        let mut balance_token = 0;
        let mut balance_minter_token = 0;
        let mut balance_storage_account = REGISTRATION_FEE + 1;
        let mut storage_data = vec![0; STORAGE_DATA_SIZE];

        let mut test_token_data: [u8; 82] = [
            0x01, 0x00, 0x00, 0x00, 0x32, 0x06, 0x37, 0xf2, 0xb0, 0x59, 0x03, 0x06, 0x66, 0x34,
            0xb4, 0x2f, 0x05, 0x3f, 0x33, 0xe3, 0xd9, 0x49, 0xa0, 0x94, 0xf6, 0xc1, 0x7d, 0x98,
            0x55, 0x83, 0x84, 0xbf, 0x98, 0x17, 0xcd, 0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x09, 0x01, 0x01, 0x00, 0x00, 0x00, 0x32, 0x06, 0x37, 0xf2, 0xb0, 0x59,
            0x03, 0x06, 0x66, 0x34, 0xb4, 0x2f, 0x05, 0x3f, 0x33, 0xe3, 0xd9, 0x49, 0xa0, 0x94,
            0xf6, 0xc1, 0x7d, 0x98, 0x55, 0x83, 0x84, 0xbf, 0x98, 0x17, 0xcd, 0xcb,
        ];
        let owner_of_token_address =
            Pubkey::from_str("4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra").unwrap();
        let owner_of_token_from_data = &test_token_data[4..36];
        assert_eq!(owner_of_token_address.to_bytes(), owner_of_token_from_data);
        let token_address =
            Pubkey::from_str("FJNj5YDJVT3pbtiZsMMeRPJumM35iF71rUzDQWC7CXqq").unwrap();

        let owner = Pubkey::default();
        let payment_account_key = Pubkey::from_str(PAYMENT_ACCOUNT_ADDRESS).unwrap();
        let payment_account = AccountInfo::new(
            &payment_account_key,         // account pubkey
            false,                        // is_signer
            true,                         // is_writable
            &mut balance_payment_account, // balance in lamports
            &mut empty_data,              // storage
            &owner,                       // owner pubkey
            false,                        // is_executable
            Epoch::default(),             // rent_epoch
        );
        let token = AccountInfo::new(
            &token_address,       // account pubkey
            false,                // is_signer
            true,                 // is_writable
            &mut balance_token,   // balance in lamports
            &mut test_token_data, // storage
            &owner,               // owner pubkey
            false,                // is_executable
            Epoch::default(),     // rent_epoch
        );
        let minter_of_token = AccountInfo::new(
            &owner_of_token_address,   // account pubkey
            true,                      // is_signer
            true,                      // is_writable
            &mut balance_minter_token, // balance in lamports
            &mut empty_data2,          // storage
            &owner,                    // owner pubkey
            false,                     // is_executable
            Epoch::default(),          // rent_epoch
        );
        let random_address = Pubkey::default();
        let storage_account = AccountInfo::new(
            &random_address,              // account pubkey
            false,                        // is_signer
            true,                         // is_writable
            &mut balance_storage_account, // balance in lamports
            &mut storage_data,            // storage
            &program_id,                  // owner pubkey
            false,                        // is_executable
            Epoch::default(),             // rent_epoch
        );
        let accounts = vec![payment_account, token, minter_of_token, storage_account];

        let token_name: [u8; 32] = *b"some super random token name xxx";
        let instruction_data = token_name;
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // Check if fund are moved
        let payment_target = accounts[0].lamports.borrow();
        assert_eq!(**payment_target, REGISTRATION_FEE);
        // Check if data stored is correct
        let data_stored = accounts[3].data.borrow();
        let registered_data = Register::unpack_from_slice(&data_stored).unwrap();
        assert_eq!(registered_data.token_address, token_address);
        assert_eq!(registered_data.token_name, token_name);
    }
}
