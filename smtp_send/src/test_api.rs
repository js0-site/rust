
use mail_send::smtp::message::Message;

pub fn test(mut msg: Message<Vec<u8>>) {
    // Try to access fields or methods to guess API
    let _ = msg.rcpt_to; 
    let _ = msg.to;
    let _ = msg.envelope_recipients;
    
    // Try methods
    msg.rcpt(vec!["test"]);
}
