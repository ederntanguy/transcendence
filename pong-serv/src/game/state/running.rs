use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::game::combined_send::CombinedSend;
use crate::game::engine::{
    bounce_off_horizontal_edges, bounce_off_pads, side_of_ball_collision_with_wall,
    ServiceGenerator,
};
use crate::game::Side;
use crate::protocol::constants::{
    BALL_MOVEMENT_PER_TICK, BALL_RADIUS, MAX_CLIENT_UPDATES_PER_SECOND, PAD_HEIGHT,
    PAD_MOVEMENT_PER_TICK, RATIO, TICKS_PER_SECOND,
};
use crate::protocol::{
    parse_gm0_input_message, parse_gm1_input_message, GameAbortedMessage, GameCompletedMessage,
    PointScoredMessage, PositionUpdateMessage, ServerToClientMessage,
};

use super::done::WinType;
use super::GameResult;

/// This structure encapsulates the Pong game state : elements, score and service side.
#[derive(Clone)]
pub struct RunningState {
    ball_x: f64,
    ball_y: f64,
    angle: f64,
    l_pad_y: f64,
    r_pad_y: f64,
    service_side: Side,
    service_generator: ServiceGenerator,
    scores: [u32; 2],
}

/// Run a game loop using [`RunningState`] until either the remote game is completed or a client disconnects.
/// The latter is not a program error - it is simply handled as a [`WinType::Withdrawal`].
pub(super) async fn run_game_0_loop<S>(
    pl_ws: &mut WebSocketStream<S>,
    pr_ws: &mut WebSocketStream<S>,
    mut rs: RunningState,
) -> GameResult
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (mut l_pad_dy, mut r_pad_dy) = (0, 0);

    let mut tick_interval = tokio::time::interval(Duration::from_millis(1000 / TICKS_PER_SECOND));

    let game_result = loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let (next_state, message) = rs.update_on_tick(
                    &mut rand::thread_rng(),
                    l_pad_dy.into(),
                    r_pad_dy.into(),
                );
                rs = match next_state {
                    UpdateOutcome::Continue(rs) => rs,
                    UpdateOutcome::Done(res) => break res,
                };
                if let Err((_, side)) = CombinedSend::new(
                    pl_ws, pr_ws, Message::Binary(message.into())
                ).await {
                    break GameResult::new(rs.end_game(), !side, WinType::Withdrawal);
                }
            }
            first_msg = pl_ws.next() => {
                l_pad_dy = match parse_gm0_input_message(first_msg) {
                    Ok(new_dy) => new_dy.unwrap_or(l_pad_dy),
                    Err(_) => break GameResult::new(
                        rs.end_game(), Side::Right, WinType::Withdrawal
                    ),
                };
            }
            executor_msg = pr_ws.next() => {
                r_pad_dy = match parse_gm0_input_message(executor_msg) {
                    Ok(new_dy) => new_dy.unwrap_or(r_pad_dy),
                    Err(_) => break GameResult::new(
                        rs.end_game(), Side::Left, WinType::Withdrawal
                    ),
                };
            }
        }
    };
    send_result_message(pl_ws, pr_ws, &game_result).await;
    game_result
}

/// Run a game loop using [`RunningState`] until the game is completed or the client disconnects.
pub(super) async fn run_game_1_loop<S>(connection: &mut WebSocketStream<S>, mut rs: RunningState)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (mut l_pad_dy, mut r_pad_dy) = (0, 0);

    let mut tick_interval = tokio::time::interval(Duration::from_millis(1000 / TICKS_PER_SECOND));
    let mut to = interval_for_next_to();
    let mut to_active = false;

    let game_result = loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let (next_state, message) = rs.update_on_tick(
                    &mut rand::thread_rng(),
                    l_pad_dy.into(),
                    r_pad_dy.into(),
                );
                rs = match next_state {
                    UpdateOutcome::Continue(rs) => rs,
                    UpdateOutcome::Done(res) => break res,
                };
                if let Err(_) = connection.send(Message::Binary(message.into())).await {
                    return;
                }
            }
            _ = to.tick(), if to_active => {
                to_active = false;
            }
            msg = connection.next(), if !to_active => {
                to = interval_for_next_to();
                (l_pad_dy, r_pad_dy) = match parse_gm1_input_message(msg) {
                    Ok(Some((left_movement, right_movement))) => (left_movement, right_movement),
                    Ok(None) => (l_pad_dy, r_pad_dy),
                    Err(_) => return,
                };
                to_active = true;
            }
        }
    };
    let _: Result<_, _> = connection
        .send(Message::Binary(
            GameCompletedMessage::new(game_result.winner).into(),
        ))
        .await;
}

/// Make an interval that will tick at the moment the timeout is lifted.
fn interval_for_next_to() -> tokio::time::Interval {
    tokio::time::interval_at(
        Instant::now() + Duration::from_millis(1000 / MAX_CLIENT_UPDATES_PER_SECOND),
        Duration::from_millis(1000), // Those subsequent ticks are not used
    )
}

