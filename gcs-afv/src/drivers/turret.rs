use std::net::Ipv4Addr;

use afv_internal::{network::InternalMessage, SOCKET_MSG_SIZE, PAN_STEPPER_STEPS_REV, TILT_STEPPER_STEPS_REV, stepper::{StepperOps, self}, FLIR_TURRET_PORT};
use log::{trace, info, debug};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::{sleep, Duration}, net::TcpStream};

use crate::network::{NetMessage, socket::Socket, scanner::{ScanBuilder, ScanCount}};

pub const POLL_STEPS_INTERVAL: u64 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TurretDriverMessage{
    SetAngle(u16, [f32; 2]),
    Angle(u16, [f32; 2]),
}

#[derive(Clone)]
pub struct TurretDriver{
    port: u16,
    net_tx: broadcast::Sender<NetMessage>,
    turret_socket: Socket,
}

impl TurretDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>, port: u16) -> Option<Self>{
        // let turret_socket = match ScanBuilder::default().scan_count(ScanCount::Infinite).add_port(port).dispatch().recv(){
        //     Ok(stream) => {
        //         Socket::new(stream, false)
        //     },
        //     Err(_) => return None,
        // };

        let turret_socket = match TcpStream::connect((Ipv4Addr::new(192,168,4,20), FLIR_TURRET_PORT)).await{
            Ok(s) => Socket::new(s, false),
            Err(_) => {return None}
        };

        info!("Turret {} connected to MCU", port);

        let turret = Self{
            port,
            net_tx,
            turret_socket,
        };

        tokio::spawn(turret.clone().forward_messages_task());
        tokio::spawn(turret.clone().poll_steps_task());
        tokio::spawn(turret.clone().set_steps_task());


        Some(turret)
    }

    async fn forward_messages_task(self){
        loop{
            let mut data = [0u8; SOCKET_MSG_SIZE];
            for i in 0..data.len(){
                data[i] = self.turret_socket.read_byte().await;
            }

            match InternalMessage::from_msg(data){
                Some(InternalMessage::Turret(afv_internal::turret::TurretMsg::Steps((pan_steps, tilt_steps)))) => {
                    let pan_angle = stepper::convert_steps_angle(pan_steps, PAN_STEPPER_STEPS_REV);
                    let tilt_angle = stepper::convert_steps_angle(tilt_steps, TILT_STEPPER_STEPS_REV);
                    let _ = self.net_tx.send(NetMessage::TurretDriver(TurretDriverMessage::Angle(self.port, [pan_angle, tilt_angle])));
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

    async fn poll_steps_task(self){
        loop{
            sleep(Duration::from_secs(POLL_STEPS_INTERVAL)).await;
            debug!("Polling turret {} for steps", self.port);
            if let Some(msg) = InternalMessage::Turret(afv_internal::turret::TurretMsg::PollSteps).to_msg(){
                self.turret_socket.write_data(&msg).await;
            }
        }
    }
    
    async fn set_steps_task(self){
        let mut net_rx = self.net_tx.subscribe();
        loop{
            if let Ok(NetMessage::TurretDriver(TurretDriverMessage::SetAngle(port, [pan_angle, tilt_angle]))) = net_rx.recv().await{
                if !port == self.port{continue;}
                let pan_steps = stepper::convert_angle_steps(pan_angle, PAN_STEPPER_STEPS_REV);
                let tilt_steps = stepper::convert_angle_steps(tilt_angle, TILT_STEPPER_STEPS_REV);
                if let Some(msg) = InternalMessage::Turret(afv_internal::turret::TurretMsg::SetSteps(pan_steps, tilt_steps)).to_msg(){
                    self.turret_socket.write_data(&msg).await;
                }
            }
        }
    }
}
