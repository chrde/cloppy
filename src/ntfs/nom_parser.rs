use nom::{IResult, le_u16, le_u32};
use ntfs::FileRecordHeader;

fn file_record_header(input: &[u8]) -> IResult<&[u8], FileRecordHeader> {
    do_parse!(input,
        tag!(b"FILE") >>
        take!(2) >>
        fixup_size: le_u16 >>
        take!(12) >>
        attr_offset: le_u16 >>
        take!(22) >>
        fr_number: le_u32 >>
        fixup_seq: take!(2 * fixup_size) >>
        (FileRecordHeader{
            fr_number,
            attr_offset: attr_offset as usize,
            fixup_seq: fixup_seq.to_vec(),
        })
    )
}

