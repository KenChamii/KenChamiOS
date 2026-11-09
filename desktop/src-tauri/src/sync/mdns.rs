use crate::error::{AppError, AppResult};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::net::{IpAddr, Ipv4Addr};

const SERVICE_TYPE: &str = "_kenchamios._tcp.local.";
const INSTANCE: &str = "KenChamiOS";
const HOST: &str = "kenchamios.local.";

fn mdns_error(error: mdns_sd::Error) -> AppError {
    AppError::Sync(format!("could not advertise on the network: {error}"))
}

/// Announces the sync server on the LAN. The record carries the protocol version only.
pub struct Advertisement(ServiceDaemon);

impl Advertisement {
    pub fn start(ip: Ipv4Addr, port: u16) -> AppResult<Self> {
        let daemon = ServiceDaemon::new().map_err(mdns_error)?;
        let info = ServiceInfo::new(
            SERVICE_TYPE,
            INSTANCE,
            HOST,
            IpAddr::V4(ip),
            port,
            &[("v", "1")][..],
        )
        .map_err(mdns_error)?;
        daemon.register(info).map_err(mdns_error)?;
        Ok(Self(daemon))
    }

    pub fn stop(&self) {
        // the daemon thread ends with the process anyway, a failed shutdown has nothing to undo
        let _ = self.0.shutdown();
    }
}
