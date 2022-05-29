use pyo3::prelude::*;
use pyo3::exceptions::PyConnectionError;
use std::time::Duration;
use std::net::{SocketAddr, TcpStream};
use std::io::{Read, Write};
use rdp::core::client::{RdpClient, Connector};
use rdp::model::error::{Error, RdpErrorKind, RdpError, RdpResult};
use rdp::core::gcc::KeyboardLayout;


/// Create a tcp stream from main args
fn tcp_from_args(ip: &str, port: &str) -> RdpResult<TcpStream> {
    // TCP connection
    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>().map_err( |e| {
        Error::RdpError(RdpError::new(RdpErrorKind::InvalidData, &format!("Cannot parse the IP PORT input [{}]", e)))
    })?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::new(5, 0))?;
    tcp.set_nodelay(true).map_err(|e| {
        Error::RdpError(RdpError::new(RdpErrorKind::InvalidData, &format!("Unable to set no delay option [{}]", e)))
    })?;

    Ok(tcp)
}

/// Create rdp client from args
fn rdp_from_args<S: Read + Write>(username: &str, password: &str, stream: S) -> RdpResult<RdpClient<S>> {

    let width = 1920;
    let height = 1080;
    let domain = "";
    let name = "freerdp";
    // let ntlm_hash = None;
    let restricted_admin_mode = false;
    let layout = KeyboardLayout::US;
    let auto_logon = false;
    let blank_creds = false;
    let check_certificate = false;
    let use_nla = true;

    let mut rdp_connector =  Connector::new()
        .screen(width, height)
        .credentials(domain.to_string(), username.to_string(), password.to_string())
        .set_restricted_admin_mode(restricted_admin_mode)
        .auto_logon(auto_logon)
        .blank_creds(blank_creds)
        .layout(layout)
        .check_certificate(check_certificate)
        .name(name.to_string())
        .use_nla(use_nla);

    // if let Some(hash) = ntlm_hash {
    //     rdp_connector = rdp_connector.set_password_hash(hex::decode(hash).map_err(|e| {
    //         Error::RdpError(RdpError::new(RdpErrorKind::InvalidData, &format!("Cannot parse the input hash [{}]", e)))
    //     })?)
    // }
    // RDP connection
    Ok(rdp_connector.connect(stream)?)
}

#[pyfunction]
fn check_connection(ip: &str, port: &str, username: &str, password: &str) -> PyResult<()> {
    // Create a tcp stream from args
    match tcp_from_args(ip, port) {
        Ok(tcp) => {
            // Create rdp client
            match rdp_from_args(username, password, tcp) {
                Ok(_) => Ok(()),
                Err(_) => Err(PyConnectionError::new_err("rdp connect failed"))
            }
        },
        Err(_) => Err(PyConnectionError::new_err("tcp connect failed"))
    }
}

#[pymodule]
fn rdp_rspy(_py: Python<'_>, m:&PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_connection, m)?)?;
    Ok(())
}
