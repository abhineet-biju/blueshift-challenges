use quasar_lang::prelude::*;

pub const INSTRUCTIONS_SYSVAR_ID: Address = address!("Sysvar1nstructions1111111111111111111111111");

pub struct InstructionView<'a> {
    sysvar_data: &'a [u8],
    pub program_id: Address,
    accounts_offset: usize,
    account_count: usize,
    pub data: &'a [u8],
}

impl<'a> InstructionView<'a> {
    #[inline(always)]
    pub fn account_pubkey(&self, index: usize) -> Option<Address> {
        if index >= self.account_count {
            return None;
        }

        let meta_offset = self
            .accounts_offset
            .checked_add(index.checked_mul(33)?)?
            .checked_add(1)?;

        read_address_at(self.sysvar_data, meta_offset)
    }
}

#[inline(always)]
pub fn current_instruction_index(data: &[u8]) -> Option<u16> {
    let start = data.len().checked_sub(2)?;
    Some(u16::from_le_bytes(data.get(start..)?.try_into().ok()?))
}

#[inline(always)]
pub fn instruction_count(data: &[u8]) -> Option<usize> {
    Some(read_u16_at(data, 0)? as usize)
}

#[inline(always)]
pub fn load_instruction(data: &[u8], index: usize) -> Option<InstructionView<'_>> {
    let count = instruction_count(data)?;
    if index >= count {
        return None;
    }

    let start_offset = 2usize.checked_add(index.checked_mul(2)?)?;
    let start = read_u16_at(data, start_offset)? as usize;

    let mut cursor = start;
    let account_count = read_u16(data, &mut cursor)? as usize;
    let accounts_offset = cursor;

    let program_id_offset = accounts_offset.checked_add(account_count.checked_mul(33)?)?;
    let program_id = read_address_at(data, program_id_offset)?;

    cursor = program_id_offset.checked_add(32)?;
    let data_len = read_u16(data, &mut cursor)? as usize;
    let data_end = cursor.checked_add(data_len)?;
    let ix_data = data.get(cursor..data_end)?;

    Some(InstructionView {
        sysvar_data: data,
        program_id,
        accounts_offset,
        account_count,
        data: ix_data,
    })
}

#[inline(always)]
fn read_u16(data: &[u8], cursor: &mut usize) -> Option<u16> {
    let value = read_u16_at(data, *cursor)?;
    *cursor = cursor.checked_add(2)?;
    Some(value)
}

#[inline(always)]
fn read_u16_at(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(offset..offset.checked_add(2)?)?.try_into().ok()?))
}

#[inline(always)]
fn read_address_at(data: &[u8], offset: usize) -> Option<Address> {
    let end = offset.checked_add(32)?;
    let bytes: [u8; 32] = data.get(offset..end)?.try_into().ok()?;
    Some(Address::new_from_array(bytes))
}
