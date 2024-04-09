use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AccountAgreeClientPacket, AccountCreateClientPacket, AccountRequestClientPacket},
        server::{
            AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
            AccountReplyServerPacketReplyCodeDataChangeFailed,
            AccountReplyServerPacketReplyCodeDataChanged,
            AccountReplyServerPacketReplyCodeDataCreated,
            AccountReplyServerPacketReplyCodeDataDefault,
            AccountReplyServerPacketReplyCodeDataExists,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Params, Row};

use crate::{
    errors::WrongSessionIdError,
    player::{
        player::account::{account_exists, generate_password_hash, validate_password},
        ClientState,
    },
};

use super::super::Player;

impl Player {
    async fn account_create(&mut self, reader: EoReader) {
        let create = match AccountCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Error deserializing AccountCreateClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::Accepted {
            return;
        }

        let conn = self.pool.get_conn();

        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return;
            }
        };

        if session_id != create.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, create.session_id)
            ))
            .await;
            return;
        }

        // TODO: validate name

        let exists = match account_exists(&mut conn, &create.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        if exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let password_hash = generate_password_hash(&create.username, &create.password);

        match conn
            .exec_drop(
                include_str!("../../../sql/create_account.sql"),
                params! {
                    "name" => &create.username,
                    "password_hash" => &password_hash,
                    "real_name" => &create.full_name,
                    "location" => &create.location,
                    "email" => &create.email,
                    "computer" => &create.computer,
                    "hdid" => &create.hdid,
                    "register_ip" => &self.ip,
                },
            )
            .await
        {
            Ok(_) => {
                info!("New account: {}", create.username);

                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        AccountReplyServerPacket {
                            reply_code: AccountReply::Created,
                            reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Created(
                                AccountReplyServerPacketReplyCodeDataCreated::new(),
                            )),
                        },
                    )
                    .await;
            }
            Err(e) => {
                self.close(format!("Error creating account: {}", e)).await;
            }
        }
    }

    async fn account_request(&mut self, reader: EoReader) {
        let request = match AccountRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing AccountRequestClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::Accepted {
            return;
        }

        // TODO: validate name

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let exists = match account_exists(&mut conn, &request.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        if exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let session_id = self.generate_session_id();
        let sequence_start = self.bus.sequencer.get_start();

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Account,
                AccountReplyServerPacket {
                    reply_code: AccountReply::Unrecognized(session_id),
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Default(
                        AccountReplyServerPacketReplyCodeDataDefault { sequence_start },
                    )),
                },
            )
            .await;
    }

    async fn account_agree(&mut self, reader: EoReader) {
        let agree = match AccountAgreeClientPacket::deserialize(&reader) {
            Ok(agree) => agree,
            Err(e) => {
                error!("Error deserializing AccountAgreeClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

        let conn = self.pool.get_conn();
        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let exists = match account_exists(&mut conn, &agree.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        if !exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_password_hash.sql"),
                params! {
                    "name" => &agree.username,
                },
            )
            .await
        {
            Ok(row) => row,
            Err(e) => {
                error!("Error getting password hash: {}", e);

                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        AccountReplyServerPacket {
                            reply_code: AccountReply::ChangeFailed,
                            reply_code_data: Some(
                                AccountReplyServerPacketReplyCodeData::ChangeFailed(
                                    AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                                ),
                            ),
                        },
                    )
                    .await;
                return;
            }
        }
        .unwrap();

        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(&agree.username, &agree.old_password, &password_hash) {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::ChangeFailed,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::ChangeFailed(
                            AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let account_id: i32 = row.get("id").unwrap();

        let password_hash = generate_password_hash(&agree.username, &agree.new_password);
        if let Err(e) = conn
            .exec_drop(
                include_str!("../../../sql/update_password_hash.sql"),
                params! {
                    "id" => account_id,
                    "password_hash" => &password_hash,
                },
            )
            .await
        {
            self.close(format!("Error updating password hash: {}", e))
                .await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Account,
                AccountReplyServerPacket {
                    reply_code: AccountReply::Changed,
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Changed(
                        AccountReplyServerPacketReplyCodeDataChanged::new(),
                    )),
                },
            )
            .await;
    }

    pub async fn handle_account(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Create => self.account_create(reader).await,
            PacketAction::Request => self.account_request(reader).await,
            PacketAction::Agree => self.account_agree(reader).await,
            _ => error!("Unhandled packet Account_{:?}", action),
        }
    }
}
