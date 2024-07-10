use crate::{
    message::*, new_connections_receiver, new_orders_receiver, next_robot_receiver,
    prev_robot_receiver, token_box::TokenBox,
};
use actix::prelude::*;
use ice_cream_shop::{
    flavour::Flavour,
    id_to_addr,
    messages::{fmt_msg, robot_msg::RobotMsg, screen_msg::ScreenMsg},
    orders::Order,
    shop_values::{
        N_ROBOTS, N_SCREEN, ROBOT_SCREEN_STARTING_PORT, ROBOT_STARTING_PORT, SCREEN_STARTING_PORT,
        STARTING_ICECREAM,
    },
    tokens::{FlavourToken, OrderToken, TokenId},
};
use std::future::Future;
use tokio::{
    io::{self, AsyncWriteExt, WriteHalf},
    net::{TcpListener, TcpStream},
    task::{self, JoinHandle},
    time::{self, Duration},
};

/// The `Robot` struct represents a robot that is part of a token ring network of robots.
/// It contains the following fields:
/// - `id`: The ID of the robot.
/// - `prev_id`: The ID of the previous robot in the ring.
/// - `next_id`: The ID of the next robot in the ring.
/// - `prev_tx`: The write half of the TCP stream to the previous robot.
/// - `next_tx`: The write half of the TCP stream to the next robot.
/// - `new_orders`: A list of new orders received by the robot.
/// - `current_order`: The current order being served by the robot.
/// - `serving_flavour`: A flag indicating if the robot is currently serving an ice cream flavour.
/// - `token_box`: A token box containing the order and flavour tokens that the next robot has not finished using.
#[derive(Debug, Default)]
pub struct Robot {
    id: u16,
    prev_id: Option<u16>,
    next_id: Option<u16>,
    prev_tx: Option<WriteHalf<TcpStream>>,
    next_tx: Option<WriteHalf<TcpStream>>,
    new_orders: Vec<Order>,
    current_order: Option<Order>,
    serving_flavour: bool,
    token_box: TokenBox,
}

/// The `Robot` struct implements the `Actor` trait.
/// It provides a context for the actor.
impl Actor for Robot {
    type Context = Context<Self>;
}

impl Robot {
    /// Initializes a new Robot with the given id.
    fn new(id: u16) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Spawns a new robot with the given ID.
    /// It spawns two tasks that will run in parallel:
    /// - A task that listens for new connections.
    /// - A task that listens for new orders.
    /// It also sends a `FindNext` message to the robot.
    /// If the robot is alone, it sends the necessary messages to intialize the token ring with the appropriate tokens.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the robot.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `JoinHandle` for the spawned tasks that will run in parallel.
    pub async fn spawn(id: u16) -> Result<JoinHandle<()>, &'static str> {
        let ip = id_to_addr(ROBOT_STARTING_PORT, id);
        let err = "Coudn't connect to my reserved robot ip address";
        let new_con_listener = TcpListener::bind(ip).await.map_err(|_| err)?;

        let ip = id_to_addr(ROBOT_SCREEN_STARTING_PORT, id);
        let err = "Couldn't connect to my reserved robot_screen ip address";
        let new_orders_listener = TcpListener::bind(ip).await.map_err(|_| err)?;

        let addr = Self::new(id).start();
        task::spawn(new_orders_receiver(addr.clone(), new_orders_listener));
        let new_con_recv = task::spawn(new_connections_receiver(addr.clone(), new_con_listener));

        let err = "Couldn't send FindNext";
        addr.send(FindNext).await.map_err(|_| err)?;

        let err = "Couldn't determine if I was alone";
        if addr.send(IsAlone).await.map_err(|_| err)? {
            let token = OrderToken::new(id);
            addr.do_send(RecvOrderToken { token });

            for flavour in Flavour::flavours() {
                let token = FlavourToken::new(id, flavour, STARTING_ICECREAM);
                addr.do_send(RecvFlavourToken { token });
            }
        }

