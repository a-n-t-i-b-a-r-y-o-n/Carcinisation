use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    static ref ASSIGNED_PORTS: Mutex<Vec<u32>> = Mutex::new(vec!());
}

const FIRST_PORT: u32 = 9001;

pub struct PortService;

impl PortService {
    
    pub fn request_port_assignment() -> Result<u32> {

        let mut assigned_ports = ASSIGNED_PORTS.lock();

        // Assign first port
        if assigned_ports.is_empty() {

            assigned_ports.push(FIRST_PORT);

            Ok(FIRST_PORT)
        } else {

            // Find next available port
            for port in FIRST_PORT..u32::MAX {

                if assigned_ports.contains(&port) {
                    continue;
                }

                assigned_ports.push(port);

                return Ok(port)
            }

            Err(anyhow!("No more available ports on the system"))
        }
        
    }
    
}