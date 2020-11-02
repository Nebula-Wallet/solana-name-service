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
static COUNTER_POINTER_ADDRESS: &'static str = "2Q8AV9MbnKYoVR1ttvmsDUxrNZUKuaDEEr3woFQToTYA";
static REGISTRATION_FEE: u64 = 1_000_000_000;
const STORAGE_DATA_SIZE: usize = 73;
const INSTRUCTION_DATA_SIZE: usize = 64;
const POINTER_DATA_SIZE: usize = 33;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pointer {
    pub token_address: Pubkey,
    pub is_initialized: bool,
}
impl Sealed for Pointer {}
impl IsInitialized for Pointer {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
// Copypasta from proxy-pointer
// error: linking with cc failed: exit code: 1 when using as dependency
impl Pack for Pointer {
    const LEN: usize = POINTER_DATA_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, POINTER_DATA_SIZE];
        let (token_address, is_initialized) = array_refs![src, 32, 1];
        let token_address = Pubkey::new_from_array(*token_address);
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(Pointer {
            token_address,
            is_initialized,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, POINTER_DATA_SIZE];
        let (token_address_dst, is_initialized_dst) = mut_array_refs![dst, 32, 1];
        let &Pointer {
            ref token_address,
            is_initialized,
        } = self;
        is_initialized_dst[0] = is_initialized as u8;
        *token_address_dst = token_address.to_bytes();
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AccountRecord {
    pub account_address: Pubkey,
    pub name: [u8; 32],
    pub is_initialized: bool,
    pub index: u64,
}
impl Sealed for AccountRecord {}
impl IsInitialized for AccountRecord {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for AccountRecord {
    const LEN: usize = STORAGE_DATA_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, STORAGE_DATA_SIZE];
        let (account_address, name, is_initialized, index) = array_refs![src, 32, 32, 1, 8];
        let account_address = Pubkey::new_from_array(*account_address);
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let index = u64::from_le_bytes(*index);
        Ok(AccountRecord {
            account_address,
            name: *name,
            is_initialized,
            index,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, STORAGE_DATA_SIZE];
        let (account_address_dst, name_dst, is_initialized_dst, index_dst) =
            mut_array_refs![dst, 32, 32, 1, 8];

        let &AccountRecord {
            ref account_address,
            name,
            is_initialized,
            index,
        } = self;
        is_initialized_dst[0] = is_initialized as u8;
        *account_address_dst = account_address.to_bytes();
        *name_dst = name;
        *index_dst = index.to_le_bytes();
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Instruction {
    pub account_address: Pubkey,
    pub name: [u8; 32],
}
impl Sealed for Instruction {}
impl IsInitialized for Instruction {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Pack for Instruction {
    const LEN: usize = INSTRUCTION_DATA_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, INSTRUCTION_DATA_SIZE];
        let (account_address, name) = array_refs![src, 32, 32];
        let account_address = Pubkey::new_from_array(*account_address);
        Ok(Instruction {
            account_address,
            name: *name,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, INSTRUCTION_DATA_SIZE];
        let (account_address_dst, name_dst) = mut_array_refs![dst, 32, 32];

        let &Instruction {
            ref account_address,
            name,
        } = self;
        *account_address_dst = account_address.to_bytes();
        *name_dst = name;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Counter {
    pub index: u64,
}
impl Sealed for Counter {}
impl IsInitialized for Counter {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Pack for Counter {
    const LEN: usize = 8;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 8];
        let (_, index) = array_refs![src, 0, 8];
        let index = u64::from_le_bytes(*index);
        Ok(Counter { index })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 8];
        let (_, index_dst) = mut_array_refs![dst, 0, 8];

        let &Counter { index } = self;
        *index_dst = index.to_le_bytes();
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
    let counter_pointer = next_account_info(accounts_iter)?;
    if counter_pointer.key.to_bytes()
        != Pubkey::from_str(COUNTER_POINTER_ADDRESS)
            .unwrap()
            .to_bytes()
    {
        log::sol_log("Invalid counter_pointer");
        return Err(ProgramError::InvalidAccountData);
    }
    let counter_pointer_data = counter_pointer.try_borrow_data()?;
    let counter_pointer_data = Pointer::unpack_from_slice(&counter_pointer_data)?;
    if counter_pointer_data.is_initialized == false {
        log::sol_log("counter pointer is not initialized");
        return Err(ProgramError::InvalidAccountData);
    }
    let counter = next_account_info(accounts_iter)?;
    if counter.key.to_bytes() != counter_pointer_data.token_address.to_bytes() {
        log::sol_log("Invalid counter address");
        return Err(ProgramError::InvalidAccountData);
    }
    let mut counter_data = counter.try_borrow_mut_data()?;
    let counter_value_slice = array_ref![counter_data, 0, 8];
    let mut counter = Counter::unpack_from_slice(counter_value_slice)?;
    // Increment counter
    counter.index = counter.index + 1;
    log::sol_log(&counter.index.to_string());
    Counter::pack_into_slice(&counter, &mut counter_data);
    println!("Current counter:{:?}", counter.index);
    // Parse instruction
    let instruction_data = Instruction::unpack_from_slice(instruction_data)?;

    let account_record = AccountRecord {
        is_initialized: true,
        account_address: instruction_data.account_address,
        name: instruction_data.name,
        index: counter.index,
    };
    // Save new record
    let storage_account = next_account_info(accounts_iter)?;
    if storage_account.owner != program_id {
        log::sol_log("storage_account must be owned by program");
        return Err(ProgramError::InvalidAccountData);
    }
    let mut storage_data = storage_account.try_borrow_mut_data()?;
    // I should add special field but is good enough for now
    if storage_data[64] != 0 {
        log::sol_log("Account data is not empty.");
        return Err(ProgramError::InvalidAccountData);
    }
    if storage_data.len().lt(&STORAGE_DATA_SIZE) {
        log::sol_log("Account data field is insufficient");
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer fee
    let mut storage_account_balance = storage_account.try_borrow_mut_lamports()?;
    if **storage_account_balance < REGISTRATION_FEE {
        log::sol_log("InsufficientFunds in storage_account");
        return Err(ProgramError::InsufficientFunds);
    }
    let mut payment_account_balance = payment_account.try_borrow_mut_lamports()?;

    **storage_account_balance = storage_account_balance.wrapping_sub(REGISTRATION_FEE);
    **payment_account_balance = payment_account_balance.wrapping_add(REGISTRATION_FEE);
    // Save data
    AccountRecord::pack_into_slice(&account_record, &mut storage_data);
    Ok(())
}

// tests
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use solana_sdk::clock::Epoch;
    #[test]
    fn test_flow() {
        // mock program id

        let program_id = Pubkey::default();

        let mut empty_data = vec![0; 0];
        let mut balance_payment_account = 0;
        let mut zero_balance = 0;
        let mut balance_minter_token = 0;
        let mut balance_storage_account = REGISTRATION_FEE + 1;
        let mut storage_data = vec![0; STORAGE_DATA_SIZE];
        let mut counter_pointer_data = vec![0; POINTER_DATA_SIZE];

        let counter_address =
            Pubkey::from_str("FJNj5YDJVT3pbtiZsMMeRPJumM35iF71rUzDQWC7CXqq").unwrap();
        let pointer = Pointer {
            is_initialized: true,
            token_address: counter_address,
        };
        Pointer::pack_into_slice(&pointer, &mut counter_pointer_data);
        let mut counter_initial_data = vec![0; 8];

        let owner = Pubkey::default();
        let payment_account_key = Pubkey::from_str(PAYMENT_ACCOUNT_ADDRESS).unwrap();
        let counter_account_key = Pubkey::from_str(COUNTER_POINTER_ADDRESS).unwrap();
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
        let counter_pointer = AccountInfo::new(
            &counter_account_key,      // account pubkey
            false,                     // is_signer
            true,                      // is_writable
            &mut zero_balance,         // balance in lamports
            &mut counter_pointer_data, // storage
            &owner,                    // owner pubkey
            false,                     // is_executable
            Epoch::default(),          // rent_epoch
        );
        let counter = AccountInfo::new(
            &counter_address,          // account pubkey
            true,                      // is_signer
            true,                      // is_writable
            &mut balance_minter_token, // balance in lamports
            &mut counter_initial_data, // storage
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
        let accounts = vec![payment_account, counter_pointer, counter, storage_account];
        let instruction = Instruction {
            account_address: Pubkey::from_str("8hVSuapWRrZXGR4MEdwCfzAi7d7hSgVwTRy6jv5kokCY")
                .unwrap(),
            name: *b"name that we want to regsiter 12",
        };
        let mut instruction_data = vec![0; INSTRUCTION_DATA_SIZE];
        instruction.pack_into_slice(&mut instruction_data);
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // Check if fund are moved
        let payment_target = accounts[0].lamports.borrow();
        assert_eq!(**payment_target, REGISTRATION_FEE);

        // Check if data stored is correct
        let data_stored = accounts[3].data.borrow();
        let registered_data = AccountRecord::unpack_from_slice(&data_stored).unwrap();
        assert_eq!(registered_data.name, instruction.name);
        assert_eq!(registered_data.account_address, instruction.account_address);
        // Check if counter incremented
        let data_stored_counter = accounts[2].data.borrow();
        let counter_data = Counter::unpack_from_slice(&data_stored_counter).unwrap();
        assert_eq!(counter_data.index, 1);
    }
}
