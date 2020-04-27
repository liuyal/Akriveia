// Unfortunately, due to using UDP sockets and broadcasts, it is difficult to associate a single
// message from the manager with a set of responses to the j
extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use actix::io::{ WriteHandler, SinkWrite, };
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler, };
use bytes::{ BytesMut, Bytes };
use crate::beacon_manager::*;
use crate::conn_common;
use futures::stream::SplitSink;
use futures::{ Stream, };
use ipnet::Ipv4Net;
use std::io;
use std::net::{ Ipv4Addr, IpAddr, };
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{ UdpSocket, UdpFramed };

pub struct BeaconUDP {
    bound_ip: Ipv4Net,
    bound_port: u16,
    manager: Addr<BeaconManager>,
    sink: SinkWrite<SplitSink<UdpFramed<BytesCodec>>>,
}

impl WriteHandler<io::Error> for BeaconUDP {
    fn error(&mut self, err: io::Error, _context: &mut Self::Context) -> Running {
        println!("beacon udp encountered an error {}", err);
        Running::Stop
    }

    fn finished(&mut self, _context: &mut Self::Context) {
        // override the finish method of the trait, because the default will stop the actor...
        println!("finish sending data");
    }
}

impl Actor for BeaconUDP {
    type Context = Context<Self>;
}

#[derive(Message)]
struct Frame {
    data: BytesMut,
    addr: SocketAddr
}

impl StreamHandler<Frame, io::Error> for BeaconUDP {
    fn handle(&mut self, msg: Frame, _: &mut Context<Self>) {
        let response = String::from_utf8_lossy(&msg.data);
        match conn_common::parse_message(&response, msg.addr.ip()) {
            Ok(bm_response) => {
                self.manager
                    .do_send(bm_response);
            },
            Err(e) => {
                println!("failed to parse message from udp beacon: {}, {}", e, response);
            }
        }
    }
}

impl Handler<BeaconCommand> for BeaconUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BeaconCommand, _context: &mut Context<Self>) -> Self::Result {
        let (command, ip) = match msg {
            BeaconCommand::StartEmergency(opt_ip)   => self.build_request("[start]".to_owned(), opt_ip),
            BeaconCommand::EndEmergency(opt_ip)     => self.build_request("[end]".to_owned(), opt_ip),
            BeaconCommand::Ping(opt_ip)             => self.build_request("[ping]".to_owned(), opt_ip),
            BeaconCommand::Reboot(opt_ip)           => self.build_request("[reboot]".to_owned(), opt_ip),
            BeaconCommand::SetIp(ip)                => self.build_request(format!("[setip|{}]", ip), None),
        };

        self.sink
            .write((Bytes::from(command), ip))
            .map(|_s| {})
            .map_err(|_e| {})
    }
}

impl BeaconUDP {
    pub fn new(manager: Addr<BeaconManager>, ip: Ipv4Net, port: u16) -> Addr<BeaconUDP> {
        // TODO test is this necessary?
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        let sock = UdpSocket::bind(&bind_addr).unwrap();
        sock.set_broadcast(true).expect("could not set broadcast");

        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(move |context| {
            context.add_stream(stream.map(|(data, sender)| Frame { data, addr: sender }));
            let sw = SinkWrite::new(sink, context);
            BeaconUDP {
                bound_ip: ip,
                bound_port: port,
                manager,
                sink: sw,
            }
        })
    }

    pub fn build_request<'a>(&mut self, command: String, opt_ip: Option<IpAddr>) -> (String, SocketAddr) {
        if let Some(ip) = opt_ip {
            let addr = SocketAddr::new(ip, self.bound_port);
            (command, addr)
        } else {
            let broadcast = SocketAddr::new(IpAddr::V4(self.bound_ip.broadcast()), self.bound_port);
            (command, broadcast)
        }
    }

}

