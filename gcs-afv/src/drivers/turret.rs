use afv_internal::{
    network::InternalMessage, stepper, PAN_STEPPER_STEPS_REV, SOCKET_MSG_SIZE,
    TILT_STEPPER_STEPS_REV,
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::broadcast,
    time::{sleep, Duration},
};

use crate::network::{
    scanner::{ScanBuilder, ScanCount},
    socket::Socket,
    NetMessage,
};

pub const POLL_STEPS_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TurretDriverMessage {
    SetAngleChange(u16, [f32; 2]),
    Angle(u16, [f32; 2]),
    SetAbsoluteAngle(u16, [f32; 2]),
}

#[derive(Clone)]
/// The TurretDriver is the struct that connects to a particular port addressed turret on the AFV
/// and sends command to the arduino running the turret's control firmware.
///
/// Port addressing in this sense means that each "Turret" that is run on an Arduino starts its own TCP server
/// on a specific port. This means that no matter what IP address/Arduino a specific turret is run on it can still be 
/// found automatically.
pub struct TurretDriver {
    port: u16,
    net_tx: broadcast::Sender<NetMessage>,
    turret_socket: Socket,
}

impl TurretDriver {
    /// This functon creates a new turret targeting a specifc port and adds it to the main bus.
    ///
    /// * `port` - The target turret port
    pub async fn new(net_tx: broadcast::Sender<NetMessage>, port: u16) -> Option<Self> {
        let turret_socket = match ScanBuilder::default()
            .scan_count(ScanCount::Infinite)
            .add_port(port)
            .dispatch()
            .recv_async()
            .await
        {
            Ok(stream) => Socket::new(stream, false),
            Err(_) => return None,
        };

        // let stream = match TcpStream::connect((Ipv4Addr::new(192,168,4,20), port)).await{
        //     Ok(s) => s,
        //     _ => return None
        // };
        // let turret_socket = Socket::new(stream, false);

        info!("Turret {} connected to MCU", port);

        let turret = Self {
            port,
            net_tx,
            turret_socket,
        };

        tokio::spawn(turret.clone().forward_messages_task());
        tokio::spawn(turret.clone().poll_steps_task());
        tokio::spawn(turret.clone().set_steps_task());

        Some(turret)
    }

    /// This task is responsible for forwarding tasks sent from the host target turret to the main bus
    async fn forward_messages_task(self) {
        loop {
            let mut data = [0u8; SOCKET_MSG_SIZE];
            for i in 0..data.len() {
                data[i] = self.turret_socket.read_byte().await;
            }

            match InternalMessage::from_msg(&data) {
                Some(InternalMessage::Turret(afv_internal::turret::TurretMsg::Steps((
                    pan_steps,
                    tilt_steps,
                )))) => {
                    let pan_angle = stepper::convert_steps_angle(pan_steps, PAN_STEPPER_STEPS_REV);
                    let tilt_angle =
                        stepper::convert_steps_angle(tilt_steps, TILT_STEPPER_STEPS_REV);
                    let _ = self
                        .net_tx
                        .send(NetMessage::TurretDriver(TurretDriverMessage::Angle(
                            self.port,
                            [pan_angle, tilt_angle],
                        )));
                    println!("Steps {:?}", (pan_steps, tilt_steps));
                }
                Some(InternalMessage::Ping(val)) => {
                    println!("Pinged {}", val);
                }
                Some(InternalMessage::Turret(afv_internal::turret::TurretMsg::PollSteps)) => {
                    println!("polled");
                }
                _ => {}
            }
        }
    }

    /// This task is responsible for frequenlty poll the current step that the target turret is at
    async fn poll_steps_task(self) {
        loop {
            sleep(POLL_STEPS_INTERVAL).await;
            debug!("Polling turret {} for steps", self.port);
            if let Some(msg) =
                InternalMessage::Turret(afv_internal::turret::TurretMsg::PollSteps).to_msg()
            {
                self.turret_socket.write_data(&msg).await;
            }
        }
    }

    /// This task send command from the main bus to the the target turret
    async fn set_steps_task(self) {
        let mut net_rx = self.net_tx.subscribe();
        loop {
            let pan_angle_change: f32;
            let tilt_angle_change: f32;

            let pan_angle: f32;
            let tilt_angle: f32;

            'outer: loop {
                if let Ok(NetMessage::TurretDriver(TurretDriverMessage::SetAngleChange(
                    port,
                    [pan_angle_change_request, tilt_angle_change_request],
                ))) = net_rx.recv().await
                {
                    if port != self.port {
                        continue;
                    }
                    pan_angle_change = pan_angle_change_request;
                    tilt_angle_change = tilt_angle_change_request;
                    break;
                }
                if let Ok(NetMessage::TurretDriver(TurretDriverMessage::SetAbsoluteAngle(
                    port,
                    [pan_angle_change_request, tilt_angle_change_request],
                ))) = net_rx.recv().await
                {
                    if port != self.port {
                        continue;
                    }
                    if let Some(msg) =
                        InternalMessage::Turret(afv_internal::turret::TurretMsg::SetSteps((
                            stepper::convert_angle_steps(
                                pan_angle_change_request,
                                PAN_STEPPER_STEPS_REV,
                            ),
                            stepper::convert_angle_steps(
                                tilt_angle_change_request,
                                TILT_STEPPER_STEPS_REV,
                            ),
                        )))
                        .to_msg()
                    {
                        self.turret_socket.write_data(&msg).await;
                    }
                    continue 'outer;
                }
            }

            loop {
                if let Ok(NetMessage::TurretDriver(TurretDriverMessage::Angle(
                    port,
                    [current_pan_angle, current_tilt_angle],
                ))) = net_rx.recv().await
                {
                    if port != self.port {
                        continue;
                    }
                    pan_angle = current_pan_angle;
                    tilt_angle = current_tilt_angle;
                    break;
                }
            }

            let new_pan_angle = pan_angle + pan_angle_change;
            let new_tilt_angle = tilt_angle + tilt_angle_change;

            error!(
                "Turret {} angle set to {} x {}",
                self.port, new_pan_angle, new_tilt_angle
            );

            if let Some(msg) = InternalMessage::Turret(afv_internal::turret::TurretMsg::SetSteps((
                stepper::convert_angle_steps(new_pan_angle, PAN_STEPPER_STEPS_REV),
                stepper::convert_angle_steps(new_tilt_angle, TILT_STEPPER_STEPS_REV),
            )))
            .to_msg()
            {
                self.turret_socket.write_data(&msg).await;
            }

            net_rx = self.net_tx.subscribe();
        }
    }
}
