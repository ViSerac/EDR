use crate::models::tcp_connection_identity::Protocol;
use crate::collectors::network::raw_tcp_data::RawTcpData;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPTABLE_OWNER_PID, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL, MIB_TCP6TABLE_OWNER_PID, MIB_TCP6ROW_OWNER_PID,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};

pub fn capture_tcp_v4() -> Vec<RawTcpData> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
    }

    let mut buffer: Vec<u8> = vec![0; size as usize];

    let result = unsafe {
        GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        )
    };
    
    if result == 0 {
        let table_ptr = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let num_entries = unsafe { (*table_ptr).dwNumEntries };
        let mut raw_connections = Vec::new();

        let rows_ptr = unsafe {
            std::ptr::addr_of!((*table_ptr).table) as *const MIB_TCPROW_OWNER_PID
    };

        for i in 0..num_entries {
            let row = unsafe { *rows_ptr.add(i as usize) };

            let local_addr = IpAddr::V4(Ipv4Addr::from(row.dwLocalAddr.to_be()));
            let local_port = (row.dwLocalPort as u16).to_be();
            let remote_addr = IpAddr::V4(Ipv4Addr::from(row.dwRemoteAddr.to_be()));
            let remote_port = (row.dwRemotePort as u16).to_be();

            let raw = RawTcpData {
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                protocol: Protocol::Tcp,
                pid: row.dwOwningPid,
                state: Some(row.dwState),
            };

            raw_connections.push(raw);
        }

        raw_connections
    } else {
        eprintln!("Error fetching TCP IPv4 table: {:?}", result);
        Vec::new()
    }
}

pub fn capture_tcp_v6() -> Vec<RawTcpData> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
    }

    let mut buffer: Vec<u8> = vec![0; size as usize];

    let result = unsafe {
        GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        )
    };
    
    if result == 0 {
        let table_ptr = buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
        let num_entries = unsafe { (*table_ptr).dwNumEntries };
        let mut raw_connections = Vec::new();

        let rows_ptr = unsafe {
            std::ptr::addr_of!((*table_ptr).table) as *const MIB_TCP6ROW_OWNER_PID
    };

        for i in 0..num_entries {
            let row = unsafe { *rows_ptr.add(i as usize) };

            let local_addr = IpAddr::V6(Ipv6Addr::from(row.ucLocalAddr));
            let local_port = (row.dwLocalPort as u16).to_be();
            let remote_addr = IpAddr::V6(Ipv6Addr::from(row.ucRemoteAddr));
            let remote_port = (row.dwRemotePort as u16).to_be();

            let raw = RawTcpData {
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                protocol: Protocol::Tcp,
                pid: row.dwOwningPid,
                state: Some(row.dwState),
            };

            raw_connections.push(raw);
        }

        raw_connections
    } else {
        eprintln!("Error fetching TCP IPv6 table: {:?}", result);
        Vec::new()
    }
}