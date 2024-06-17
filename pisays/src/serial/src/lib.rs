//! Thanks https://blog.mbedded.ninja/programming/operating-systems/linux/linux-serial-ports-using-c-cpp/
//!
//!
//!

extern crate errno;
extern crate libc;
extern crate termios;

use std::{io, os::raw::c_void};

use errno::{errno, Errno};
use libc::{open, CSIZE, O_NOCTTY, O_RDWR};
use termios::{
    cfsetospeed,
    os::linux::{B115200, CRTSCTS, IUTF8},
    tcgetattr, tcsetattr, Termios, BRKINT, CLOCAL, CREAD, CS8, CSTOPB, ECHO, ECHOE, ECHONL, ICANON,
    ICRNL, IGNBRK, IGNCR, INLCR, ISIG, ISTRIP, IXANY, IXOFF, IXON, ONLCR, OPOST, PARENB, PARMRK,
    TCSANOW,
};

pub(crate) const SERIALPORT: &str = "/dev/ttyS0\0";

pub struct Serial {
    fd: i32,
    tty: Termios,
}

#[derive(Debug)]
pub enum SerialError {
    OS(Errno),
    Termios(io::Error),
}

impl Serial {
    fn init_ctrl(term: &mut Termios) {
        // Clear parity bit, rpi doesn't use it
        term.c_cflag &= !PARENB;

        // Clear stop bit. If this is set, the program will use 2 stop bits,
        // but rpi only uses one.
        term.c_cflag &= !CSTOPB;

        // Set 8 bits per byte - clear first, then set to 8.
        // Just being pedantic at this point.
        term.c_cflag &= !CSIZE;
        term.c_cflag |= CS8;

        // Disable flow control. I only have 3 wires
        term.c_cflag &= !CRTSCTS;

        // Read? Not yet
        term.c_cflag |= !CREAD;

        // Ignore local stuff like carriage return
        term.c_cflag |= CLOCAL;
    }

    fn disable_echo(term: &mut Termios) {
        term.c_lflag &= !ECHO; // no echo
        term.c_lflag &= !ECHOE; // no erasure
        term.c_lflag &= !ECHONL; // no newline echo
    }

    fn init_local(term: &mut Termios, echo: bool) {
        // Disable canonical mode, otherwise the input would be line-by-line
        term.c_lflag &= !ICANON;
        if !echo {
            Self::disable_echo(term);
        }

        // Disable signal chars
        term.c_lflag &= !ISIG;
    }

    fn init_input_modes(term: &mut Termios) {
        // Disable software flow control
        term.c_iflag &= !(IXON | IXOFF | IXANY);

        // Disable any special byte handling
        term.c_iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL);

        // Set UTF-8 so that we don't read junk
        term.c_iflag &= IUTF8;
    }

    fn init_ouput_modes(term: &mut Termios) {
        // Prevent special interpretation of output bytes (e.g. newline chars)
        term.c_oflag &= !OPOST;
        // Prevent conversion of newline to carriage return/line feed
        term.c_oflag &= !ONLCR;
    }

    pub fn new() -> Result<Self, SerialError> {
        let flags = O_RDWR // will write to the serial port
    | O_NOCTTY; // don't make the terminal we're opening the controlling one for this process
        let fd = unsafe { open(SERIALPORT.as_ptr(), flags) };
        if fd < 0 {
            return Err(SerialError::OS(errno()));
        }
        let mut term = Termios::from_fd(fd).map_err(SerialError::Termios)?;

        // First, tcgetattr to populate with current settings.
        // POSIX is particular about this
        // https://pubs.opengroup.org/onlinepubs/007904875/functions/tcgetattr.html
        //
        // "Care must be taken when changing the terminal attributes.
        // Applications should always do a tcgetattr(), save the termios
        // structure values returned, and then do a tcsetattr(), changing only
        // the necessary fields. The application should use the values saved
        // from the tcgetattr() to reset the terminal state whenever it is done
        // with the terminal. "

        // TODO set signal handlers

        tcgetattr(fd, &mut term).map_err(SerialError::Termios)?;

        Self::init_ctrl(&mut term);
        Self::init_local(&mut term, true);
        Self::init_input_modes(&mut term);
        Self::init_ouput_modes(&mut term);

        // Uncomment if/when reading
        // // How to poll/wait?
        // // Let's wait for up to 1s (that means 10 deciseconds...sheesh) and
        // // return as soon as any data is received (1 or more bytes depending on
        // // buffering).
        // term.c_cc[VTIME] = 10;
        // term.c_cc[VMIN] = 0;

        // Set (out) baud rate
        cfsetospeed(&mut term, B115200).map_err(SerialError::Termios)?;
        // Set both baud rates hehe
        // cfsetspeed(&mut term, B115200).map_err(SerialError::Termios)?;

        tcsetattr(fd, TCSANOW, &term).map_err(SerialError::Termios)?;

        Ok(Serial { fd, tty: term })
    }

    pub fn write(&self, msg: &str) -> Result<(), SerialError> {
        // Safe because the input is a valid slice and fd is a valid descriptor.
        if unsafe { libc::write(self.fd, msg.as_ptr() as *const c_void, msg.len()) } < 0 {
            return Err(SerialError::OS(errno()));
        }
        Ok(())
    }
}

impl Drop for Serial {
    fn drop(&mut self) {
        // Safe because self.fd was checked in the constructor and is valid.
        unsafe { libc::close(self.fd) };
    }
}
