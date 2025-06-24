#!/usr/bin/env python3

import os
import socket
import rerun as rr
import numpy as np
import struct

def decode_message(data: bytes):
    """Decode bincode message - handles Rust enum serialization."""
    if len(data) < 4:
        return None
    # Read enum variant (u32 little-endian)
    variant = struct.unpack('<I', data[:4])[0]
    pos = 4
    if variant == 0:  # Timeline
        # offset_percentage: f32
        offset_percentage = struct.unpack('<f', data[pos:pos+4])[0]
        pos += 4
        return {
            "Timeline": {
                "offset_percentage": offset_percentage
            }
        }
    elif variant == 1:  # Disconnect
        return "Disconnect"
    return None

if __name__ == "__main__":
    # Define Connection Parameters
    IP_ADDRESS = "127.0.0.1"
    PORT = 9877

    try:
        # Initialize the Rerun client connection
        rr.init("rerun_ros2_bag")
        rr.connect_grpc(f"rerun+http://{IP_ADDRESS}:{PORT}/proxy", flush_timeout_sec=100.0)

        rr.log("points", rr.Points2D([[0, 0], [1, 1]]))

        # take the server name and port name

        HOST            =   "127.0.0.1"  # Standard loopback interface address (localhost)
        PORT            =   8888  # Port to listen on (non-privileged ports are > 1023)
        MAX_CLIENTS     =   1
        TIMEOUT_SECONDS = 5  # Timeout for socket operations

        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.settimeout(TIMEOUT_SECONDS)
                s.bind((HOST, PORT))
                s.listen(MAX_CLIENTS)

                try:
                    conn, addr = s.accept()
                except socket.timeout:
                    print(f"Connection timeout after {TIMEOUT_SECONDS} seconds.")
                    exit(1)

                with conn:
                    print(f"Connected by {addr}")
                    while True:
                        data = conn.recv(1024)
                        if not data:
                            break
                        try:
                            # Decode the bincode data
                            decoded = decode_message(data)
                            print(f"Received data: {decoded}")
                        except Exception as e:
                                print(f"Error decoding message: {e}")
        except Exception as e:
            print(f"Error occurred: {e}")
            rr.disconnect()

    except Exception as e:
        print(f"Failed to connect to Rerun server: {e}")
