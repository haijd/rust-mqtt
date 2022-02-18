use super::property::Property;
use super::packet_type::PacketType;
use crate::utils::buffer_reader::BuffReader;
use crate::utils::buffer_reader::EncodedString;
use crate::utils::buffer_reader::BinaryData;
use crate::utils::buffer_reader::ParseError;
use crate::packet::mqtt_packet::Packet;
use heapless::Vec;

pub const MAX_PROPERTIES: usize = 18;
pub const MAX_WILL_PROPERTIES: usize = 7;

pub struct ControlPacket<'a> {
    // 7 - 4 mqtt control packet type, 3-0 flagy
    pub fixed_header: u8,
    // 1 - 4 B lenght of variable header + len of payload
    pub remain_len: u32,

    // variable header
    //optional  prida se pouze u packetu ve kterych ma co delat 
    pub packet_identifier: u16,
    pub protocol_name_len: u16,
    pub protocol_name: u32,
    pub protocol_version: u8,
    pub connect_flags: u8,
    pub keep_alive: u16,
    // property len
    pub property_len: u32,

    // properties
    pub properties: Vec<Property<'a>, MAX_PROPERTIES>,

    //payload
    pub client_id: EncodedString<'a>,
    // property len
    pub will_property_len: u32,
    pub will_properties: Vec<Property<'a>, MAX_WILL_PROPERTIES>,
    pub will_topic: EncodedString<'a>,
    pub will_payload: BinaryData<'a>,
    pub username: EncodedString<'a>,
    pub password: BinaryData<'a>
}

impl<'a> ControlPacket<'a> {
    pub fn clean(properties: Vec<Property<'a>, MAX_PROPERTIES>, will_properties: Vec<Property<'a>, MAX_WILL_PROPERTIES> ) -> Self {
        Self{ fixed_header: 0x00, remain_len: 0, packet_identifier: 0, protocol_name_len: 0, protocol_name: 0, protocol_version: 5, connect_flags: 0, 
            keep_alive: 0, property_len: 0, properties, client_id: EncodedString::new(), will_property_len: 0, will_properties, will_topic: EncodedString::new(), 
            will_payload: BinaryData::new(), username: EncodedString::new(), password: BinaryData::new() }
    }

    pub fn get_reason_code(&self) {
        log::info!("Getting reason code!");
    }

    pub fn addPacketType(& mut self, new_packet_type: PacketType) {
        self.fixed_header = self.fixed_header & 0x0F;
        self.fixed_header = self.fixed_header | <PacketType as Into<u8>>::into(new_packet_type);
    }

    pub fn addFlags(& mut self, dup: bool, qos: u8, retain: bool) {
        let cur_type: u8 = self.fixed_header & 0xF0;
        if cur_type != 0x30 {
            log::error!("Cannot add flags into packet with other than PUBLISH type");
            return;
        }
        let mut flags: u8 = 0x00;
        if dup {
            flags = flags | 0x08;
        }
        if qos == 1 {
            flags = flags | 0x02;
        }
        if qos == 2 {
            flags = flags | 0x04;
        }
        if retain {
            flags = flags | 0x01;
        }
        self.fixed_header = cur_type | flags;
    }

    pub fn decode_fixed_header(& mut self, buff_reader: & mut BuffReader) -> PacketType {
        let first_byte: u8 = buff_reader.readU8().unwrap();
        self.fixed_header = first_byte;
        self.remain_len = buff_reader.readVariableByteInt().unwrap();
        return PacketType::from(self.fixed_header);
    }

    pub fn decode_properties(& mut self, buff_reader: & mut BuffReader<'a>) {

        self.property_len = buff_reader.readVariableByteInt().unwrap();
        let mut x: u32 = 0;
        let mut prop: Result<Property, ParseError>;
        loop {
            let mut res: Property;
            prop = Property::decode(buff_reader);
            if let Ok(res) = prop {
                log::info!("Parsed property {:?}", res);
                x = x + res.len() as u32 + 1;
                self.properties.push(res);
            } else {
                // error handlo
                log::error!("Problem during property decoding");
            }
            
            if x == self.property_len {
                break;
            }
        }
    }

    pub fn decode_will_properties(& mut self, buff_reader: & mut BuffReader<'a>) {
        //todo: need to check if we are parsing only will properties
        let will_property_len = buff_reader.readVariableByteInt().unwrap();
        let mut x: u32 = 0;
        let mut prop: Result<Property, ParseError>;
        loop {
            let mut res: Property;
            prop = Property::decode(buff_reader);
            if let Ok(res) = prop {
                log::info!("Will property parsed: {:?}", res);
                x = x + res.len() as u32 + 1;
                self.will_properties.push(res);
            } else {
                // error handlo
                log::error!("Problem during property decoding");
            }
            
            if x == will_property_len {
                break;
            }
        }
    }

    pub fn decode_payload(& mut self, buff_reader: & mut BuffReader<'a>) {
        self.client_id = buff_reader.readString().unwrap();
        if self.connect_flags & (1 << 2) == 1 {
            self.decode_will_properties(buff_reader);
            self.will_topic = buff_reader.readString().unwrap();
            self.will_payload = buff_reader.readBinary().unwrap();
        }
        
        if self.connect_flags & (1 << 7) == 1 {
            self.username = buff_reader.readString().unwrap();
        }
        if self.connect_flags & (1 << 6) == 1 {
            self.password = buff_reader.readBinary().unwrap();
        }
    }

    pub fn decode_control_packet(& mut self, buff_reader: & mut BuffReader<'a>) {
        if self.decode_fixed_header(buff_reader) != (PacketType::Connect).into() {
            log::error!("Packet you are trying to decode is not CONNECT packet!");
        }
        self.packet_identifier = 0;
        self.protocol_name_len = buff_reader.readU16().unwrap();
        self.protocol_name = buff_reader.readU32().unwrap();
        self.protocol_version = buff_reader.readU8().unwrap();
        self.connect_flags = buff_reader.readU8().unwrap();
        self.keep_alive = buff_reader.readU16().unwrap();
        self.decode_properties(buff_reader);
        self.decode_payload(buff_reader);
    }
}

impl<'a> Packet<'a> for ControlPacket<'a> {
    fn decode(& mut self, buff_reader: & mut BuffReader<'a>) {
        log::error!("Decode function is not available for control packet!")
        //self.decode_control_packet(buff_reader);
    }

    fn encode(& mut self, buffer: & mut [u8]) {

    }
}