        Ok(new_con_recv)
    }

    /// Disconnects from the previous robot.
    ///
    /// # Returns
    ///
    /// A future that resolves when the disconnection is complete.
    fn disconnect_prev(&mut self) -> impl Future<Output = ()> {
        let prev_tx = self.prev_tx.take();
        self.prev_id = None;

        async move {
            if let Some(mut tx) = prev_tx {
                let msg = fmt_msg(RobotMsg::Disconnect);
                let _ = tx.write_all(&msg).await;
            }
        }
    }

    /// Sends a token to the next robot.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token.
    /// * `msg` - The message to send.
    ///
    /// # Returns
    ///
    /// A future that resolves when the token is sent.
    /// It returns a tuple containing the previous and next write halves.
    /// If the next write half is `None`, it returns the previous write half.
    fn send_token(
        &mut self,
        token_id: TokenId,
        msg: RobotMsg,
    ) -> impl Future<
        Output = Result<
            (Option<WriteHalf<TcpStream>>, Option<WriteHalf<TcpStream>>),
            Option<WriteHalf<TcpStream>>,
        >,
    > {
        let mut next_tx = self.next_tx.take();
        let mut prev_tx = self.prev_tx.take();

        async move {
            if let Some(tx) = next_tx.as_mut() {
                let recv_token = fmt_msg(msg);
                if tx.write_all(&recv_token).await.is_err() {
                    return Err(prev_tx);
                }

                if let Some(tx) = prev_tx.as_mut() {
                    let eou = fmt_msg(RobotMsg::EndOfUse(token_id));
                    let _ = tx.write_all(&eou).await;
                }
            }

            Ok((prev_tx, next_tx))
        }
    }

    /// Sends a screen message to the screen with the given ID.
    ///
    /// # Arguments
    ///
    /// * `msg` - The screen message to send.
    ///
    /// # Returns
    ///
    /// A future that resolves when the message is sent.
    fn send_screen(&self, msg: ScreenMsg, id: u16) -> impl Future<Output = ()> {
        async move {
            let msg = fmt_msg(msg);
            for screen in (0..N_SCREEN).map(|i| (i + id) % N_SCREEN) {
                let ip = id_to_addr(SCREEN_STARTING_PORT, screen);
                if let Ok(mut stream) = TcpStream::connect(ip).await {
                    if stream.write_all(&msg).await.is_ok() {
                        break;
                    }
                }
            }
        }
    }

    /// Returns an iterator over the intermediate robot IDs.
    ///
    /// # Returns
    ///
    /// An iterator over the intermediate robot IDs.
    fn intermediate_ids(&self) -> impl Iterator<Item = u16> {
        let robot_id = self.id;
        let next_id = self.next_id;

        (1..=N_ROBOTS)
            .map(move |offset| (robot_id + offset) % N_ROBOTS)
            .take_while(move |&id| id != next_id.unwrap_or(robot_id))
    }

    /// If the robot is not currently serving an order, it downloads a now order from the token.
    /// If there is an uncompleted order from another robot, it recovers it.
    /// If there is no order in progress, it downloads the next order from the token.
    ///
    /// # Arguments
    ///
    /// * `token` - The order token.
    ///
    /// # Returns
    ///
    /// A future that resolves when the order is downloaded.
    fn download_current_order(&mut self, token: &mut OrderToken) {
        if self.current_order.is_none() {
            token.remove_in_progress(self.id);

            for id in self.intermediate_ids() {
                if let Some(order) = token.remove_in_progress(id) {
                    //println!("Recovered a lost order started by {id}");
                    token.add_in_progress(self.id, order.clone());
                    self.current_order = Some(order);
                    return;
                }
            }

            if let Some(order) = token.next_order() {
                token.add_in_progress(self.id, order.clone());
                self.current_order = Some(order);
            }
        }
    }

    /// Clears the current order.
    fn clear_order(&mut self) {
        self.current_order = None;
    }
}

/// Implements the handler trait for the `Robot` struct to handle the IsAlone message.
impl Handler<IsAlone> for Robot {
    type Result = bool;

    /// Handles the IsAlone message.
    /// It returns `true` if the robot is alone in the ring, `false` otherwise.
    fn handle(&mut self, _: IsAlone, _: &mut Self::Context) -> Self::Result {
        self.next_id.map(|next| next == self.id).unwrap_or_default()
    }
}

/// Implements the handler trait for the `Robot` struct to handle the Connect message.
impl Handler<Connect> for Robot {
    type Result = ();

    /// Handles the Connect message.
    /// It disconnects from the previous robot and connects to the new robot.
    /// It also spawns a task to listen for messages from the new robot.
    ///
    /// # Arguments
    ///
    /// * `msg` - The Connect message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the connection is complete.
    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        self.disconnect_prev().into_actor(self).wait(ctx);

        let (rx, tx) = io::split(msg.stream);
        self.prev_tx = Some(tx);

