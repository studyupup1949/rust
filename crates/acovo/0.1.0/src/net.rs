use anyhow::{anyhow, Result as AnyResult};
use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::{env, fs};
use tracing::*;

#[cfg(feature = "net")]
#[derive(Debug, Clone, Default)]
pub struct NetLink {
    pub Mac: String,
    pub Ipv4: String,
    pub Ipv6: String,
    pub Name: String,
    pub Index: String,
    pub State: String,
}

impl NetLink {
    pub fn get_ipv4_addr(&self) -> String {
        let ifo: Vec<&str> = self.Ipv4.split("/").collect();
        ifo.get(0).unwrap().to_string()
    }
}

#[cfg(feature = "net")]
#[derive(Debug, Default)]
pub struct NetRoute {
    pub Dest: String,
    pub Dev: String,
    pub Gateway: String,
    pub Src: String,
    pub State: String,
}

#[cfg(feature = "net")]
impl NetRoute {
    pub fn subnet_contains(&self, ip: &str) -> bool {
        //TODO: calc
        false
    }
}

#[cfg(feature = "net")]
#[derive(Debug, Default)]
pub struct GatewayInfo {
    pub bHasGateway: bool,
    pub nGatewayCount: u8,
    pub bHasDefaultGateway: bool,
    pub nDefaultGatewayCount: u8,
    pub nLinkUp: u8,
}

#[cfg(feature = "net")]
#[derive(Debug, Default)]
pub struct RouteTable {
    data: Vec<NetRoute>,
}

#[cfg(feature = "net")]
impl RouteTable {
    /// check gateway address
    pub fn Parse(&self) -> GatewayInfo {
        let mut gatewayInfo = GatewayInfo::default();

        debug!("{:?}", self.data);

        if self.data.len() == 0 {
            gatewayInfo.nLinkUp = 0;
            gatewayInfo.bHasGateway = false;
            gatewayInfo.nDefaultGatewayCount = 0;
        }

        for route in &self.data {
            if route.State != "linkdown" {
                gatewayInfo.nLinkUp += 1;
            }

            if route.Gateway != "" {
                gatewayInfo.bHasGateway = true;
                gatewayInfo.nGatewayCount += 1;

                if route.Dest.contains("0.0.0.0") || route.Dest == "default" {
                    gatewayInfo.bHasDefaultGateway = true;
                    gatewayInfo.nDefaultGatewayCount += 1;
                }
            }
        }

        debug!("gatewayInfo:{:?}", gatewayInfo);
        gatewayInfo
    }

    /// get destination hop
    pub fn FindRoute(&self, ip: &str) -> Option<&NetRoute> {
        let mut bHasGateway = false;
        for route in &self.data {
            //check custom route
            if route.Dest != "0.0.0.0/0" && route.Dest != "default" {
                //calculate subnet
                if route.subnet_contains(ip) {
                    bHasGateway = true;
                    return Some(route);
                }
            }
        }

        if !bHasGateway {
            for route in &self.data {
                //check default route
                if route.Dest == "0.0.0.0/0" || route.Dest == "default" {
                    bHasGateway = true;
                    return Some(route);
                }
            }
        }

        None
    }
}

#[cfg(feature = "net")]
#[derive(Debug, Default)]
pub struct PingResult {
    duration: f32,
}

#[cfg(feature = "net")]
#[derive(Debug)]
pub struct TcpPingResult {
    duration: f32,
}

#[cfg(feature = "net")]
#[derive(Debug)]
pub struct NsLookupResult {
    ip_list: Vec<IpAddr>,
}

#[cfg(feature = "net")]
pub trait os_network {
    fn get_interface_list() -> AnyResult<Vec<NetLink>>;
    fn get_route_table() -> AnyResult<RouteTable>;
    fn ping(host: &str) -> AnyResult<PingResult>;
    fn nslookup(host: &str) -> AnyResult<NsLookupResult>;
    fn tcping(host: &str, port: i32) -> AnyResult<TcpPingResult>;
}

#[cfg(feature = "net")]
pub struct LinuxNetwork {}