/// Send the appropriate end-of-game message to the appropriate client(s).
async fn send_result_message<S>(
    pl_ws: &mut WebSocketStream<S>,
    pr_ws: &mut WebSocketStream<S>,
    game_result: &GameResult,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    match game_result.win_type {
        WinType::ScoreReached => {
            let bytes = Vec::from(GameCompletedMessage::new(game_result.winner));
            let _: Result<_, _> = tokio::try_join!(
                pl_ws.send(Message::Binary(bytes.clone())),
                pr_ws.send(Message::Binary(bytes)),
            );
        }
        WinType::Withdrawal => {
            let bytes = GameAbortedMessage::new().into();
            let _: Result<_, _> = match game_result.winner {
                Side::Left => pl_ws,
                Side::Right => pr_ws,
            }
            .send(Message::Binary(bytes))
            .await;
        }
    }
}

enum UpdateOutcome {
    Continue(RunningState),
    Done(GameResult),
}

impl RunningState {
    const INITIAL_BALL_X: f64 = RATIO / 2.0 - BALL_RADIUS;
    const INITIAL_BALL_Y: f64 = 1.0 / 2.0 - BALL_RADIUS;

    /// Creates a [`RunningState`] instance with the elements positioned for game start, scores set at 0 and a random
    /// side for the first service.
    pub(super) fn new<R: Rng + ?Sized>(rng: &mut R) -> RunningState {
        let initial_side = rng.gen();
        let service_generator = ServiceGenerator::new();
        RunningState {
            ball_x: Self::INITIAL_BALL_X,
            ball_y: Self::INITIAL_BALL_Y,
            angle: service_generator.gen_angle(initial_side, rng),
            l_pad_y: (1.0 - PAD_HEIGHT) / 2.0,
            r_pad_y: (1.0 - PAD_HEIGHT) / 2.0,
            service_side: initial_side,
            service_generator,
            scores: [0, 0],
        }
    }

    /// Abort the game early. Return the current score.
    pub(super) fn end_game(self) -> [u32; 2] {
        self.scores
    }

    /// Updates the elements' positions by their movement per tick. No collision check is done here.
    fn move_elements(&mut self, l_pad_dy: f64, r_pad_dy: f64) {
        self.l_pad_y = f64::clamp(
            self.l_pad_y + l_pad_dy * PAD_MOVEMENT_PER_TICK,
            0.0,
            1.0 - PAD_HEIGHT,
        );
        self.r_pad_y = f64::clamp(
            self.r_pad_y + r_pad_dy * PAD_MOVEMENT_PER_TICK,
            0.0,
            1.0 - PAD_HEIGHT,
        );
        self.ball_x += BALL_MOVEMENT_PER_TICK * f64::cos(self.angle);
        self.ball_y -= BALL_MOVEMENT_PER_TICK * f64::sin(self.angle);
    }

    /// Sets the elements positions' back at their appropriate initial positions.
    fn reset_elements<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        (self.ball_x, self.ball_y) = (Self::INITIAL_BALL_X, Self::INITIAL_BALL_Y);
        self.service_side = !self.service_side;
        self.angle = self.service_generator.gen_angle(self.service_side, rng);
    }

    fn update_on_tick<R>(
        mut self,
        rng: &mut R,
        l_pad_dy: f64,
        r_pad_dy: f64,
    ) -> (UpdateOutcome, ServerToClientMessage)
    where
        Self: Sized,
        R: Rng + ?Sized,
    {
        //Update the state
        self.move_elements(l_pad_dy, r_pad_dy);

        //Check for ball-pad collision before checking for end-of-game, to have high save moments
        (self.ball_x, self.ball_y, self.angle) = bounce_off_pads(
            self.ball_x,
            self.ball_y,
            self.angle,
            self.l_pad_y,
            self.r_pad_y,
        );

        if let Some(out_side) = side_of_ball_collision_with_wall(self.ball_x) {
            //Ball is out, update scores
            let win_side = !out_side;
            self.scores[u8::from(win_side) as usize] += 1;
            if self.scores[u8::from(win_side) as usize] != 10 {
                //10 points not reached, play another round
                self.reset_elements(rng);
                let message = PointScoredMessage::new(
                    win_side,
                    self.l_pad_y,
                    self.r_pad_y,
                    self.ball_x,
                    self.ball_y,
                );
                (
                    UpdateOutcome::Continue(self),
                    ServerToClientMessage::PointScored(message),
                )
            } else {
                //End the game
                let message = GameCompletedMessage::new(win_side);
                (
                    UpdateOutcome::Done(GameResult::new(
                        self.scores,
                        win_side,
                        WinType::ScoreReached,
                    )),
                    ServerToClientMessage::GameDone(message),
                )
            }
        } else {
            //Ball is in, keep playing
            (self.ball_y, self.angle) = bounce_off_horizontal_edges(self.ball_y, self.angle);
            let message =
                PositionUpdateMessage::new(self.l_pad_y, self.r_pad_y, self.ball_x, self.ball_y);
            (
                UpdateOutcome::Continue(self),
                ServerToClientMessage::PositionUpdate(message),
            )
        }
    }
}
