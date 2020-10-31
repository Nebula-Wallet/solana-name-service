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
const STORAGE_DATA_SIZE: usize = 33;
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
impl Pack for Pointer {
    const LEN: usize = STORAGE_DATA_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, STORAGE_DATA_SIZE];
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
        let dst = array_mut_ref![dst, 0, STORAGE_DATA_SIZE];
        let (token_address_dst, is_initialized_dst) = mut_array_refs![dst, 32, 1];
        let &Pointer {
            ref token_address,
            is_initialized,
        } = self;
        is_initialized_dst[0] = is_initialized as u8;
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
    let accounts_iter = &mut accounts.iter();
    let storage_account = next_account_info(accounts_iter)?;
    // Check if programs owns account where we store data
    let address_to_point = array_ref![instruction_data, 0, 32];
    let mut storage_account_data = storage_account.try_borrow_mut_data()?;
    if storage_account.owner != program_id {
        log::sol_log("storage_account must be owned by program");
        return Err(ProgramError::InvalidAccountData);
    }
    // I should add special field but is good enough for now
    if storage_account_data[0] != 0 {
        log::sol_log("Account data is not empty.");
        return Err(ProgramError::InvalidAccountData);
    }
    if storage_account_data.len().lt(&STORAGE_DATA_SIZE) {
        log::sol_log("Account data field is insufficient");
        return Err(ProgramError::InvalidAccountData);
    }
    let pointer = Pointer {
        is_initialized: true,
        token_address: Pubkey::new_from_array(*address_to_point),
    };
    // Store data
    Pointer::pack_into_slice(&pointer, &mut storage_account_data);

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
        let program_id = Pubkey::default();
        let mut balance = 0;
        let mut empty_data = vec![0; 33];
        let storage_address =
            Pubkey::from_str("4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra").unwrap();
        let storage_account = AccountInfo::new(
            &storage_address, // account pubkey
            false,            // is_signer
            true,             // is_writable
            &mut balance,     // balance in lamports
            &mut empty_data,  // storage
            &program_id,      // owner pubkey
            false,            // is_executable
            Epoch::default(), // rent_epoch
        );
        let accounts = vec![storage_account];
        let token_address =
            Pubkey::from_str("FJNj5YDJVT3pbtiZsMMeRPJumM35iF71rUzDQWC7CXqq").unwrap();
        let instruction_data = token_address.to_bytes();
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        
        // Check if data stored is correct
        let data_stored = accounts[0].data.borrow();
        let pointer = Pointer::unpack_from_slice(&data_stored).unwrap();
        assert_eq!(pointer.token_address, token_address);
        assert_eq!(pointer.is_initialized, true);
    }
}
