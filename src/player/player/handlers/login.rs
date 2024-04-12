use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::LoginRequestClientPacket,
        server::{
            LoginReply, LoginReplyServerPacket, LoginReplyServerPacketReplyCodeData,
            LoginReplyServerPacketReplyCodeDataBanned, LoginReplyServerPacketReplyCodeDataBusy,
            LoginReplyServerPacketReplyCodeDataLoggedIn, LoginReplyServerPacketReplyCodeDataOk,
            LoginReplyServerPacketReplyCodeDataWrongUser,
            LoginReplyServerPacketReplyCodeDataWrongUserPassword,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Params, Row};

use crate::{
    deep::{
        AccountRecoverPinReply, AccountRecoverReply, AccountRecoverUpdateReply,
        LoginAcceptClientPacket, LoginAcceptServerPacket, LoginAgreeClientPacket,
        LoginAgreeServerPacket, LoginCreateClientPacket, LoginCreateServerPacket,
        LoginTakeClientPacket, LoginTakeServerPacket,
    },
    player::{
        player::account::{
            account_banned, account_exists, generate_password_hash, get_character_list,
            update_last_login_ip, validate_password,
        },
        ClientState,
    },
    utils::{mask_email, send_email},
    EMAILS, SETTINGS,
};

use super::super::Player;

