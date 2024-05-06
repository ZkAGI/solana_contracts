use borsh::BorshDeserialize;
use solana_program::msg;
use solana_program::program_error::ProgramError;
#[derive(Debug, PartialEq)]
pub enum GpuRegistryInstruction {
    RegisterGpu { model: String },
}

#[derive(BorshDeserialize, Debug)]
struct Payload {
    arg1: u8,
    arg3: String,
}


impl GpuRegistryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        msg!("we are deserializing data");
        let payload = Payload::try_from_slice(input).unwrap();
        msg!("{:#?}", payload);

        match payload.arg1 {
            0 => Ok(GpuRegistryInstruction::RegisterGpu {
                model: payload.arg3,
            }),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
