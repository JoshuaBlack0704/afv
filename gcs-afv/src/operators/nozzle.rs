use afv_internal::{FLIR_TURRET_PORT, NOZZLE_TURRET_PORT};
use glam::Vec2;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::{network::NetMessage, drivers::{turret::TurretDriverMessage, lidar::LidarDriverMessage}};

use super::flir::FlirOperatorMessage;

pub const AUTO_TARGET_REQUEST_INTERVAL: u64 = 1;
pub const LOCK_ON_ANGLE: [f32; 2] = [3.0, 3.0];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum NozzleOperatorMessage{
    AutoTarget,
}

#[derive(Clone)]
pub struct NozzleOperator{
    net_tx: broadcast::Sender<NetMessage>,
}

impl NozzleOperator{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> NozzleOperator {
        let operator = Self{
            net_tx,
        };

        tokio::spawn(operator.clone().auto_target());

        operator
        
    }

    async fn auto_target(self){
        let mut net_rx = self.net_tx.subscribe();
        
        loop{

            // Receive auto target token
            loop {
                if let Ok(NetMessage::NozzleOperator(NozzleOperatorMessage::AutoTarget)) = net_rx.recv().await{
                    break;
                }
            }
            
            // Review analysis for lock
            loop{
                if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(Some(analysis)))) = net_rx.recv().await{
                    let [delta_x, delta_y] = analysis.angle_change;
                    if delta_x.abs() < LOCK_ON_ANGLE[0] && delta_y.abs() < LOCK_ON_ANGLE[0]{
                        break;
                    }
                }
            }

            let flir_pan_angle: f32;
            // Get flir angle
            loop{
                if let Ok(NetMessage::TurretDriver(TurretDriverMessage::Angle(FLIR_TURRET_PORT, [pan_angle, _]))) = net_rx.recv().await{
                    flir_pan_angle = pan_angle;
                    break;
                }
            }

            let lidar_distance: u32;
            // Get flir angle
            loop{
                if let Ok(NetMessage::LidarDriver(LidarDriverMessage::LidarDistanceCm(distance))) = net_rx.recv().await{
                    lidar_distance = distance;
                    break;
                }
            }


            let new_angles = Self::calculate_firing_solution(flir_pan_angle, lidar_distance as f32 / 100.0);

            let _ = self.net_tx.send(NetMessage::TurretDriver(TurretDriverMessage::SetAngle(NOZZLE_TURRET_PORT, new_angles)));

            net_rx = self.net_tx.subscribe();
        }
    }

    /// Should return the required pan, tilt angles for the nozzle turret
    fn calculate_firing_solution(flir_pan_angle: f32, lidar_distance_meter: f32) -> [f32; 2]{
        let target_dir = Vec2::new(flir_pan_angle.cos(), flir_pan_angle.sin());
        let mut target_coords = target_dir * lidar_distance_meter;
        
        let nozzle_turret_offset = Vec2::new(0.4572, 0.0);

        target_coords -= nozzle_turret_offset;
        
        let pan_angle = target_coords.angle_between(Vec2::X);

        // m/s
        let speed = 20.0;

        if let Some(tilt_angle) = Self::optimal_angle(lidar_distance_meter, 0.0, speed, 9.81){
            return [pan_angle, tilt_angle];
        }
        if let Some(tilt_angle) = Self::lob_angle(lidar_distance_meter, 0.0, speed, 9.81){
            return [pan_angle, tilt_angle];
        }

        return [pan_angle, 0.0];
    }
    
    
    fn optimal_angle(x: f32, y: f32, v0: f32, g: f32) -> Option<f32> {
    	let root = v0 * v0 * v0 * v0 - g * (g * x * x + 2.0 * y * v0 * v0);
    	if root < 0.0 {
    		return None;
    	}
    	let root = f32::sqrt(root);
    	let angle = f32::atan((v0 * v0 - root) / (g * x));
    	Some(angle)
    }

    fn lob_angle(x: f32, y: f32, v0: f32, g: f32) -> Option<f32> {
    	let root = v0 * v0 * v0 * v0 - g * (g * x * x + 2.0 * y * v0 * v0);
    	if root < 0.0 {
    		return None;
    	}
    	let root = f32::sqrt(root);
    	let angle = f32::atan((v0 * v0 + root) / (g * x));
    	Some(angle)
    }    
}