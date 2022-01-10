

use tokio_tungstenite::tungstenite::{handshake::server::NoCallback, ServerHandshake, HandshakeError, Message};



type ServerHandshakeError = HandshakeError<ServerHandshake<std::net::TcpStream, NoCallback>>;
type TokioTungError = tokio_tungstenite::tungstenite::Error;
type FutureSendError = futures_channel::mpsc::TrySendError<Message>;

#[derive(Debug)]
pub enum DoxMeDaddyError {
    ServerHandshakeError(ServerHandshakeError),
    DotEnvError(dotenv::Error),
    ReqwestError(reqwest::Error),
    IoError(std::io::Error),
    TungsteniteError(TokioTungError),
    FutureTrySendError(FutureSendError),
    Unknown
}

impl From<TokioTungError> for DoxMeDaddyError {
    fn from(err: TokioTungError) -> Self {
        return DoxMeDaddyError::TungsteniteError(err);
    }
}

impl From<FutureSendError> for DoxMeDaddyError {
    fn from(err: FutureSendError) -> Self {
        return DoxMeDaddyError::FutureTrySendError(err);
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
