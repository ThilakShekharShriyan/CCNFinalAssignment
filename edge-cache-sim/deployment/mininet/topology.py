#!/usr/bin/env python3
"""
Mininet lab launcher for the real edge-cache prototype.

Creates:
  - user
  - edge1, edge2, edge3
  - origin

Starts:
  - origin_server on origin host
  - edge_server on edge hosts
Optionally starts workload_replay from user host.
"""

import argparse
import os
import time
from mininet.cli import CLI
from mininet.link import TCLink
from mininet.log import info, setLogLevel
from mininet.net import Mininet
from mininet.node import OVSController


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--binary-dir", default=os.path.abspath("target/release"))
    p.add_argument("--edge-delay-ms", type=int, default=2)
    p.add_argument("--edge-bw-mbit", type=int, default=500)
    p.add_argument("--origin-delay-ms", type=int, default=35)
    p.add_argument("--origin-bw-mbit", type=int, default=12)
    p.add_argument("--run-replay", action="store_true")
    p.add_argument("--replay-requests", type=int, default=2000)
    return p.parse_args()


def cmd_bg(host, command):
    host.cmd(f"nohup {command} > /tmp/{host.name}.log 2>&1 &")


def main():
    args = parse_args()
    setLogLevel("info")

    net = Mininet(controller=OVSController, link=TCLink, autoSetMacs=True, build=False)
    c0 = net.addController("c0")
    s1 = net.addSwitch("s1")

    user = net.addHost("user", ip="10.0.0.10/24")
    edge1 = net.addHost("edge1", ip="10.0.0.11/24")
    edge2 = net.addHost("edge2", ip="10.0.0.12/24")
    edge3 = net.addHost("edge3", ip="10.0.0.13/24")
    origin = net.addHost("origin", ip="10.0.0.20/24")

    # user/edges share low-latency local links
    for h in [user, edge1, edge2, edge3]:
        net.addLink(
            h,
            s1,
            cls=TCLink,
            delay=f"{args.edge_delay_ms}ms",
            bw=args.edge_bw_mbit,
        )
    # origin link is slower by default
    net.addLink(
        origin,
        s1,
        cls=TCLink,
        delay=f"{args.origin_delay_ms}ms",
        bw=args.origin_bw_mbit,
    )

    net.build()
    c0.start()
    s1.start([c0])

    bin_dir = args.binary_dir
    origin_bin = os.path.join(bin_dir, "origin_server")
    edge_bin = os.path.join(bin_dir, "edge_server")
    replay_bin = os.path.join(bin_dir, "workload_replay")

    info("*** Starting origin service\n")
    cmd_bg(
        origin,
        f"{origin_bin} --http-addr 10.0.0.20:7000 --grpc-addr 10.0.0.20:7100 "
        f"--objects 1024 --chunks-per-object 4 --chunk-size-kib 64",
    )

    time.sleep(1)
    info("*** Starting edge services\n")
    cmd_bg(
        edge1,
        f"{edge_bin} --node-name edge1 --http-addr 10.0.0.11:7001 "
        f"--origin-http http://10.0.0.20:7000 --origin-grpc http://10.0.0.20:7100 "
        f"--neighbors http://10.0.0.12:7002,http://10.0.0.13:7003 --origin-protocol http",
    )
    cmd_bg(
        edge2,
        f"{edge_bin} --node-name edge2 --http-addr 10.0.0.12:7002 "
        f"--origin-http http://10.0.0.20:7000 --origin-grpc http://10.0.0.20:7100 "
        f"--neighbors http://10.0.0.11:7001,http://10.0.0.13:7003 --origin-protocol http",
    )
    cmd_bg(
        edge3,
        f"{edge_bin} --node-name edge3 --http-addr 10.0.0.13:7003 "
        f"--origin-http http://10.0.0.20:7000 --origin-grpc http://10.0.0.20:7100 "
        f"--neighbors http://10.0.0.11:7001,http://10.0.0.12:7002 --origin-protocol http",
    )

    if args.run_replay:
        time.sleep(1)
        info("*** Running workload replay once from user host\n")
        user.cmd(
            f"{replay_bin} "
            f"--edges http://10.0.0.11:7001,http://10.0.0.12:7002,http://10.0.0.13:7003 "
            f"--requests {args.replay_requests} "
            f"--csv-out /tmp/mininet_replay.csv"
        )
        info("*** replay output: /tmp/mininet_replay.csv on user host\n")

    info("*** Mininet is ready. Use CLI to inspect hosts/logs.\n")
    info("*** Example: edge1 curl -s http://10.0.0.11:7001/metrics\n")
    CLI(net)
    net.stop()


if __name__ == "__main__":
    main()
