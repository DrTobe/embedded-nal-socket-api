fn main() {
    let mut driver = DriverA::new();
    let mut stack = TcpClientStack::new(&mut driver);
    let mut socket0 = stack.socket().unwrap();
    let mut socket1 = stack.socket().unwrap();
    let mut socket2 = stack.socket().unwrap();
    let _ = socket0.connect(SocketAddr);
    let _ = socket1.connect(SocketAddr);
    let _ = socket2.connect(SocketAddr);
    drop(socket0);
    let _socket0_again = stack.socket();
}

//
// NalDriver trait definition
//

pub trait NalDriver {
    type SocketIdentifier: Copy;
    fn socket(&mut self) -> Result<Self::SocketIdentifier, Error>;
    fn connect(&mut self, socket: Self::SocketIdentifier, remote: SocketAddr) -> Result<(), Error>;
    // + send, receive
    fn close(&mut self, socket: Self::SocketIdentifier);
}

//
// This is what could be supplied by embedded-nal
//

#[derive(Clone, Copy)]
pub struct TcpClientStack<'a, D>
where
D: NalDriver
{
    driver: &'a core::cell::RefCell<D>,
}

impl<'a, D> TcpClientStack<'a, D>
where
D: NalDriver
{
    /// Although we only store the Bg77Driver as a `&RefCell` we require a `&mut RefCell`. Like
    /// this, we can be sure that there are no other `&RefCell`s to the driver, only the ones we
    /// control ourselves. This ensures that `driver.borrow_mut()` will never panic because:
    /// 1. Our own `borrow_mut()` calls will never outlive the method calls.
    /// 2. The network stack and the sockets can not be transferred to other threads/contexts (i.e.
    /// they are `!Send + !Sync`).
    pub fn new(
        driver: &'a mut core::cell::RefCell<D>,
    ) -> Self {
        TcpClientStack { driver }
    }

    fn socket(&mut self) -> Result<TcpSocketHandle<'a, D>, Error> {
        Ok(TcpSocketHandle {
            socket: self.driver.borrow_mut().socket()?,
            driver: self.driver,
        })
    }
}

/// Socket handle type for a TCP socket
pub struct TcpSocketHandle<'a, D>
where
D: NalDriver
{
    socket: D::SocketIdentifier,
    driver: &'a core::cell::RefCell<D>,
}

impl <'a, D> TcpSocketHandle<'a, D> 
where
D: NalDriver
{
    fn connect(
        &mut self,
        remote: SocketAddr,
    ) -> Result<(), Error> {
        let mut driver = self.driver.borrow_mut();
        driver.connect(self.socket, remote)
    }

    // + send, receive
}

impl<D> Drop for TcpSocketHandle<'_, D>
where
D: NalDriver
{
    fn drop(&mut self) {
        self.driver.borrow_mut().close(self.socket)
    }
}

//
// This is what the driver developer A has to do
//

/// A Network Driver that supports multiple sockets
pub struct DriverA
{
    sockets: [SocketA; 12],
}

impl DriverA
{
    pub fn new(
    ) -> core::cell::RefCell<Self> {
        let driver = DriverA {
            sockets: [SocketA { state: SocketAState::Available }; 12],
        };
        core::cell::RefCell::new(driver)
    }
}

impl NalDriver for DriverA {
    type SocketIdentifier = usize;

    fn socket(&mut self) -> Result<usize, Error> {
        for i in 0..12 {
            if let SocketAState::Available = self.sockets[i].state {
                println!("Using socket {}", i);
                self.sockets[i].state = SocketAState::Used;
                return Ok(i);
            }
        }
        Err(Error)
    }

    fn connect(
        &mut self,
        socket_index: usize,
        _remote: SocketAddr,
    ) -> Result<(), Error> {
        // ...
        println!("Connecting/Using socket {}", socket_index);
        Ok(())
    }

    fn close(&mut self, socket_index: usize) {
        // ...
        println!("Closing socket {}", socket_index);
        self.sockets[socket_index].state = SocketAState::Available;
    }

    // + send, receive
}

// This is just Clone+Copy to simplify array initialization in DriverA::new
#[derive(Clone, Copy)]
struct SocketA {
    state: SocketAState,
}

#[derive(Clone, Copy, PartialEq)]
enum SocketAState {
    Available,
    Used,
    _Open, // close -> Available again
}

//
// This is what the driver developer B has to do
//

/// A Network Driver that supports a single socket
pub struct DriverB
{
    socket_available: bool,
}

impl DriverB
{
    pub fn new(
    ) -> core::cell::RefCell<Self> {
        let driver = DriverB {
            socket_available: true,
        };
        core::cell::RefCell::new(driver)
    }
}

impl NalDriver for DriverB {
    type SocketIdentifier = ();

    fn socket(&mut self) -> Result<(), Error> {
        if self.socket_available {
            self.socket_available = false;
            Ok(())
        } else {
            Err(Error)
        }
    }

    fn connect(
        &mut self,
        _socket: (),
        _remote: SocketAddr,
    ) -> Result<(), Error> {
        // ...
        println!("Connecting/Using single socket");
        Ok(())
    }

    fn close(&mut self, _socket: ()) {
        // ...
        println!("Closing socket");
        self.socket_available = true;
    }

    // + send, receive
}

//
// Type definitions required so that the above compiles successfully
//

pub struct SocketAddr;
#[derive(Debug)]
pub struct Error;