#[cfg(feature = "net")]
impl LinuxNetwork {
    /// return mac,ipv4,ipv6,index
    pub fn getIfMacIpAddr(ifname: &str) -> AnyResult<NetLink> {
        match Command::new("/bin/ip")
            .args(["addr", "show", ifname])
            .output()
        {
            Ok(output) => {
                let mut netlink = NetLink::default();
                let mut sOutput = String::from_utf8(output.stdout)?;
                //println!("Ouput {}", sOutput);
                let sErr = String::from_utf8(output.stderr)?;
                let lines = sOutput.split("\n");

                let mut mac = "".to_string();
                let mut ip = "".to_string();
                let mut ip6 = "".to_string();
                let mut index = "".to_string();

                let mut counter = 0;

                for line in lines {
                    let line = line.trim();

                    let mut netaddr: Vec<&str> = line.split(" ").collect();

                    if counter == 0 {
                        netlink.Index = netaddr.get(0).unwrap_or(&"").to_string().replace(":", "");
                        counter += 1;
                    }

                    if line.starts_with("link") {
                        netlink.Mac = netaddr.get(1).unwrap_or(&"").to_string();
                    } else if line.starts_with("inet6") {
                        netlink.Ipv6 = netaddr.get(1).unwrap_or(&"").to_string();
                    } else if line.starts_with("inet") {
                        netlink.Ipv4 = netaddr.get(1).unwrap_or(&"").to_string();
                    }
                }
                Ok(netlink)
            }
            Err(e) => {
                println!("getNetAddrError {}", e);
                Err(anyhow!("getNetAddrError".to_string()))
            }
        }
    }

    fn ping_internal(host: &str, diag: bool) -> AnyResult<PingResult> {
        let mut result = PingResult::default();
        match Command::new("/bin/ping")
            .args(["-c", "1", "-W", "1", host])
            .output()
        {
            Ok(output) => {
                let mut sOutput = String::from_utf8(output.stdout)?;
                let mut sError = String::from_utf8(output.stderr)?;

                if sOutput.contains("icmp_seq=") {
                    let resultLines: Vec<&str> = sOutput.split("\n").collect();
                    let response: Vec<&str> =
                        resultLines.get(1).unwrap_or(&"").split("=").collect();
                    let response_time = response.get(3).unwrap_or(&"");
                    let response_list: Vec<&str> = response_time.split(" ").collect();
                    //println!("response_list {:?}", response_list);
                    result.duration = response_list.get(0).unwrap_or(&"").parse::<f32>().unwrap();
                    //println!("sOutputOk {}",response.get(3).unwrap_or(&""));
                } else {
                    //linux no output
                    if sError.is_empty() {
                        if diag {
                            //find other reason
                            let network = LinuxNetwork {};

                            let route_table = LinuxNetwork::get_route_table().unwrap();

                            let routeInfo = route_table.Parse();
                            debug!("GatewayInfo {:?}", routeInfo);

                            if routeInfo.nLinkUp == 0 {
                                error!("Net NoLinkUp");
                                return Err(anyhow!("NoLinkUp"));
                            } else if routeInfo.nGatewayCount == 0 {
                                error!("Net NoGateway");
                                return Err(anyhow!("NoGateway"));
                            } else {
                                //find route by host
                                let route = route_table.FindRoute(host);

                                if route.is_none() {
                                    error!("Net NoRoute");
                                    return Err(anyhow!("NoRoute"));
                                } else {
                                    //check route info
                                    debug!("FoundRoute-{:?}", route);
                                    let gateway = &route.unwrap().Gateway;

                                    match LinuxNetwork::ping_internal(&gateway, false) {
                                        Ok(dt) => {}
                                        Err(e) => {
                                            error!("GatewayNotReachable {}", e);
                                            return Err(anyhow!("GatewayNotReachable"));
                                        }
                                    }
                                }
                            }
                        } else {
                            return Err(anyhow!("NoErrData"));
                        }
                    } else {
                        if sError.contains("ping: unknown host") {
                            sError = "ping: unknown host".to_string()
                        }

                        error!("sError {}", sError);
                        return Err(anyhow!("{}", sError));
                    }
                }
            }
            Err(e) => {
                error!("Err-{}", e);
                return Err(anyhow!("{}", e));
            }
        }

        Ok(result)
    }
}

#[cfg(feature = "net")]
impl os_network for LinuxNetwork {
    fn get_route_table() -> AnyResult<RouteTable> {
        println!("get_route_table");
        match Command::new("/bin/ip")
            .args(["route", "list", "table", "0"])
            .output()
        {
            Ok(output) => {
                let mut result: RouteTable = RouteTable::default();

                let mut sOutput = String::from_utf8(output.stdout)?;
                //println!("Ouput \n{}", sOutput);

                let route_list = sOutput.split("\n");

                for route in route_list {
                    //
                    let items: Vec<&str> = route.split(" ").collect();

                    if items.len() > 4 {
                        let mut route = NetRoute::default();

                        let dst = items.get(0).unwrap_or(&"").to_string();

                        if dst == "local" || dst == "multicast" || dst == "broadcast" {
                            continue;
                        } else {
                            route.Dest = dst;
                        }

                        let mut pos = 0;
                        for mut pos in 0..items.len() {
                            let mut flag = items.get(pos).unwrap_or(&"").to_string();
                            if flag == "dev" {
                                route.Dev = items.get(pos + 1).unwrap_or(&"").to_string();
                                pos += 1;
                            } else if flag == "src" {
                                route.Src = items.get(pos + 1).unwrap_or(&"").to_string();
                                pos += 1;
                            } else if flag == "linkdown" {
                                route.State = flag.to_string();
                                pos += 1;
                            } else if flag == "via" {
                                route.Gateway = items.get(pos + 1).unwrap_or(&"").to_string();
                                pos += 1;
                            }
                        }
                        //

                        let mut dev_flag = items.get(1).unwrap_or(&"").to_string();

                        if dev_flag == "dev" {
                            route.Dev = items.get(2).unwrap_or(&"").to_string();
                        }

                        if route.Dev.starts_with("dummy") {
                            continue;
                        }
                        result.data.push(route);
                    }
                }

                return Ok(result);
            }
            Err(e) => {
                println!("get_route_table_error {}", e);
                Err(anyhow!("get_route_table_error".to_string()))
            }
        }
    }

