use futures_channel::mpsc::unbounded;

use tokio::task::JoinHandle;
use twitch_irc::login::{StaticLoginCredentials};
use twitch_irc::TwitchIRCClient;

use twitch_irc::{ClientConfig, SecureTCPTransport};


use crate::forwarder::{ReceiverGiverAsync, ForwarderEvent};
use crate::opts::ServerOpts;
use crate::{async_receiver_giver};

type FutureReceiver = futures_channel::mpsc::UnboundedReceiver<ForwarderEvent>;
type FutureSender = futures_channel::mpsc::UnboundedSender<ForwarderEvent>;

pub struct Twitch {
    pub join_handle: JoinHandle<()>,
    tx: FutureSender,
    rx: Option<FutureReceiver>,
    client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>
}

async_receiver_giver!(Twitch);
// TODO: simple_forwarder needs to be implemented
// simple_forwarder!(Twitch);

impl Twitch {

    pub async fn send_message(&mut self, s: String) -> Result<(), Box<dyn std::error::Error>> {
        self.client.privmsg("theprimeagen".to_string(), s).await?;
        return Ok(());
    }

    pub async fn new(_opts: &ServerOpts) -> Twitch {

        let login_name = std::env::var("OAUTH_NAME").expect("OAUTH_NAME is required for twitch client");
        let oauth_token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is required for twitch client");

        let config: ClientConfig<StaticLoginCredentials> = ClientConfig::new_simple(StaticLoginCredentials::new(
            login_name,
            Some(oauth_token),
        ));

        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        // join a channel
        client.join("theprimeagen".to_owned());

        let (tx, rx) = unbounded();
        let inner_tx = tx.clone();
        let join_handle = tokio::spawn(async move {
            loop {
                if let Some(message) = incoming_messages.recv().await {
                    inner_tx.unbounded_send(ForwarderEvent::TwitchMessageRaw(message)).expect("Never going to give you up");
                } else {
                    print!("LOOK AT ME FAIL");
                }
            }
        });

        return Twitch {
            join_handle,
            tx,
            rx: Some(rx),
            client,
        };
    }
}