        task::spawn(prev_robot_receiver(ctx.address(), rx));
    }
}

/// Implements the handler trait for the `Robot` struct to handle the FindNext message.
impl Handler<FindNext> for Robot {
    type Result = ();

    /// Handles the FindNext message.
    /// It tries to connect to the next robot in the ring.
    /// If it succeeds, it spawns a task to listen for new connections.
    /// If it fails, it tries to connect to the next robot in the ring and repeats the process.
    ///
    /// # Arguments
    ///
    /// * `msg` - The FindNext message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the next robot is found.
    fn handle(&mut self, _: FindNext, ctx: &mut Self::Context) -> Self::Result {
        let robot_id = self.id;
        let addr = ctx.address();

        async move {
            for offset in 1..=N_ROBOTS {
                let id = (robot_id + offset) % N_ROBOTS;
                let ip = id_to_addr(ROBOT_STARTING_PORT, id);

                if let Ok((rx, tx)) = TcpStream::connect(ip).await.map(io::split) {
                    tokio::spawn(next_robot_receiver(addr, rx));
                    return Ok((tx, id));
                }
            }

            Err("Couldn't find next robot")
        }
        .into_actor(self)
        .map(|res, robot, _| {
            if let Ok((tx, id)) = res {
                robot.next_id = Some(id);
                robot.next_tx = Some(tx);
            }
        })
        .wait(ctx);
    }
}

/// Implements the handler trait for the `Robot` struct to handle the RecvOrderToken message.
impl Handler<RecvOrderToken> for Robot {
    type Result = ();

    /// Handles the RecvOrderToken message.
    /// It receives an order token and processes it.
    /// If the robot is not currently serving an order, it downloads a new order from the token.
    /// It checks if the current order is completed and sends a confirmation message to the screen in case it is.
    /// It also uploads new orders to the token.
    ///
    /// # Arguments
    ///
    /// * `msg` - The RecvOrderToken message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the order token is processed.
    fn handle(&mut self, msg: RecvOrderToken, ctx: &mut Self::Context) -> Self::Result {
        let mut token = msg.token;

        self.prev_id = Some(token.sender());
        token.mark(self.id);

        println!(
            "me: {}, prev: {:?}, next: {:?}",
            self.id, self.prev_id, self.next_id
        );

        //println!("Recibí el order token de  {:?}", self.prev_id);

        token.upload_new_orders(self.new_orders.drain(..));

        if !self.serving_flavour {
            if let Some(order) = self.current_order.as_mut() {
                if order.is_completed() {
                    let order_id = order.id();
                    let msg = ScreenMsg::ConfirmOrder(order.id());

                    self.clear_order();
                    self.send_screen(msg, order_id.screen_id())
                        .into_actor(self)
                        .spawn(ctx);
                }
            }
        }

        self.download_current_order(&mut token);
        ctx.address().do_send(ReleaseOrderToken { token });
    }
}

/// Implements the handler trait for the `Robot` struct to handle the RecvFlavourToken message.
impl Handler<ReleaseOrderToken> for Robot {
    type Result = ();

    /// Handles the ReleaseOrderToken message.
    /// It releases the order token and sends it to the next robot.
    ///
    /// # Arguments
    ///
    /// * `msg` - The ReleaseOrderToken message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the order token is released.
    fn handle(&mut self, msg: ReleaseOrderToken, ctx: &mut Self::Context) -> Self::Result {
        let token = msg.token;
        self.token_box.stash_order_token(token.clone());

        //println!("Envío el order token a    {:?}", self.next_id);

        self.send_token(token.id(), RobotMsg::RecvOrderToken(token))
            .into_actor(self)
            .map(|res, robot, _| match res {
                Ok((prev_tx, next_tx)) => {
                    robot.prev_tx = prev_tx;
                    robot.next_tx = next_tx;
                }

                Err(prev_tx) => robot.prev_tx = prev_tx,
            })
            .wait(ctx);
    }
}

/// Implements the handler trait for the `Robot` struct to handle the RecvFlavourToken message.
impl Handler<RecvFlavourToken> for Robot {
    type Result = ResponseActFuture<Self, ()>;

