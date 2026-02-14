use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};
use prop_amm_submission_sdk::{set_return_data_bytes, set_return_data_u64, set_storage};

const NAME: &str = "Dynamic Fee CPMM";
const MODEL_USED: &str = "Claude";
const STORAGE_SIZE: usize = 1024;

const PRICE_SCALE: u128 = 1_000_000_000;
const DEFAULT_FEE_BPS: u64 = 480;
const INIT_VOL_EMA: u64 = 10000;

// Storage layout (bytes)
const OFF_LAST_PRICE: usize = 0; // u64: price * 1e9
const OFF_VOL_EMA: usize = 8;    // u64: EMA of squared returns
const OFF_FEE_BPS: usize = 16;   // u64: current fee in basis points

#[derive(wincode::SchemaRead)]
struct ComputeSwapInstruction {
    side: u8,
    input_amount: u64,
    reserve_x: u64,
    reserve_y: u64,
    storage: [u8; STORAGE_SIZE],
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
        2 => {
            if instruction_data.len() >= 42 + STORAGE_SIZE {
                let mut storage_buf = [0u8; STORAGE_SIZE];
                storage_buf.copy_from_slice(&instruction_data[42..42 + STORAGE_SIZE]);
                after_swap(instruction_data, &mut storage_buf);
                let _ = set_storage(&storage_buf);
            }
        }
        3 => set_return_data_bytes(NAME.as_bytes()),
        4 => set_return_data_bytes(get_model_used().as_bytes()),
        _ => {}
    }

    Ok(())
}

pub fn get_model_used() -> &'static str {
    MODEL_USED
}

fn read_u64_le(buf: &[u8], offset: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[offset..offset + 8]);
    u64::from_le_bytes(bytes)
}

fn write_u64_le(buf: &mut [u8], offset: usize, val: u64) {
    buf[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
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

    // Read fee from storage (dynamic, updated by after_swap)
    let fee_bps_raw = read_u64_le(&decoded.storage, OFF_FEE_BPS);
    let fee_bps = if fee_bps_raw < 19 {
        DEFAULT_FEE_BPS
    } else {
        fee_bps_raw.min(2000)
    };

    let k = reserve_x * reserve_y;
    let fee_num = 10000u128 - fee_bps as u128;

    match side {
        0 => {
            // Buy X: Y is input
            let net_input = input_amount * fee_num / 10000;
            let new_ry = reserve_y + net_input;
            let k_div = (k + new_ry - 1) / new_ry;
            reserve_x.saturating_sub(k_div) as u64
        }
        1 => {
            // Sell X: X is input
            let net_input = input_amount * fee_num / 10000;
            let new_rx = reserve_x + net_input;
            let k_div = (k + new_rx - 1) / new_rx;
            reserve_y.saturating_sub(k_div) as u64
        }
        _ => 0,
    }
}

pub fn after_swap(data: &[u8], storage: &mut [u8]) {
    if data.len() < 42 {
        return;
    }

    // Parse post-trade reserves from after_swap instruction data
    let reserve_x = read_u64_le(data, 18) as u128;
    let reserve_y = read_u64_le(data, 26) as u128;

    if reserve_x == 0 {
        return;
    }

    // Current spot price scaled by PRICE_SCALE
    let current_price = (reserve_y * PRICE_SCALE / reserve_x) as u64;

    let last_price = read_u64_le(storage, OFF_LAST_PRICE);
    let vol_ema = read_u64_le(storage, OFF_VOL_EMA);

    if last_price == 0 {
        // First trade — initialize storage with reasonable vol estimate
        write_u64_le(storage, OFF_LAST_PRICE, current_price);
        write_u64_le(storage, OFF_VOL_EMA, INIT_VOL_EMA);
        write_u64_le(storage, OFF_FEE_BPS, DEFAULT_FEE_BPS);
        return;
    }

    // Squared return: var_obs = ((p1 - p0) / p0)^2, scaled
    let price_diff = if current_price > last_price {
        (current_price - last_price) as u128
    } else {
        (last_price - current_price) as u128
    };
    let ret_scaled = price_diff * PRICE_SCALE / (last_price as u128);
    let var_obs = (ret_scaled * ret_scaled / PRICE_SCALE) as u64;

    // EMA update: alpha = 30/128
    // new_ema = (30 * var_obs + 98 * vol_ema) / 128
    let new_vol_ema = (30u64.saturating_mul(var_obs)
        .saturating_add(98u64.saturating_mul(vol_ema)))
        / 128;

    // Fee calculation
    let fee_bps = if new_vol_ema == 0 {
        DEFAULT_FEE_BPS
    } else {
        let base_fee = new_vol_ema / 120 + 400;

        // Spike: moderate multiplier on variance spikes
        let spike_ratio_x128 = (var_obs as u128) * 128 / (new_vol_ema as u128);
        let fee = if spike_ratio_x128 > 128 {
            let spike_x128 = ((spike_ratio_x128 - 128) * 3).min(4 * 128);
            (base_fee as u128 * (128 + spike_x128) / 128) as u64
        } else {
            base_fee
        };

        fee.max(300).min(2000)
    };

    write_u64_le(storage, OFF_LAST_PRICE, current_price);
    write_u64_le(storage, OFF_VOL_EMA, new_vol_ema);
    write_u64_le(storage, OFF_FEE_BPS, fee_bps);
}
