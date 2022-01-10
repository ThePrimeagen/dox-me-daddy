use serde::Deserialize;
use tokio::task::JoinError;
use tokio_tungstenite::tungstenite::{
    handshake::server::NoCallback, HandshakeError, Message, ServerHandshake,
};

use crate::forwarder::ForwarderEvent;

type ServerHandshakeError = HandshakeError<ServerHandshake<std::net::TcpStream, NoCallback>>;
type TokioTungError = tokio_tungstenite::tungstenite::Error;
type FutureSendMessage = futures_channel::mpsc::TrySendError<Message>;
type FutureSendForwarder = futures_channel::mpsc::TrySendError<ForwarderEvent>;

#[derive(Debug)]
pub enum DoxMeDaddyError {
    ServerHandshakeError(ServerHandshakeError),
    DotEnvError(dotenv::Error),
    ReqwestError(reqwest::Error),
    IoError(std::io::Error),
    TungsteniteError(TokioTungError),
    FutureTrySendMessageError(FutureSendMessage),
    FutureTrySendForwarderError(FutureSendForwarder),
    JoinError(JoinError),
    ReceiverGiver,
    SerdeJsonError(serde_json::Error),
    Unknown,
}

impl From<serde_json::Error> for DoxMeDaddyError {
    fn from(err: serde_json::Error) -> Self {
        return DoxMeDaddyError::SerdeJsonError(err);
    }
}

impl From<JoinError> for DoxMeDaddyError {
    fn from(err: JoinError) -> Self {
        return DoxMeDaddyError::JoinError(err);
    }
}

impl From<TokioTungError> for DoxMeDaddyError {
    fn from(err: TokioTungError) -> Self {
        return DoxMeDaddyError::TungsteniteError(err);
    }
}

impl From<FutureSendForwarder> for DoxMeDaddyError {
    fn from(err: FutureSendForwarder) -> Self {
        return DoxMeDaddyError::FutureTrySendForwarderError(err);
    }
}

impl From<FutureSendMessage> for DoxMeDaddyError {
    fn from(err: FutureSendMessage) -> Self {
        return DoxMeDaddyError::FutureTrySendMessageError(err);
    }
}

impl From<std::io::Error> for DoxMeDaddyError {
    fn from(err: std::io::Error) -> Self {
        return DoxMeDaddyError::IoError(err);
    }
}

impl From<ServerHandshakeError> for DoxMeDaddyError {
    fn from(err: ServerHandshakeError) -> Self {
        return DoxMeDaddyError::ServerHandshakeError(err);
    }
}

impl From<dotenv::Error> for DoxMeDaddyError {
    fn from(err: dotenv::Error) -> Self {
        return DoxMeDaddyError::DotEnvError(err);
    }
}

impl From<reqwest::Error> for DoxMeDaddyError {
    fn from(err: reqwest::Error) -> Self {
        return DoxMeDaddyError::ReqwestError(err);
    }
}