    /// Handles the RecvFlavourToken message.
    /// It receives a flavour token and processes it.
    /// If the robot is not currently serving a flavour, it checks if the current order requires the flavour and serves it.
    ///
    /// # Arguments
    ///
    /// * `msg` - The RecvFlavourToken message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the flavour token is processed.
    fn handle(&mut self, msg: RecvFlavourToken, ctx: &mut Self::Context) -> Self::Result {
        let mut token = msg.token;
        let mut duration = 0;

        /* println!(
            "flavour_token: {}\t{:?}   \t{}",
            token.sender(),
            token.flavour(),
            token.servings()
        ); */

        self.prev_id = Some(token.sender());
        token.mark(self.id);

        if !self.serving_flavour {
            if let Some(order) = self.current_order.as_mut() {
                if let Some(servings) = order.cross(token.flavour()) {
                    if !token.has_enough(servings) {
                        let order_id = order.id();
                        let msg = ScreenMsg::CancelOrder(order_id);

                        self.clear_order();
                        self.send_screen(msg, order_id.screen_id())
                            .into_actor(self)
                            .spawn(ctx);
                    } else {
                        duration = token.take(servings);
                        self.serving_flavour = true;
                    }
                }
            }
        }

        self.token_box.discard_flavour_token(token.flavour());
        let fut = time::sleep(Duration::from_secs(duration as u64))
            .into_actor(self)
            .map(move |_, robot, ctx| {
                if duration > 0 {
                    robot.serving_flavour = false;
                    println!("Took {duration} seconds to serve {:?}", token.flavour());
                }

                ctx.address().do_send(ReleaseFlavourToken { token })
            });

        Box::pin(fut)
    }
}

/// Implements the handler trait for the `Robot` struct to handle the ReleaseFlavourToken message.
impl Handler<ReleaseFlavourToken> for Robot {
    type Result = ();

    /// Handles the ReleaseFlavourToken message.
    /// It releases the flavour token and sends it to the next robot.
    ///
    /// # Arguments
    ///
    /// * `msg` - The ReleaseFlavourToken message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the flavour token is released.
    fn handle(&mut self, msg: ReleaseFlavourToken, ctx: &mut Self::Context) -> Self::Result {
        let token = msg.token;
        self.token_box.stash_flavour_token(token.clone());

        self.send_token(token.id(), RobotMsg::RecvFlavourToken(token))
            .into_actor(self)
            .map(|res, robot, _| match res {
                Ok((prev_tx, next_tx)) => {
                    robot.prev_tx = prev_tx;
                    robot.next_tx = next_tx;
                }

                Err(prev_tx) => robot.prev_tx = prev_tx,
            })
            .wait(ctx);
    }
}

/// Implements the handler trait for the `Robot` struct to handle the ReleaseOrderToken message.
impl Handler<EndOfUse> for Robot {
    type Result = ();

    /// Handles the EndOfUse message.
    /// It discards the token if the previous and next IDs are different.
    ///
    /// # Arguments
    ///
    /// * `msg` - The EndOfUse message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the token is discarded.
    fn handle(&mut self, msg: EndOfUse, _: &mut Self::Context) -> Self::Result {
        if self.prev_id != self.next_id {
            match msg.token_id {
                TokenId::Order => self.token_box.discard_order_token(),
                TokenId::Flavour(flavour) => self.token_box.discard_flavour_token(flavour),
            }
        }
    }
}

/// Implements the handler trait for the `Robot` struct to handle the RecvOrder message.
impl Handler<CheckTokenBox> for Robot {
    type Result = ();

    /// Handles the CheckTokenBox message.
    /// It checks the token box for any tokens and releases them.
    /// This will be called when the robot detects the next robot has been disconnected.
    ///
    /// # Arguments
    ///
    /// * `msg` - The CheckTokenBox message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the tokens are released.
    fn handle(&mut self, _: CheckTokenBox, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();

        if let Some(token) = self.token_box.take_order_token() {
            addr.do_send(ReleaseOrderToken { token });
        }

        for token in self.token_box.take_flavour_tokens() {
            addr.do_send(ReleaseFlavourToken { token });
        }
    }
}

/// Implements the handler trait for the `Robot` struct to handle the RecvOrder message.
impl Handler<RecvOrder> for Robot {
    type Result = ();

    /// Handles the RecvOrder message.
    /// It receives a new order and adds it to the list of new orders.
    ///
    /// # Arguments
    ///
    /// * `msg` - The RecvOrder message.
    /// * `ctx` - The context of the actor.
    ///
    /// # Returns
    ///
    /// A future that resolves when the order is received.
    fn handle(&mut self, msg: RecvOrder, _ctx: &mut Self::Context) -> Self::Result {
        self.new_orders.push(msg.order);
    }
}