impl Player {
    async fn login_request(&mut self, reader: EoReader) {
        let request = match LoginRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing LoginRequestClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::Accepted {
            self.close("Logging in before connection accepted".to_string())
                .await;
            return;
        }

        let player_count = self.world.get_player_count().await;
        if player_count >= SETTINGS.server.max_players {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::Busy,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::Busy(
                            LoginReplyServerPacketReplyCodeDataBusy::new(),
                        )),
                    },
                )
                .await;

            self.close("Server busy".to_string()).await;

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

        let exists = match account_exists(&mut conn, &request.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        self.login_attempts += 1;

        if !exists {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::WrongUser,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::WrongUser(
                            LoginReplyServerPacketReplyCodeDataWrongUser::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let banned = match account_banned(&mut conn, &request.username).await {
            Ok(banned) => banned,
            Err(e) => {
                self.close(format!("Error checking if account is banned: {}", e))
                    .await;
                return;
            }
        };

        if banned {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::Banned,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::Banned(
                            LoginReplyServerPacketReplyCodeDataBanned::new(),
                        )),
                    },
                )
                .await;
            self.close("Account is banned".to_string()).await;
            return;
        }

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_password_hash.sql"),
                params! {
                    "name" => &request.username,
                },
            )
            .await
        {
            Ok(row) => row,
            Err(e) => {
                self.close(format!("Error getting password hash: {}", e))
                    .await;
                return;
            }
        }
        .unwrap();

        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(&request.username, &request.password, &password_hash) {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::WrongUserPassword,
                        reply_code_data: Some(
                            LoginReplyServerPacketReplyCodeData::WrongUserPassword(
                                LoginReplyServerPacketReplyCodeDataWrongUserPassword::new(),
                            ),
                        ),
                    },
                )
                .await;
            return;
        }

        let account_id: i32 = row.get("id").unwrap();
        if self.world.is_logged_in(account_id).await {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::LoggedIn,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::LoggedIn(
                            LoginReplyServerPacketReplyCodeDataLoggedIn::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        if let Err(e) = update_last_login_ip(&mut conn, account_id, &self.ip).await {
            self.close(format!("Error updating last login IP: {}", e))
                .await;
            return;
        }

        let characters = match get_character_list(&mut conn, account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return;
            }
        };

        self.world.add_logged_in_account(account_id);
        self.account_id = account_id;
        self.state = ClientState::LoggedIn;

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Login,
                LoginReplyServerPacket {
                    reply_code: LoginReply::OK,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::OK(
                        LoginReplyServerPacketReplyCodeDataOk { characters },
                    )),
                },
            )
            .await;
    }

    async fn login_take(&mut self, reader: EoReader) {
        if let Err(e) = LoginTakeClientPacket::deserialize(&reader) {
            error!("Failed to deserialize LoginTakeClientPacket: {}", e);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Take,
                PacketFamily::Login,
                LoginTakeServerPacket {
                    reply_code: if SETTINGS.account.recovery {
                        AccountRecoverReply::RequestAccepted
                    } else {
                        AccountRecoverReply::RecoveryDisabled
                    },
                },
            )
            .await;
    }

    async fn login_create(&mut self, reader: EoReader) {
        let create = match LoginCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Failed to deserialize LoginCreateClientPacket: {}", e);
                return;
            }
        };

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get SQL connection: {}", e);
                return;
            }
        };

        let mut row: Row = match conn
            .exec_first(
                include_str!("../../../sql/get_account_email.sql"),
                params! {
                    "name" => &create.account_name,
                },
            )
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                let _ = self
                    .bus
                    .send(
                        PacketAction::Create,
                        PacketFamily::Login,
                        LoginCreateServerPacket {
                            reply_code: AccountRecoverReply::AccountNotFound,
                            email_address: None,
                        },
                    )
                    .await;
                return;
            }
            Err(e) => {
                error!("Failed to get account email: {}", e);
                return;
            }
        };

        self.account_id = match row.take(0) {
            Some(id) => id,
            None => return,
        };

        let email: String = match row.take(1) {
            Some(email) => email,
            None => return,
        };

        let code = self.generate_email_pin();

        if let Err(e) = send_email(
            &email,
            &create.account_name,
            &get_lang_string!(&EMAILS.recovery.subject, name = create.account_name),
            &get_lang_string!(
                &EMAILS.recovery.body,
                name = create.account_name,
                code = code
            ),
        )
        .await
        {
            self.close(format!("Failed to send recovery email: {}", e))
                .await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Create,
                PacketFamily::Login,
                LoginCreateServerPacket {
                    reply_code: if SETTINGS.account.recovery_show_email {
                        AccountRecoverReply::RequestAcceptedShowEmail
                    } else {
                        AccountRecoverReply::RequestAccepted
                    },
                    email_address: if SETTINGS.account.recovery_show_email {
                        Some(if SETTINGS.account.recovery_mask_email {
                            mask_email(&email)
                        } else {
                            email
                        })
                    } else {
                        None
                    },
                },
            )
            .await;
    }

    async fn login_accept(&mut self, reader: EoReader) {
        let accept = match LoginAcceptClientPacket::deserialize(&reader) {
            Ok(accept) => accept,
            Err(e) => {
                error!("Failed to deserialize LoginAcceptClientPacket: {}", e);
                return;
            }
        };

        let pin = match &self.email_pin {
            Some(pin) => pin,
            None => return,
        };

        let _ = self
            .bus
            .send(
                PacketAction::Accept,
                PacketFamily::Login,
                LoginAcceptServerPacket {
                    reply_code: if *pin != accept.pin {
                        AccountRecoverPinReply::WrongPin
                    } else {
                        AccountRecoverPinReply::OK
                    },
                },
            )
            .await;
    }

    async fn login_agree(&mut self, reader: EoReader) {
        let agree = match LoginAgreeClientPacket::deserialize(&reader) {
            Ok(agree) => agree,
            Err(e) => {
                error!("Failed to deserialize LoginAgreeClientPacket: {}", e);
                self.send_login_agree_error().await;
                return;
            }
        };

        if self.account_id == 0 {
            self.send_login_agree_error().await;
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get SQL connection: {}", e);
                self.send_login_agree_error().await;
                return;
            }
        };

        let password_hash = generate_password_hash(&agree.account_name, &agree.password);

        if let Err(e) = conn
            .exec_drop(
                include_str!("../../../sql/update_password_hash.sql"),
                params! {
                    "id" => self.account_id,
                    "password_hash" => &password_hash,
                },
            )
            .await
        {
            error!("Error updating password hash: {}", e);
            self.send_login_agree_error().await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Login,
                LoginAgreeServerPacket {
                    reply_code: AccountRecoverUpdateReply::OK,
                },
            )
            .await;
    }

    async fn send_login_agree_error(&mut self) {
        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Login,
                LoginAgreeServerPacket {
                    reply_code: AccountRecoverUpdateReply::Error,
                },
            )
            .await;
    }

    pub async fn handle_login(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.login_request(reader).await,
            PacketAction::Take => self.login_take(reader).await,
            PacketAction::Create => self.login_create(reader).await,
            PacketAction::Accept => self.login_accept(reader).await,
            PacketAction::Agree => self.login_agree(reader).await,
            _ => error!("Unhandled packet Login_{:?}", action),
        }
    }
}