    fn get_interface_list() -> AnyResult<Vec<NetLink>> {
        let mut result: Vec<NetLink> = vec![];
        match Command::new("/bin/ip").args(["addr"]).output() {
            Ok(output) => {
                let mut sOutput = String::from_utf8(output.stdout)?;
                if sOutput.len() == 0 {
                    let sErr = String::from_utf8(output.stderr)?;
                    return Err(anyhow!("get_interfaces_error {}", sErr));
                }

                let lines = sOutput.split("\n");

                let mut counter = 0;

                let mut netlink = NetLink {
                    Mac: "".to_string(),
                    Ipv4: "".to_string(),
                    Ipv6: "".to_string(),
                    Name: "".to_string(),
                    Index: "".to_string(),
                    State: "".to_string(),
                };

                for line in lines {
                    let line = line.trim();
                    let mut netaddr: Vec<&str> = line.split(" ").collect();

                    if line.contains("state") {
                        //new line
                        counter = 0;
                        if netlink.Name != "" {
                            result.push(netlink.clone());
                        }

                        //state check
                        let mut nPos = 0;
                        for item in netaddr.clone() {
                            if item == "state" {
                                netlink.State = netaddr.get(nPos + 1).unwrap_or(&"").to_string();
                            }
                            nPos += 1;
                        }
                    }

                    if counter == 0 {
                        netlink.Index = netaddr.get(0).unwrap_or(&"").to_string().replace(":", "");
                        netlink.Name = netaddr.get(1).unwrap_or(&"").to_string().replace(":", "");
                        counter += 1;
                    }

                    if line.starts_with("link") {
                        netlink.Mac = netaddr.get(1).unwrap_or(&"").to_string();
                    }

                    if line.starts_with("inet6") {
                        netlink.Ipv6 = netaddr.get(1).unwrap_or(&"").to_string();
                    } else if line.starts_with("inet") {
                        netlink.Ipv4 = netaddr.get(1).unwrap_or(&"").to_string();
                    }
                }
                
                if netlink.Name != "" {
                    result.push(netlink.clone());
                }

                Ok(result)
            }
            Err(e) => {
                error!("get_interfaces_error {}", e);
                Err(anyhow!("get_interfaces_error".to_string()))
            }
        }
    }

    fn ping(host: &str) -> AnyResult<PingResult> {
        LinuxNetwork::ping_internal(host, true)
    }

    fn nslookup(host: &str) -> AnyResult<NsLookupResult> {
        Err(anyhow!("NotImplement"))
    }
    fn tcping(host: &str, port: i32) -> AnyResult<TcpPingResult> {
        Err(anyhow!("NotImplement"))
    }
}

#[cfg(test)]
#[cfg(feature = "net")]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result as AnyResult};

    #[test]
    fn test_get_interface_list() {
        let network = LinuxNetwork {};
        println!("{:?}", LinuxNetwork::get_interface_list());
    }

    #[test]
    fn test_get_interface_list() {
        let network = LinuxNetwork {};
        println!("{:?}", LinuxNetwork::get_interface_list());
    }

    #[test]
    fn test_get_route_table() {
        let network = LinuxNetwork {};

        let route_table = LinuxNetwork::get_route_table().unwrap();

        println!(
            "{:?}/{:?}/test-route {:?}",
            route_table,
            route_table.Parse(),
            route_table.FindRoute("8.8.8.8")
        );
    }

    #[test]
    fn test_ping() {
        let network = LinuxNetwork {};

        let route_table = LinuxNetwork::get_route_table().unwrap();

        println!("{:?}", LinuxNetwork::ping("8.8.8.8"));
        println!("{:?}", LinuxNetwork::ping("1.net"));

        /*println!(
            "{:?}/{:?}/test-route {:?}",
            route_table,
            route_table.Parse(),
            route_table.FindRoute("8.8.8.8")
        );*/
    }
}
