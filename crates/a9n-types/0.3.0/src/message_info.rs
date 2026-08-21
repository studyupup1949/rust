use crate::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageSource {
    Normal = 0,
    Fault = 1,
    Notification = 2,
    Reserved = 3,
}

impl From<u8> for MessageSource {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageSource::Normal,
            1 => MessageSource::Fault,
            2 => MessageSource::Notification,
            _ => MessageSource::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageInfo {
    pub data: Word,
}

impl MessageInfo {
    pub const BLOCK_SHIFT: Word = 0;
    pub const MESSAGE_LENGTH_SHIFT: Word = 1;
    pub const TRANSFER_COUNT_SHIFT: Word = 9;
    pub const SOURCE_SHIFT: Word = 13;
    pub const RESERVED_SHIFT: Word = 15;

    pub const BLOCK_MASK: Word = 0x1;
    pub const MESSAGE_LENGTH_MASK: Word = 0xff;
    pub const TRANSFER_COUNT_MASK: Word = 0x0f;
    pub const SOURCE_MASK: Word = 0x03;
    pub const RESERVED_MASK: Word = 0x01;

    pub fn new(
        is_block: bool,
        message_length: u8,
        transfer_count: u8,
        source: MessageSource,
    ) -> Self {
        let mut data = 0 as Word;

        data |= ((is_block as Word) & Self::BLOCK_MASK) << Self::BLOCK_SHIFT;
        data |=
            ((message_length as Word) & Self::MESSAGE_LENGTH_MASK) << Self::MESSAGE_LENGTH_SHIFT;
        data |=
            ((transfer_count as Word) & Self::TRANSFER_COUNT_MASK) << Self::TRANSFER_COUNT_SHIFT;
        data |= ((source as Word) & Self::SOURCE_MASK) << Self::SOURCE_SHIFT;

        Self { data }
    }

    pub fn normal(is_block: bool, message_length: u8, transfer_count: u8) -> Self {
        Self::new(
            is_block,
            message_length,
            transfer_count,
            MessageSource::Normal,
        )
    }

    pub fn fault(message_length: u8) -> Self {
        Self::new(false, message_length, 0, MessageSource::Fault)
    }

    pub fn notification() -> Self {
        Self::new(false, 0, 0, MessageSource::Notification)
    }

    pub fn is_block(&self) -> bool {
        ((self.data >> Self::BLOCK_SHIFT) & Self::BLOCK_MASK) != 0
    }

    pub fn message_length(&self) -> u8 {
        ((self.data >> Self::MESSAGE_LENGTH_SHIFT) & Self::MESSAGE_LENGTH_MASK) as u8
    }

    pub fn transfer_count(&self) -> u8 {
        ((self.data >> Self::TRANSFER_COUNT_SHIFT) & Self::TRANSFER_COUNT_MASK) as u8
    }

    pub fn source(&self) -> MessageSource {
        let value = ((self.data >> Self::SOURCE_SHIFT) & Self::SOURCE_MASK) as u8;

        MessageSource::from(value)
    }

    pub fn has_reserved_bit(&self) -> bool {
        ((self.data >> Self::RESERVED_SHIFT) & Self::RESERVED_MASK) != 0
    }

    pub fn is_normal(&self) -> bool {
        self.source() == MessageSource::Normal
    }

    pub fn is_fault(&self) -> bool {
        self.source() == MessageSource::Fault
    }

    pub fn is_notification(&self) -> bool {
        self.source() == MessageSource::Notification
    }
}

impl From<Word> for MessageInfo {
    fn from(data: Word) -> MessageInfo {
        MessageInfo { data }
    }
}

impl From<MessageInfo> for Word {
    fn from(info: MessageInfo) -> Word {
        info.data
    }
}
