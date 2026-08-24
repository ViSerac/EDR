use crate::collectors::network::raw_udp_data::RawUdpData;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedUdpTable, MIB_UDPTABLE_OWNER_PID, MIB_UDPROW_OWNER_PID, UDP_TABLE_OWNER_PID, MIB_UDP6TABLE_OWNER_PID, MIB_UDP6ROW_OWNER_PID ,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};

pub fn capture_udp_v4() -> Vec<RawUdpData> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedUdpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );
    }

    let mut buffer: Vec<u8> = vec![0; size as usize];

    let result = unsafe {
        GetExtendedUdpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        )
    };
    
    if result == 0 {
        let table_ptr = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
        let num_entries = unsafe { (*table_ptr).dwNumEntries };
        let mut raw_endpoints = Vec::new();

        let rows_ptr = unsafe {
            std::ptr::addr_of!((*table_ptr).table) as *const MIB_UDPROW_OWNER_PID
    };

        for i in 0..num_entries {
            let row = unsafe { *rows_ptr.add(i as usize) };

            let local_addr = IpAddr::V4(Ipv4Addr::from(row.dwLocalAddr.to_be()));
            let local_port = (row.dwLocalPort as u16).to_be();

            let raw = RawUdpData {
                local_addr,
                local_port,
                pid: row.dwOwningPid,
            };

            raw_endpoints.push(raw);
        }

        raw_endpoints
    } else {
        eprintln!("Error fetching UDP IPv4 table: {:?}", result);
        Vec::new()
    }
}

pub fn capture_udp_v6() -> Vec<RawUdpData> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedUdpTable(
            None,
            &mut size,
            false,
            AF_INET6.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );
    }

    let mut buffer: Vec<u8> = vec![0; size as usize];

    let result = unsafe {
        GetExtendedUdpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET6.0 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        )
    };
    
    if result == 0 {
        let table_ptr = buffer.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID;
        let num_entries = unsafe { (*table_ptr).dwNumEntries };
        let mut raw_endpoints = Vec::new();

        let rows_ptr = unsafe {
            std::ptr::addr_of!((*table_ptr).table) as *const MIB_UDP6ROW_OWNER_PID 
    };

        for i in 0..num_entries {
            let row = unsafe { *rows_ptr.add(i as usize) };

            let local_addr = IpAddr::V6(Ipv6Addr::from(row.ucLocalAddr));
            let local_port = (row.dwLocalPort as u16).to_be();

            let raw = RawUdpData {
                local_addr,
                local_port,
                pid: row.dwOwningPid,
            };

            raw_endpoints.push(raw);
        }

        raw_endpoints
    } else {
        eprintln!("Error fetching UDP IPv6 table: {:?}", result);
        Vec::new()
    }
}