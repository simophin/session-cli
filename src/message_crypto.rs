use std::borrow::Cow;

pub fn strip_message_padding(data: &[u8]) -> &[u8] {
    match data.iter().rposition(|x| *x == 0x80 || *x != 0x00) {
        Some(padding_start) if data[padding_start] == 0x80 => &data[..padding_start],
        _ => data,
    }
}

pub fn pad_message(data: Cow<[u8]>) -> Vec<u8> {
    let mut owned = data.into_owned();

    // Make sure the message ends with a 0x80 byte
    owned.push(0x80);

    // Work out how many bytes we need to add to make the message a multiple of 160
    let padding = 160 - (owned.len() % 160);
    let new_len = owned.len() + padding;
    owned.resize(new_len, 0);
    owned
}
