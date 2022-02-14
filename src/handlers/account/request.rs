use eo::{
    data::{EOChar, EOInt, Serializeable, StreamReader},
    net::packets::server::account::Reply,
    net::{packets::client::account::Request, replies::AccountReply, Action, Family},
};
use mysql_async::{prelude::*, Conn, Params, Row};

use crate::{player::Command, PacketBuf, Tx};

pub async fn request(
    buf: PacketBuf,
    tx: &Tx,
    conn: &mut Conn,
    sequence: EOInt,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let mut reply = Reply::new();
    if conn
        .exec_first::<Row, &str, Params>(
            include_str!("../login/get_account.sql"),
            params! {
                "name" => &request.name,
            },
        )
        .await?
        .is_some()
    {
        reply.reply = AccountReply::Exists;
        reply.message = "NO".to_string();
    } else {
        reply.session_id = 1000; // TODO: sessions?
        reply.sequence = sequence as EOChar;
        reply.message = "OK".to_string();

        // TODO: validate name
    }

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Account,
        reply.serialize(),
    ))?;

    Ok(())
}