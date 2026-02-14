use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};
use prop_amm_submission_sdk::{set_return_data_bytes, set_return_data_u64};

const NAME: &str = "Prop AMM";
const MODEL_USED: &str = "Claude";
const FEE_NUMERATOR: u128 = 950;
const FEE_DENOMINATOR: u128 = 1000;
const STORAGE_SIZE: usize = 1024;

#[derive(wincode::SchemaRead)]
struct ComputeSwapInstruction {
    side: u8,
    input_amount: u64,
    reserve_x: u64,
    reserve_y: u64,
    _storage: [u8; STORAGE_SIZE],
}

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Ok(());
    }

    match instruction_data[0] {
        0 | 1 => {
            let output = compute_swap(instruction_data);
            set_return_data_u64(output);
        }
        2 => {}
        3 => set_return_data_bytes(NAME.as_bytes()),
        4 => set_return_data_bytes(get_model_used().as_bytes()),
        _ => {}
    }

    Ok(())
}

pub fn get_model_used() -> &'static str {
    MODEL_USED
}

pub fn compute_swap(data: &[u8]) -> u64 {
    let decoded: ComputeSwapInstruction = match wincode::deserialize(data) {
        Ok(d) => d,
        Err(_) => return 0,
    };

    let side = decoded.side;
    let input_amount = decoded.input_amount as u128;
    let reserve_x = decoded.reserve_x as u128;
    let reserve_y = decoded.reserve_y as u128;

    if reserve_x == 0 || reserve_y == 0 || input_amount == 0 {
        return 0;
    }

    // Single-division CPMM: output = rx * input * FEE_NUM / (ry * FEE_DEN + input * FEE_NUM)
    // Mathematically equivalent to fee-on-input CPMM, but uses a single floor
    // division instead of two (fee floor + CPMM ceiling). This avoids the
    // double-quantization that triggers concavity violations in the curve checker.
    match side {
        0 => {
            // Buy X: Y is input
            let num = reserve_x * input_amount * FEE_NUMERATOR;
            let den = reserve_y * FEE_DENOMINATOR + input_amount * FEE_NUMERATOR;
            (num / den) as u64
        }
        1 => {
            // Sell X: X is input
            let num = reserve_y * input_amount * FEE_NUMERATOR;
            let den = reserve_x * FEE_DENOMINATOR + input_amount * FEE_NUMERATOR;
            (num / den) as u64
        }
        _ => 0,
    }
}
