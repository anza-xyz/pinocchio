#![allow(unexpected_cfgs)]

use {
    crate::instructions::initialize_permanent_delegate,
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_pubkey::declare_id,
    spl_token_2022_interface::{
        extension::{
            cpi_guard::instruction::CpiGuardInstruction,
            default_account_state::instruction::DefaultAccountStateInstruction,
            group_member_pointer::instruction::GroupMemberPointerInstruction,
            group_pointer::instruction::GroupPointerInstruction,
            interest_bearing_mint::instruction::InterestBearingMintInstruction,
            pausable::instruction::PausableInstruction,
            scaled_ui_amount::instruction::ScaledUiAmountMintInstruction,
            transfer_hook::instruction::TransferHookInstruction,
            memo_transfer::instruction::RequiredMemoTransfersInstruction,
            metadata_pointer::instruction::MetadataPointerInstruction,
        },
        instruction::{decode_instruction_type, TokenInstruction},
    },
    spl_token_group_interface::instruction::{
        InitializeGroup, InitializeMember, TokenGroupInstruction, UpdateGroupAuthority,
        UpdateGroupMaxSize,
    },
};

pub mod helpers;
mod instructions;

use instructions as i;

entrypoint!(process_instruction);
declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match TokenInstruction::unpack(instruction_data) {
        // try to match TokenInstruction
        Ok(token_instruction) => {
            match token_instruction {
                TokenInstruction::InitializeMint {
                    decimals,
                    mint_authority,
                    freeze_authority,
                } => i::initialize_mint(accounts, decimals, mint_authority, freeze_authority),

                // For Initializing TokenAccount
                TokenInstruction::InitializeAccount => i::initialize_token_account(accounts),

                TokenInstruction::GroupPointerExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: GroupPointerInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        GroupPointerInstruction::Initialize => {
                            i::group_pointer::initialize(accounts, instruction_data)
                        }
                        GroupPointerInstruction::Update => {
                            i::group_pointer::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::GroupMemberPointerExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: GroupMemberPointerInstruction =
                        decode_instruction_type(instruction_data)
                            .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        GroupMemberPointerInstruction::Initialize => {
                            i::group_member_pointer::initialize(accounts, instruction_data)
                        }
                        GroupMemberPointerInstruction::Update => {
                            i::group_member_pointer::update(accounts, instruction_data)
                        }
                    }
                }

                // MetadataPointer extention
                TokenInstruction::MetadataPointerExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: MetadataPointerInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        MetadataPointerInstruction::Initialize => {
                            i::metadata_pointer::initialize(accounts, instruction_data)
                        }
                        MetadataPointerInstruction::Update => {
                            i::metadata_pointer::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::InitializePermanentDelegate { delegate } => {
                    initialize_permanent_delegate(accounts, delegate)
                }
                TokenInstruction::CpiGuardExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: CpiGuardInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        CpiGuardInstruction::Enable => {
                            i::cpi_guard::enable_guard(accounts, instruction_data)
                        }
                        CpiGuardInstruction::Disable => {
                            i::cpi_guard::disable_guard(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::ScaledUiAmountExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: ScaledUiAmountMintInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        ScaledUiAmountMintInstruction::Initialize => {
                            i::scaled_ui_amount::initialize_scaled_ui_amount(accounts, instruction_data)
                        }
                        ScaledUiAmountMintInstruction::UpdateMultiplier => {
                            i::scaled_ui_amount::update_multiplier(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::DefaultAccountStateExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: DefaultAccountStateInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        DefaultAccountStateInstruction::Initialize => {
                            i::default_account_state::initialize(accounts, instruction_data)
                        }
                        DefaultAccountStateInstruction::Update => {
                            i::default_account_state::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::PausableExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: PausableInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        PausableInstruction::Initialize => {
                            i::pausable::initialize(accounts, instruction_data)
                        }
                        PausableInstruction::Pause => {
                            i::pausable::pause(accounts, instruction_data)
                        }
                        PausableInstruction::Resume => {
                            i::pausable::resume(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::TransferHookExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: TransferHookInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        TransferHookInstruction::Initialize => {
                            i::transfer_hook::initialize(accounts, instruction_data)
                        }
                        TransferHookInstruction::Update => {
                            i::transfer_hook::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::InterestBearingMintExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: InterestBearingMintInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        InterestBearingMintInstruction::Initialize => {
                            i::interest_bearing_mint::initialize(accounts, instruction_data)
                        }
                        InterestBearingMintInstruction::UpdateRate => {
                            i::interest_bearing_mint::update_rate(accounts, instruction_data)
                        }
                    }
                }

                // MemoTransfer Extention
                TokenInstruction::MemoTransferExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: RequiredMemoTransfersInstruction =
                        decode_instruction_type(instruction_data)
                            .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        RequiredMemoTransfersInstruction::Enable => {
                            i::memo_transfer::enable(accounts, instruction_data)
                        }
                        RequiredMemoTransfersInstruction::Disable => {
                            i::memo_transfer::disable(accounts, instruction_data)
                        }
                    }
                }

                _ => Err(ProgramError::InvalidInstructionData)?,
            }
        }
        Err(_) => {
            // try to match TokenGroupInstruction
            match TokenGroupInstruction::unpack(instruction_data) {
                Ok(token_instruction) => match token_instruction {
                    TokenGroupInstruction::InitializeGroup(InitializeGroup {
                        update_authority,
                        max_size,
                    }) => i::token_group::initialize_group(accounts, update_authority, max_size),
                    TokenGroupInstruction::UpdateGroupMaxSize(UpdateGroupMaxSize { max_size }) => {
                        i::token_group::update_max_size(accounts, max_size)
                    }
                    TokenGroupInstruction::UpdateGroupAuthority(UpdateGroupAuthority {
                        new_authority,
                    }) => i::token_group::update_group_authority(accounts, new_authority),
                    TokenGroupInstruction::InitializeMember(InitializeMember) => {
                        i::token_group::initialize_member(accounts)
                    }
                },
                _ => Err(ProgramError::InvalidInstructionData)?,
            }
        }
    }
}
